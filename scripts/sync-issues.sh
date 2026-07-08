#!/usr/bin/env bash
#
# Sync BACKLOG.md user stories to GitHub Issues.
#
# Usage:
#   ./scripts/sync-issues.sh           # Dry run (default)
#   ./scripts/sync-issues.sh --create  # Actually create issues
#   ./scripts/sync-issues.sh --wave 2  # Only sync Vague 2 stories
#
# Requires: gh CLI authenticated (`gh auth login`)

set -euo pipefail

REPO="${REPO:-Bupz782/metis}"
BACKLOG="${BACKLOG:-BACKLOG.md}"
MODE="dry-run"
WAVE_FILTER=""

# Parse args
while [[ $# -gt 0 ]]; do
  case "$1" in
    --create)
      MODE="create"
      shift
      ;;
    --wave)
      WAVE_FILTER="$2"
      shift 2
      ;;
    --help|-h)
      echo "Usage: $0 [--create] [--wave N]"
      echo ""
      echo "Options:"
      echo "  --create    Actually create/update GitHub issues"
      echo "  --wave N    Only process stories from Vague N"
      echo "  --help      Show this help"
      exit 0
      ;;
    *)
      echo "Unknown option: $1"
      exit 1
      ;;
  esac
done

if [[ ! -f "$BACKLOG" ]]; then
  echo "Error: $BACKLOG not found. Run from repo root."
  exit 1
fi

echo "Mode: $MODE"
echo "Repo: $REPO"
if [[ -n "$WAVE_FILTER" ]]; then
  echo "Wave filter: $WAVE_FILTER"
fi
echo ""

# Get existing issues to avoid duplicates
existing_issues=""
if [[ "$MODE" == "create" ]]; then
  echo "Fetching existing issues..."
  existing_issues=$(gh issue list --repo "$REPO" --limit 1000 --json number,title --jq '.[] | .title' 2>/dev/null || true)
fi

issue_exists() {
  local title="$1"
  if [[ -z "$existing_issues" ]]; then
    return 1
  fi
  echo "$existing_issues" | grep -qF "$title"
}

current_wave=""
current_epic=""
created=0
skipped=0
errors=0

# Parse BACKLOG.md line by line
while IFS= read -r line; do
  # Detect wave headers like "## 🌊 Vague 2 : ..."
  if [[ "$line" =~ ^##\ +🌊\ +Vague\ +([0-9\.]+)\ *: ]]; then
    current_wave="${BASH_REMATCH[1]}"
    current_epic=""
    continue
  fi

  # Detect epic headers like "### Epic 2.1 : ..."
  if [[ "$line" =~ ^###\ +Epic\ +([0-9\.]+)\ *: ]]; then
    current_epic="${BASH_REMATCH[1]}"
    continue
  fi

  # Detect user stories like "- ⏳ **US-042** : En tant que ..."
  if [[ "$line" =~ ^-[[:space:]]+([✅🚧⏳])[[:space:]]+\*\*US-([0-9]+)\*\*[[:space:]]*:[[:space:]]*(.+)$ ]]; then
    status_emoji="${BASH_REMATCH[1]}"
    us_num="${BASH_REMATCH[2]}"
    us_title="${BASH_REMATCH[3]}"

    # Skip if wave filter is set
    if [[ -n "$WAVE_FILTER" && "$current_wave" != "$WAVE_FILTER" ]]; then
      continue
    fi

    # Determine labels
    labels="vague-$current_wave"
    if [[ -n "$current_epic" ]]; then
      labels="$labels,epic-$current_epic"
    fi

    # Determine status label
    case "$status_emoji" in
      ✅) labels="$labels,status-done" ;;
      🚧) labels="$labels,status-progress" ;;
      ⏳) labels="$labels,status-pending" ;;
    esac

    # Build acceptance criteria and technical notes based on story content
    acceptance_criteria=""
    technical_notes=""
    definition_of_done=""

    # Infer acceptance criteria from keywords in the story
    if [[ "$us_title" =~ benchmark|mesurer|profiler ]]; then
      acceptance_criteria="$acceptance_criteria\n- [ ] Benchmarks écrits et résultats documentés"
    fi
    if [[ "$us_title" =~ test|tester|verifier|prouver ]]; then
      acceptance_criteria="$acceptance_criteria\n- [ ] Tests unitaires couvrant les cas nominal et d'erreur"
      acceptance_criteria="$acceptance_criteria\n- [ ] Tests de edge cases identifiés"
    fi
    if [[ "$us_title" =~ circuit|Noir|preuve|verifier|ZKP ]]; then
      acceptance_criteria="$acceptance_criteria\n- [ ] Circuit compile sans erreur (`nargo compile`)"
      acceptance_criteria="$acceptance_criteria\n- [ ] Preuve générée et vérifiée (`nargo prove` / `nargo verify`)"
      technical_notes="$technical_notes\n- **Crate:** `delegation_circuit/`"
      technical_notes="$technical_notes\n- **Ref:** Voir [Noir docs](https://noir-lang.org/docs/)"
    fi
    if [[ "$us_title" =~ contrat|contract|vault|verifier|sol ]]; then
      acceptance_criteria="$acceptance_criteria\n- [ ] Contrat compile (`forge build`)"
      acceptance_criteria="$acceptance_criteria\n- [ ] Tests Foundry passent (`forge test`)"
      technical_notes="$technical_notes\n- **Directory:** `contracts/src/`"
      technical_notes="$technical_notes\n- **Ref:** Voir [Foundry book](https://book.getfoundry.sh/)"
    fi
    if [[ "$us_title" =~ CLI|metis\ |user.*veut\ (ex|voir|executer) ]]; then
      acceptance_criteria="$acceptance_criteria\n- [ ] Commande CLI fonctionne et retourne la sortie attendue"
      acceptance_criteria="$acceptance_criteria\n- [ ] Help text et error messages clairs"
      technical_notes="$technical_notes\n- **Crate:** `crates/interfaces/src/cli/`"
    fi
    if [[ "$us_title" =~ adapter|infra|port ]]; then
      acceptance_criteria="$acceptance_criteria\n- [ ] Adapter implémente le trait du domaine"
      acceptance_criteria="$acceptance_criteria\n- [ ] Errors propres et typed"
      technical_notes="$technical_notes\n- **Crate:** `crates/infrastructure/src/`"
    fi
    if [[ "$us_title" =~ use\ case|application ]]; then
      acceptance_criteria="$acceptance_criteria\n- [ ] Use case callable depuis l'orchestrator"
      acceptance_criteria="$acceptance_criteria\n- [ ] Input/output typés et validés"
      technical_notes="$technical_notes\n- **Crate:** `crates/application/src/use_cases/`"
    fi
    if [[ "$us_title" =~ domain|struct|enum|trait ]]; then
      acceptance_criteria="$acceptance_criteria\n- [ ] Types définis dans `crates/domain/src/`"
      acceptance_criteria="$acceptance_criteria\n- [ ] Derives appropriés (Debug, Clone, PartialEq, etc.)"
      technical_notes="$technical_notes\n- **Crate:** `crates/domain/src/`"
    fi
    if [[ "$us_title" =~ FHE|tfhe|chiffr|homomorphe ]]; then
      acceptance_criteria="$acceptance_criteria\n- [ ] Opération testée avec données réelles (encrypt → op → decrypt)"
      acceptance_criteria="$acceptance_criteria\n- [ ] Benchmarks de performance inclus"
      technical_notes="$technical_notes\n- **Crate:** `crates/infrastructure/src/fhe/`"
      technical_notes="$technical_notes\n- **Ref:** [tfhe-rs docs](https://docs.zama.ai/tfhe-rs)"
    fi
    if [[ "$us_title" =~ LLM|Claude|prompt|parser ]]; then
      acceptance_criteria="$acceptance_criteria\n- [ ] Prompt structuré et testé"
      acceptance_criteria="$acceptance_criteria\n- [ ] Fallback vers rule-based parser fonctionne"
      technical_notes="$technical_notes\n- **Crate:** `crates/infrastructure/src/llm/`"
    fi

    # Default DoD if nothing specific was inferred
    if [[ -z "$acceptance_criteria" ]]; then
      acceptance_criteria="\n- [ ] Implémentation complète et fonctionnelle"
      acceptance_criteria="$acceptance_criteria\n- [ ] Cas d'erreur gérés"
    fi

    # Standard Definition of Done
    definition_of_done="\n- [ ] Code reviewé (self-review ou pair)"
    definition_of_done="$definition_of_done\n- [ ] Tests passent (`cargo test`)"
    definition_of_done="$definition_of_done\n- [ ] Clippy clean (`cargo clippy`)"
    definition_of_done="$definition_of_done\n- [ ] Documenté (comments / rustdoc / LEARNING.md si pertinent)"

    # Build issue title and body
    issue_title="US-$us_num: $us_title"
    issue_body="## 📖 User Story
**$us_title**

## 📊 Metadata
| Field | Value |
|-------|-------|
| **Wave** | Vague $current_wave |
| **Epic** | Epic $current_epic |
| **Status** | $status_emoji |
| **Story ID** | US-$us_num |

## ✅ Acceptance Criteria$acceptance_criteria

## 📝 Definition of Done$definition_of_done"

    # Add technical notes if we have any
    if [[ -n "$technical_notes" ]]; then
      issue_body="$issue_body

## 🔧 Technical Notes$technical_notes"
    fi

    # Add related stories hint for waves > 0
    if [[ "$current_wave" != "0" ]]; then
      issue_body="$issue_body

## 🔗 Related
- [BACKLOG.md — Vague $current_wave](../blob/main/BACKLOG.md#-vague-$current_wave)"
    fi

    issue_body="$issue_body

---
*Generated from [BACKLOG.md](../blob/main/BACKLOG.md). Update status there first, then close this issue.*"

    if issue_exists "$issue_title"; then
      echo "  ↷ SKIP (exists): US-$us_num"
      ((skipped++)) || true
      continue
    fi

    if [[ "$MODE" == "dry-run" ]]; then
      echo "  [DRY-RUN] Would create: US-$us_num ($status_emoji) [labels: $labels]"
      echo "            Title: $issue_title"
      ((created++)) || true
    else
      if gh issue create \
        --repo "$REPO" \
        --title "$issue_title" \
        --body "$issue_body" \
        --label "$labels" 2>/dev/null; then
        echo "  ✓ CREATED: US-$us_num"
        ((created++)) || true
      else
        echo "  ✗ ERROR: US-$us_num (creation failed)"
        ((errors++)) || true
      fi
    fi
  fi
done < "$BACKLOG"

echo ""
echo "========================================"
echo "Summary:"
echo "  Created/Would create: $created"
echo "  Skipped (existing):   $skipped"
echo "  Errors:               $errors"
echo "========================================"

if [[ "$MODE" == "dry-run" ]]; then
  echo ""
  echo "This was a dry run. To actually create issues, run:"
  echo "  $0 --create"
  if [[ -n "$WAVE_FILTER" ]]; then
    echo "  $0 --create --wave $WAVE_FILTER"
  fi
fi
