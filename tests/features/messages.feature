# language: en
Feature: Release messages and multi-branch tags
  Background:
    Given an isolated repository, global directory and Git tags

  Scenario: Default message template
    When generating the message
    Then the message contains all commits in order with preserved newlines
    And the worktree and Git refs are unchanged and no records are created

  Scenario: Reject commands outside a repository
    Given running outside the repository
    When generating the message
    Then the error contains "target Git repository"

  Scenario: Read Git history from subdirectories
    Given running from a repository subdirectory
    When generating the message
    Then the message contains all commits in order with preserved newlines

  Scenario: File output preserves newlines
    When writing the message to a file
    Then the worktree and Git refs are unchanged and no records are created

  Scenario: Inherit and override global templates and variables
    Given the global configuration is
      """
      schema_version = 1
      [message]
      template = "templates/message.j2"
      [message.variables]
      greeting = "Global greeting"
      suffix = "Thanks"
      """
    And the global template is
      """
      {{ variables.greeting }}|{{ variables.suffix }}|{{ releases[0].tag }}
      """
    When generating the message
    Then the message contains "Global greeting|Thanks|v1.0.1"
    Given the repository configuration is
      """
      schema_version = 1
      [message.variables]
      greeting = "Repository greeting"
      releases = "Cannot override facts"
      """
    When generating the message
    Then the message contains "Repository greeting|Thanks|v1.0.1"
    And the worktree and Git refs are unchanged and no records are created
    Given the repository configuration is
      """
      schema_version = 1
      [message.variables]
      greeting = ""
      """
    When generating the message
    Then the message contains "|Thanks|v1.0.1"

  Scenario: Shared configuration and relative templates in worktrees
    Given the global configuration is
      """
      schema_version = 1
      [message.variables]
      greeting = "Inherited variable"
      """
    And the repository configuration is
      """
      schema_version = 1
      [message]
      template = "templates/message.j2"
      """
    And the repository template is
      """
      Repository template|{{ variables.greeting }}|{{ releases[0].tag }}
      """
    And running from a linked worktree
    When generating the message
    Then the message contains "Repository template|Inherited variable|v1.0.1"

  Scenario: Repository template overrides global template
    Given the global configuration is
      """
      schema_version = 1
      [message]
      template = "templates/message.j2"
      """
    And the global template is
      """
      Global template
      """
    And the repository configuration is
      """
      schema_version = 1
      [message]
      template = "templates/message.j2"
      """
    And the repository template is
      """
      Repository template
      """
    When generating the message
    Then the message contains "Repository template"

  Scenario: Missing template reports an error
    Given the repository configuration is
      """
      schema_version = 1
      [message]
      template = "missing.j2"
      """
    When generating the message
    Then the error contains "missing.j2"

  Scenario: Undefined template variables fail
    Given the repository configuration is
      """
      schema_version = 1
      [message]
      template = "templates/message.j2"
      """
    And the repository template is
      """
      {{ variables.typo }}
      """
    When generating the message
    Then the error contains "undefined"

  Scenario: Invalid template syntax fails
    Given the repository configuration is
      """
      schema_version = 1
      [message]
      template = "templates/message.j2"
      """
    And the repository template is
      """
      {% broken %}
      """
    When generating the message
    Then the error contains "Parse message template"

  Scenario Outline: Invalid configuration cannot silently fall back
    Given the repository configuration is
      """
      <config>
      """
    When generating the message
    Then the error contains "<error>"

    Examples:
      | config             | error          |
      | schema_version = 9 | schema_version |
      | invalid = [        | Parse configuration       |

  Scenario: Invalid global configuration fails
    Given the global configuration is
      """
      schema_version = 99
      """
    When generating the message
    Then the error contains "schema_version 99"

  Scenario: Absolute template paths are preserved
    Given the repository template uses an absolute path
    When generating the message
    Then the message contains "Absolute template|v1.0.1"

  Scenario: Merge explicit and legacy message versions
    Given the global configuration is
      """
      schema_version = 1
      [message]
      version = 1
      template = "templates/message.j2"
      [message.variables]
      greeting = "global"
      """
    And the global template is
      """
      {{ variables.greeting }}|{{ releases[0].tag }}
      """
    And the repository configuration is
      """
      schema_version = 1
      [message.variables]
      greeting = "Legacy repository config"
      """
    When generating the message
    Then the message contains "Legacy repository config|v1.0.1"

  Scenario Outline: Validate message versions independently in each layer
    Given the global configuration is
      """
      schema_version = 1
      [message]
      version = <global_version>
      """
    And the repository configuration is
      """
      schema_version = 1
      [message]
      version = <project_version>
      """
    When generating the message
    Then the error contains "unsupported message.version <invalid_version>"

    Examples:
      | global_version | project_version | invalid_version |
      | 2              | 1               | 2               |
      | 1              | 2               | 2               |
      | 1              | 0               | 0               |

  Scenario: Message version must be an integer
    Given the repository configuration is
      """
      schema_version = 1
      [message]
      version = "1"
      """
    When generating the message
    Then the error contains "Parse configuration"

  Scenario: Aliases and default current branch
    When running the CLI with "m"
    Then the message contains all commits in order with preserved newlines
    And the worktree and Git refs are unchanged and no records are created

  Scenario: Reject tag and range arguments
    When running the CLI with "msg v1.0.0..v1.0.1"
    Then the error contains "Git check-ref-format"

  Scenario: Reject the old from option
    When running the CLI with "msg main --from v1.0.0"
    Then the error contains "--from"

  Scenario: Reject ambiguous numeric tags
    Given tag "v1.0.00" points to "HEAD~2"
    When running the CLI with "m main"
    Then the error contains "Ambiguous tag history"

  Scenario: Reject non-monotonic history
    Given tag "v1.0.9" points to "HEAD~1"
    When running the CLI with "m main"
    Then the error contains "Non-monotonic"

  Scenario: Exclude an explicitly configured legacy tag from messages
    Given tag "v1.0.1007" points to "HEAD~1"
    And the repository configuration is
      """
      schema_version = 1
      [branches.main]
      version_prefix = "1.0"
      tag_prefix = "v"
      exclude_tags = ["v1.0.1007"]
      """
    When running the CLI with "m main"
    Then the message contains all commits in order with preserved newlines
    And the message excludes "v1.0.1007"

  Scenario: Exclude a legacy tag when creating and pushing the next release
    Given a local bare remote and two release branches exist
    And tag "v1.0.1007" points to "HEAD~2"
    And the repository configuration is
      """
      schema_version = 1
      [branches.main]
      version_prefix = "1.0"
      tag_prefix = "v"
      exclude_tags = ["v1.0.1007"]
      [branches.maintenance]
      version_prefix = "2.0"
      tag_prefix = "v"
      """
    When running the CLI with "t main maintenance -p"
    Then both new tags point to remote commits and are pushed without changing the worktree

  Scenario: Explicit names cannot reuse an excluded legacy tag
    Given a local bare remote and two release branches exist
    And the repository configuration is
      """
      schema_version = 1
      [branches.main]
      version_prefix = "1.0"
      tag_prefix = "v"
      exclude_tags = ["v1.0.1007"]
      """
    When running the CLI with "t main --name v1.0.1007 -n"
    Then the error contains "--name"

  Scenario: Reject shallow history
    Given running from a depth-one shallow clone
    When running the CLI with "m main"
    Then the error contains "shallow clone"

  Scenario: Do not output partial messages
    When running the CLI with "m main missing"
    Then the error contains "Configure branches.missing"

  Scenario: Multi-branch messages exclude untagged commits
    Given a local bare remote and two release branches exist
    When running the CLI with "msg maintenance main"
    Then the message contains "v2.0.1"
    And the message contains "v1.0.1"
    And the message excludes "Pending release"
    And the message excludes "Local only"

  Scenario: Preview does not create or push release tags
    Given a local bare remote and two release branches exist
    When running the CLI with "t main maintenance -n -p"
    Then no new tags are created

  Scenario: Create and push tags at remote branch commits
    Given a local bare remote and two release branches exist
    When running the CLI with "tag main maintenance -p"
    Then both new tags point to remote commits and are pushed without changing the worktree

  Scenario: Create local tags by default and retry pushing
    Given a local bare remote and two release branches exist
    When running the CLI with "t main"
    Then only a local tag is created and pushing the same name can be retried

  Scenario: Never overwrite a conflicting tag
    Given a local bare remote and two release branches exist
    And tag "v1.0.2" points to "HEAD"
    When running the CLI with "t main"
    Then the error contains "points to another commit"

  Scenario: Retain successful branches after partial failure
    Given a local bare remote and two release branches exist
    When running the CLI with "t missing main"
    Then the valid branch succeeds and the batch fails while retaining results

  Scenario: Retry the same tag after push rejection
    Given a local bare remote and two release branches exist
    And the remote temporarily rejects pushes
    When running the CLI with "t main -p"
    Then the rejected push retains its tag and can be retried after recovery

  Scenario: Manual increment requires an explicit name
    Given a local bare remote and two release branches exist
    And main uses manual increment mode
    When running the CLI with "t main"
    Then the error contains "Manual mode requires --name"
    When running the CLI with "t main --name v1.0.7 -n"
    Then the message contains "v1.0.7"

  Scenario: Increment numerically rather than lexically
    Given a local bare remote and two release branches exist
    And tag "v1.0.9" points to "HEAD~2"
    When running the CLI with "t main -n"
    Then the message contains "v1.0.10"

  Scenario: Do not create empty releases
    Given a local bare remote and two release branches exist
    And tag "v1.0.2" points to "HEAD~1"
    When running the CLI with "t main"
    Then the message contains "No changes"

  Scenario: Detached HEAD requires explicit branches
    Given running from a linked worktree
    When running the CLI with "m"
    Then the error contains "Detached HEAD"

  Scenario: Reject duplicate branches
    When running the CLI with "t main main"
    Then the error contains "Duplicate branch"
