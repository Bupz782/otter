#!/usr/bin/env bash
# Prouve la chaine de notification des alertes Otter de bout en bout :
# alerte injectee dans Alertmanager -> webhook -> reception HTTP horodatee.
#
# La preuve (payload JSON recu) est consignee dans docs/preuves/.
#
# Prerequis : docker compose avec le profil monitoring (demarre par ce script).
set -euo pipefail

PORT="${ALERT_TEST_PORT:-9876}"
ALERTMANAGER_URL="${ALERTMANAGER_URL:-http://localhost:9093}"
OUT_DIR="docs/preuves"
STAMP="$(date +%Y%m%d-%H%M%S)"
OUT_FILE="$OUT_DIR/alerte-test-$STAMP.json"
CAPTURE_LOG="$(mktemp)"

mkdir -p "$OUT_DIR"

echo "==> Demarrage d'Alertmanager avec webhook de capture local (port $PORT)"
export ALERT_WEBHOOK_URL="http://host.docker.internal:$PORT/alert"
docker compose --profile monitoring up -d alertmanager >/dev/null

for _ in {1..30}; do
    if curl -fsS "$ALERTMANAGER_URL/-/ready" >/dev/null 2>&1; then
        break
    fi
    sleep 1
done
if ! curl -fsS "$ALERTMANAGER_URL/-/ready" >/dev/null 2>&1; then
    echo "ERROR: Alertmanager n'est pas ready sur $ALERTMANAGER_URL" >&2
    exit 1
fi

echo "==> Demarrage du serveur de capture HTTP"
python3 - "$PORT" "$CAPTURE_LOG" <<'PY' &
import json
import sys
from http.server import BaseHTTPRequestHandler, HTTPServer

port, log_path = int(sys.argv[1]), sys.argv[2]


class Handler(BaseHTTPRequestHandler):
    def do_POST(self):
        body = self.rfile.read(int(self.headers.get("Content-Length", 0)))
        with open(log_path, "a") as fh:
            fh.write(body.decode() + "\n")
        self.send_response(200)
        self.end_headers()

    def log_message(self, *args):
        pass


HTTPServer(("0.0.0.0", port), Handler).serve_forever()
PY
CAPTURE_PID=$!
trap 'kill $CAPTURE_PID 2>/dev/null || true; wait $CAPTURE_PID 2>/dev/null || true; rm -f "$CAPTURE_LOG"' EXIT
sleep 1

echo "==> Injection d'une alerte de test dans Alertmanager"
curl -fsS -X POST "$ALERTMANAGER_URL/api/v2/alerts" \
  -H 'Content-Type: application/json' \
  -d '[{
    "labels": {"alertname": "OtterAgentDown", "severity": "critical", "job": "otter-api", "test": "true"},
    "annotations": {"summary": "Test du canal de notification Otter", "description": "Alerte synthetique injectee par scripts/test-alert.sh"},
    "startsAt": "'"$(date -u +%Y-%m-%dT%H:%M:%SZ)"'"
  }]' >/dev/null

echo "==> Attente de la notification (group_wait 10s + marge)"
received=""
for _ in {1..30}; do
    if [[ -s "$CAPTURE_LOG" ]]; then
        received="yes"
        break
    fi
    sleep 1
done
if [[ -z "$received" ]]; then
    echo "ERROR: aucune notification recue apres 30 secondes" >&2
    exit 1
fi

{
    echo "{"
    echo "  \"preuve\": \"Notification Alertmanager recue par le webhook de capture\","
    echo "  \"date_reception\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\","
    echo "  \"alertmanager_url\": \"$ALERTMANAGER_URL\","
    echo "  \"webhook_url\": \"$ALERT_WEBHOOK_URL\","
    echo "  \"payload\": $(head -n 1 "$CAPTURE_LOG")"
    echo "}"
} > "$OUT_FILE"

echo "==> Notification recue, preuve consignee dans $OUT_FILE"
cat "$OUT_FILE"
