# Script naming is configuration-driven; --name bypasses scripts and completion never runs them.
# Hand-maintained rls completion.
complete -c rls -f
complete -c rls -n 'not __fish_seen_subcommand_from message msg m tag t' -a 'message msg m tag t'
complete -c rls -s h -l help -d Help
complete -c rls -n 'not __fish_seen_subcommand_from message msg m tag t' -s V -l version
complete -c rls -n '__fish_seen_subcommand_from message msg m tag t' -a '(rls completion-branches 2>/dev/null)'
complete -c rls -n '__fish_seen_subcommand_from message msg m' -s o -l output -r -F
complete -c rls -n '__fish_seen_subcommand_from message msg m' -s c -l copy
complete -c rls -n '__fish_seen_subcommand_from tag t' -s n -l dry-run
complete -c rls -n '__fish_seen_subcommand_from tag t' -s p -l push
complete -c rls -n '__fish_seen_subcommand_from tag t' -s r -l remote -r -a '(git remote 2>/dev/null)'
complete -c rls -n '__fish_seen_subcommand_from tag t' -l name -r -a '(git tag --list 2>/dev/null)'
complete -c rls -l lang -r -a 'en zh-CN'
