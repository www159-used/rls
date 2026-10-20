# language: en
Feature: Script-based tag naming
  Background:
    Given an isolated repository, global directory and Git tags
    And script-based release branches

  Scenario: Multi-branch release with fixed remote targets
    Given the naming script is
      """
      import json, sys, subprocess
      r = json.load(sys.stdin)
      assert set(r) == {'version','branch','target_sha','previous_tag','previous_sha','dry_run'}
      assert r['version'] == 1 and r['dry_run'] is False
      assert r['previous_tag'] == 'release-' + r['branch'] + '-a'
      assert subprocess.check_output(['git','show',r['target_sha']+':user.txt']).decode() == 'original'
      assert open('user.txt').read() == 'user edits'
      print(json.dumps({'version':1,'action':'tag','name':'release-'+r['branch']+'-next'}))
      """
    When running the CLI with "t main maintenance -p"
    Then custom tags are pushed at the remote commits

  Scenario: Preview executes from the worktree root
    Given the naming script is
      """
      import json, sys, os
      r = json.load(sys.stdin)
      assert r['dry_run'] and os.path.isfile('user.txt')
      print(json.dumps({'version':1,'action':'tag','name':'release-main-next'}))
      """
    And running from a repository subdirectory
    When running the CLI with "t main -n"
    Then the message contains "release-main-next"
    And no custom release tag exists

  Scenario: All branches may skip successfully
    Given the naming script is
      """
      import json, sys
      json.load(sys.stdin)
      print(json.dumps({'version':1,'action':'skip','reason':'Not scheduled'}))
      """
    When running the CLI with "t main maintenance"
    Then the message contains "Skipped: Not scheduled"
    And no custom release tag exists

  Scenario: Initial release receives null baseline fields
    Given no custom baseline tags exist
    And the naming script is
      """
      import json, sys
      r = json.load(sys.stdin)
      assert r['previous_tag'] is None and r['previous_sha'] is None
      print(json.dumps({'version':1,'action':'tag','name':'release-main-next'}))
      """
    When running the CLI with "t main -n"
    Then the message contains "release-main-next"

  Scenario Outline: Invalid responses never publish
    Given the naming script is
      """
      import json, sys
      json.load(sys.stdin)
      print('<response>')
      """
    When running the CLI with "t main -p"
    Then the error contains "<error>"
    And no custom release tag exists
    Examples:
      | response | error |
      | {"version":2,"action":"tag","name":"release-main-next"} | Unsupported script protocol |
      | {"version":1,"action":"tag","name":"release-main-next","name":"x"} | Invalid script protocol |
      | {"version":1,"action":"tag","name":"release-main-next","push":true} | Invalid script protocol |
      | log then JSON | Invalid script protocol |
      | {"version":1,"action":"tag","name":"other-next"} | must match the branch rule |
      | {"version":1,"action":"skip","reason":""} | Invalid script tag/name |

  Scenario: Nonzero exit overrides valid JSON
    Given the naming script is
      """
      import json, sys
      json.load(sys.stdin)
      print('{"version":1,"action":"tag","name":"release-main-next"}')
      sys.exit(7)
      """
    When running the CLI with "t main"
    Then the error contains "Script exited unsuccessfully"
    And no custom release tag exists

  Scenario: Timeout includes pipes held by descendants
    Given the naming script is
      """
      import json, sys, subprocess
      json.load(sys.stdin)
      subprocess.Popen(['sleep','30'])
      print('{"version":1,"action":"tag","name":"release-main-next"}')
      """
    When running the CLI with "t main"
    Then the error contains "Script timed out"
    And no custom release tag exists

  Scenario: Output limit stops the script
    Given the naming script is
      """
      import json, sys
      json.load(sys.stdin)
      print('x' * 65537)
      """
    When running the CLI with "t main"
    Then the error contains "stdout exceeds 64 KiB"
    And no custom release tag exists

  Scenario: Message selection uses ancestry without executing the script
    Given the naming script is
      """
      raise Exception('must not execute')
      """
    When running the CLI with "msg main"
    Then the message contains "release-main-a"
    And the message excludes "Pending release"

  Scenario: A second tag at the same commit is ambiguous
    Given tag "release-main-duplicate" points to "HEAD~2"
    When running the CLI with "msg main"
    Then the error contains "Ambiguous tag history"

  Scenario: No-change skips script execution
    Given tag "release-main-current" points to "HEAD~1"
    And the naming script is
      """
      raise Exception('must not execute')
      """
    When running the CLI with "t main"
    Then the message contains "No changes"

  Scenario: Rejected pushes retry the original name without a script
    Given the remote temporarily rejects pushes
    And the naming script is
      """
      import json, sys
      json.load(sys.stdin)
      print('{"version":1,"action":"tag","name":"release-main-next"}')
      """
    When running the CLI with "t main -p"
    Then the same custom tag can be pushed without running the script

  Scenario: Version is derived from the selected pattern branch
    Given the message shows the selected branch version
    When running the CLI with "msg main"
    Then the message contains "main|main-a|release-main-z"

  Scenario: Incomparable ancestors are rejected
    Given custom tag history forks before the target
    When running the CLI with "msg main"
    Then the error contains "divergent tag history"

  Scenario: Excluded script names cannot be published or used explicitly
    Given the next custom name is excluded
    And the naming script is
      """
      import json, sys
      json.load(sys.stdin)
      print('{"version":1,"action":"tag","name":"release-main-next"}')
      """
    When running the CLI with "t main"
    Then the error contains "must not be excluded"
    When running the CLI with "t main --name release-main-next"
    Then the error contains "must not be excluded"
    And no custom release tag exists

  Scenario: Chinese interface preserves script-selected names
    Given the naming script is
      """
      import json, sys
      json.load(sys.stdin)
      print('{"version":1,"action":"tag","name":"release-main-next"}')
      """
    When running the CLI with "t main -n --lang zh-CN"
    Then the message contains "release-main-next"
    And no custom release tag exists

  Scenario: A custom name cannot overwrite a different commit
    Given tag "release-main-next" points to "HEAD"
    And the naming script is
      """
      import json, sys
      json.load(sys.stdin)
      print('{"version":1,"action":"tag","name":"release-main-next"}')
      """
    When running the CLI with "t main -p"
    Then the error contains "points to another commit"

  Scenario: Shell completion does not invoke the naming script
    Given the naming script is
      """
      raise Exception('must not execute')
      """
    When running the CLI with "completion-branches"
    Then the message contains "main"
    And the message contains "maintenance"

  Scenario: Matching tags pointing to trees do not enter release history
    Given tag "release-main-tree" points to "HEAD^{tree}"
    When running the CLI with "msg main"
    Then the message contains "release-main-a"
