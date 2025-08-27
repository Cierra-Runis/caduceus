#!/bin/bash

if [ $# -ne 2 ]; then
    echo "Usage: $0 <source_branch> <target_branch>"
    exit 1
fi

SOURCE_BRANCH="$1"
TARGET_BRANCH="$2"

git log "$SOURCE_BRANCH..$TARGET_BRANCH" --oneline --reverse | while read -r line; do
    hash=$(echo "$line" | cut -d' ' -f1)

    git cherry-pick "$hash" -n

    mapfile -t msg_lines < <(git log -1 --pretty=format:%B "$hash")

    commit_args=()
    for line in "${msg_lines[@]}"; do
        commit_args+=("-m" "$line")
    done

    git commit "${commit_args[@]}"
done