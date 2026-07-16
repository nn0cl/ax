# Feature: <feature name>

## EARS

When <trigger>, the system shall <expected result>.

If <optional condition>, the system shall <expected result>.

While <state>, the system shall <expected result>.

## Gherkin

```gherkin
Scenario: <observable scenario name>
  Given <precondition>
  And <precondition>
  When <trigger>
  Then <observable result>
  And <observable result>
```

## External Dependencies

- <dependency> via <port>

## Out of Scope

- <behavior that must not be implemented in this feature>

## Ambiguities

- <decision that needs Adjudicator review or ADR>
