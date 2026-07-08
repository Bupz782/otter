#!/usr/bin/env bash
#
# Setup GitHub issue labels for the otter project.
# Run this once to create wave/epic labels for filtering.
#
# Requires: gh CLI authenticated (`gh auth login`)

set -euo pipefail

REPO="${REPO:-Bupz782/metis}"

echo "Setting up labels for $REPO..."

# Wave labels (color-coded by wave)
declare -a WAVE_LABELS=(
  "vague-0|0D1117|Setup & Foundation"
  "vague-1|1D4ED8|Intent Parsing"
  "vague-2|7C3AED|ZKP Delegation"
  "vague-3|C026FC|FHE Calculations"
  "vague-4|DB2777|Encrypted Mempool"
  "vague-5|EA580C|Blockchain Integration"
  "vague-6|CA8A04|Orchestrator"
  "vague-6.5|65A30D|DAPP Frontend"
  "vague-7|16A34A|Production"
  "vague-8|0891B2|Advanced Features"
  "vague-9|2563EB|Research"
  "vague-10|4F46E5|Community"
)

# Status labels
# Use hex colors without #
declare -a STATUS_LABELS=(
  "status-pending|FEF3C7|Not started"
  "status-progress|DBEAFE|In progress"
  "status-done|D1FAE5|Done"
  "status-blocked|FEE2E2|Blocked"
)

# Type labels
declare -a TYPE_LABELS=(
  "type-feature|A5F3FC|New feature"
  "type-bug|FECACA|Bug fix"
  "type-test|E9D5FF|Testing"
  "type-docs|BFDBFE|Documentation"
  "type-refactor|E5E7EB|Refactoring"
  "type-research|FDE68A|Research"
)

create_label() {
  local name="$1"
  local color="$2"
  local desc="$3"

  if gh label create "$name" --color "$color" --description "$desc" --repo "$REPO" 2>/dev/null; then
    echo "  ✓ Created label: $name"
  else
    echo "  ⚠ Label '$name' may already exist or creation failed"
  fi
}

echo ""
echo "=== Wave Labels ==="
for item in "${WAVE_LABELS[@]}"; do
  IFS='|' read -r name color desc <<< "$item"
  create_label "$name" "$color" "$desc"
done

echo ""
echo "=== Status Labels ==="
for item in "${STATUS_LABELS[@]}"; do
  IFS='|' read -r name color desc <<< "$item"
  create_label "$name" "$color" "$desc"
done

echo ""
echo "=== Type Labels ==="
for item in "${TYPE_LABELS[@]}"; do
  IFS='|' read -r name color desc <<< "$item"
  create_label "$name" "$color" "$desc"
done

echo ""
echo "Done! Verify with: gh label list --repo $REPO"
