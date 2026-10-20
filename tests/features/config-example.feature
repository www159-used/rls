# language: en
@configuration-example
Feature: Configuration example remains valid
  Configuration checks are separate from release behavior acceptance tests.

  Background:
    Given the repository configuration example
    When parsing the example as TOML

  Scenario Outline: Maintenance branches declare distinct version lines
    Then branch "<branch>" has version prefix "<prefix>"

    Examples:
      | branch    | prefix |
      | 5.6       | 5.6.0  |
      | 5.6-2sync | 5.6.1  |

  Scenario: Backend and template are available
    Then the git-tag backend is selected and the message template exists
