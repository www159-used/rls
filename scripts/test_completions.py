import pathlib
import shutil
import subprocess
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]

class CompletionTests(unittest.TestCase):
    def test_bash_candidates(self):
        script = r'''
source "$1/completions/rls.bash"
rls() { [[ "$1" == completion-branches ]] || return 1; printf '%s\n' main maintenance; }
COMP_WORDS=(rls m mai); COMP_CWORD=2; _rls_complete
[[ "${COMPREPLY[*]}" == "main maintenance" ]] || exit 1
COMP_WORDS=(rls --lang en t --p); COMP_CWORD=4; _rls_complete
[[ "${COMPREPLY[*]}" == --push ]] || exit 2
COMP_WORDS=(rls t --lang z); COMP_CWORD=3; _rls_complete
[[ "${COMPREPLY[*]}" == zh-CN ]] || exit 3
'''
        subprocess.run(['bash', '-c', script, '_', str(ROOT)], check=True)

    def test_shell_syntax(self):
        for shell, file in [('bash', 'rls.bash'), ('zsh', '_rls'), ('fish', 'rls.fish')]:
            if shutil.which(shell):
                subprocess.run([shell, '-n', str(ROOT / 'completions' / file)], check=True)
