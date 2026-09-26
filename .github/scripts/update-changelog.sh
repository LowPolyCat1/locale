#!/usr/bin/env bash
# Adds CHANGELOG.md entries for every released tag newer than the newest version
# already in the file, newest first. The notes come from the tag's GitHub release
# (falling back to generated notes).
#
# Usage: update-changelog.sh <released-tag>
# Env:   GH_TOKEN, REPO (owner/name), SERVER_URL (e.g. https://github.com)
# Writes `versions` (e.g. "v0.3.2, v0.3.3"; empty if nothing was added) to
# $GITHUB_OUTPUT when set.
set -euo pipefail

TAG="${1:?usage: update-changelog.sh <released-tag>}"
FILE=CHANGELOG.md

out() {
    echo "$1=$2"
    if [ -n "${GITHUB_OUTPUT:-}" ]; then echo "$1=$2" >> "$GITHUB_OUTPUT"; fi
}

# a <= b for versions without a leading "v"
version_le() { [ "$(printf '%s\n%s\n' "$1" "$2" | sort -V | head -n1)" = "$1" ]; }

prev_tag() { git describe --tags --abbrev=0 --match 'v*' "$1^" 2>/dev/null || true; }

notes_for() {
    local tag="$1" prev
    if gh release view "$tag" --repo "$REPO" --json body --jq .body 2>/dev/null; then
        return
    fi
    prev=$(prev_tag "$tag")
    local args=(-f tag_name="$tag" -f target_commitish="$(git rev-parse "$tag^{commit}")")
    if [ -n "$prev" ]; then args+=(-f previous_tag_name="$prev"); fi
    gh api "repos/$REPO/releases/generate-notes" "${args[@]}" --jq .body
}

if [ ! -f "$FILE" ]; then
    printf '# Changelog\n\nAll notable changes to locale-rs are documented here.\n' > "$FILE"
fi

latest=$(sed -n 's/^## \[\([^]]*\)\].*/\1/p' "$FILE" | head -n1)
if [ -z "$latest" ]; then
    # Empty changelog: start with the released tag only.
    latest=$(prev_tag "$TAG")
    latest="${latest#v}"
    latest="${latest:-0.0.0}"
fi
echo "Newest version in $FILE: $latest"

entries=$(mktemp)
added=()
for tag in $(git tag -l 'v*' --sort=-v:refname); do
    version="${tag#v}"
    if version_le "$version" "$latest"; then break; fi
    echo "Adding $tag"
    date=$(git log -1 --format=%cs "$tag")
    {
        echo "## [$version]($SERVER_URL/$REPO/releases/tag/$tag) - $date"
        echo
        # Nest the release note headings below the version heading.
        notes_for "$tag" | sed -E 's/^(#+) /\1# /'
        echo
    } >> "$entries"
    added+=("$tag")
done

if [ ${#added[@]} -eq 0 ]; then
    echo "$FILE is up to date."
    out versions ""
    exit 0
fi

# Insert above the first existing version entry (or append to the header).
awk -v entry="$entries" '
    !done && /^## / { while ((getline l < entry) > 0) print l; done = 1 }
    { print }
    END { if (!done) { print ""; while ((getline l < entry) > 0) print l } }
' "$FILE" > "$FILE.tmp"
mv "$FILE.tmp" "$FILE"

# Oldest first in the PR title, e.g. "v0.3.2, v0.3.3".
list=$(printf '%s\n' "${added[@]}" | sort -V | paste -sd, - | sed 's/,/, /g')
out versions "$list"
