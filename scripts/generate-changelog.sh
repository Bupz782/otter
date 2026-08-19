#!/usr/bin/env bash
# Regenere CHANGELOG.md depuis l'historique git (Conventional Commits).
# Usage : scripts/generate-changelog.sh
set -euo pipefail

OUT="CHANGELOG.md"

# Versions = tags v* tries du plus recent au plus ancien ; la section
# "Unreleased" couvre les commits posterieurs au dernier tag.
tags=()
while IFS= read -r tag; do
    tags+=("$tag")
done < <(git tag -l 'v*' --sort=-v:refname)

emit_section() {
    local title="$1" range="$2"
    local feats="" fixes="" chores="" others=""
    local line subject
    while IFS= read -r line || [[ -n "$line" ]]; do
        subject="${line%% (*}"
        case "$subject" in
            feat:*|feat\(*\):*) feats+="- $line"$'\n' ;;
            fix:*|fix\(*\):*)   fixes+="- $line"$'\n' ;;
            chore:*|chore\(*\):*) chores+="- $line"$'\n' ;;
            *)                  others+="- $line"$'\n' ;;
        esac
    done < <(git log "$range" --no-merges --pretty=format:'%s (%h)')

    echo "## $title"
    echo
    for group in "Features|$feats" "Fixes|$fixes" "Chores|$chores" "Other|$others"; do
        local name="${group%%|*}" body="${group#*|}"
        if [[ -n "$body" ]]; then
            echo "### $name"
            echo
            printf '%s' "$body"
            echo
        fi
    done
}

{
    echo "# Changelog"
    echo
    echo "Journal des versions d'Otter, genere depuis l'historique git"
    echo "(Conventional Commits) par \`scripts/generate-changelog.sh\`."
    echo
    if [[ ${#tags[@]} -eq 0 ]]; then
        emit_section "[Unreleased]" "HEAD"
    else
        head_commit=$(git rev-list -n 1 "${tags[0]}")
        if [[ "$(git rev-parse HEAD)" != "$head_commit" ]]; then
            emit_section "[Unreleased]" "${tags[0]}..HEAD"
        fi
        prev=""
        for tag in "${tags[@]}"; do
            date=$(git log -1 --pretty=format:'%ad' --date=short "$tag")
            if [[ -n "$prev" ]]; then
                emit_section "[$prev] - $prev_date" "$tag..$prev"
            fi
            prev="$tag"
            prev_date="$date"
        done
        emit_section "[$prev] - $prev_date" "$prev"
    fi
} > "$OUT"

echo "==> $OUT regenere"
