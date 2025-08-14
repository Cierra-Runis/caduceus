#!/bin/bash

git log dev..origin/draft --oneline --reverse | while read -r line; do
    hash=$(echo "$line" | cut -d' ' -f1)

    git cherry-pick "$hash" -n

    mapfile -t msg_lines < <(git log -1 --pretty=format:%B "$hash")

    commit_args=()
    for line in "${msg_lines[@]}"; do
        commit_args+=("-m" "$line")
    done

    git commit "${commit_args[@]}"
done