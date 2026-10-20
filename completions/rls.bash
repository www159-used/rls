# Hand-maintained rls completion; requires the release CLI on PATH.
_rls_complete() {
    local cur prev cmd candidates candidate
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    cmd=""
    local i skip=0
    for ((i=1; i<COMP_CWORD; i++)); do
        if (( skip )); then skip=0; continue; fi
        case "${COMP_WORDS[i]}" in
            --lang) skip=1 ;;
            --lang=*) ;;
            message|msg|m|tag|t) cmd="${COMP_WORDS[i]}"; break ;;
        esac
    done
    COMPREPLY=()
    if [[ "$prev" == --lang ]]; then
        candidates="en zh-CN"
    elif [[ -z "$cmd" ]]; then
        candidates='message msg m tag t --help --version --lang'
    else
        case "$prev" in
            --lang) candidates="en zh-CN" ;;
            -o|--output) while IFS= read -r candidate; do COMPREPLY+=("$candidate"); done < <(compgen -f -- "$cur"); return ;;
            -r|--remote) candidates="$(git remote 2>/dev/null)" ;;
            --name) candidates="$(git tag --list 2>/dev/null)" ;;
            *)
                case "$cmd" in
                    message|msg|m) candidates='--output -o --copy -c --help -h --lang' ;;
                    tag|t) candidates='--dry-run -n --push -p --remote -r --name --help -h --lang' ;;
                    *) return ;;
                esac
                if [[ "$cur" != -* ]]; then candidates="$(rls completion-branches 2>/dev/null)"; fi
                ;;
        esac
    fi
    while IFS= read -r candidate; do COMPREPLY+=("$candidate"); done < <(compgen -W "$candidates" -- "$cur")
}
complete -F _rls_complete rls
