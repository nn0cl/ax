## Summary

- 

## Phase

- [ ] Phase 0: Design Intake / collaboration scheme only
- [ ] Phase 1: Red - failing tests only
- [ ] Phase 2: Green - minimal implementation
- [ ] Phase 3: Refactor
- [ ] Documentation / ADR / project setup only

## Collaboration Checklist

- [ ] Local issue / GitHub issue / no-issue reason is identified.
- [ ] Issue dependencies are resolved, waived, or not applicable.
- [ ] Design intake / context ledger is included or not applicable.
- [ ] Adjudicator approval points are identified.
- [ ] Phase transition is explicit.
- [ ] Handoff notes are included when work is incomplete.
- [ ] AI payload includes/omits are documented when AI assistance shaped the change.
- [ ] AI work trace is linked or intentionally not applicable.
- [ ] Branch scope is one feature, process change, or reviewable unit.
- [ ] If this PR changes an agent operating contract file (see
      `docs/collaboration/prompt-instruction-change-control.md`), the reason
      is stated and a trace is linked.

## Architecture Checklist

- [ ] I kept business logic out of UI components, framework request/command
      handlers, and adapters.
- [ ] I used ports for external resources.
- [ ] I did not make optional external services required for core tests.
- [ ] I updated ADRs or ambiguity lists for new decisions.

## Readability Checklist

- [ ] Changed source files have clear single responsibilities.
- [ ] Functions are small enough to review without holding multiple business decisions in mind.
- [ ] The code is not compressed into clever or dense constructs just to be shorter.
- [ ] New abstractions are required by current tests or reduce real duplication.
- [ ] Tests are readable and focused on one behavior at a time.

## Verification

- [ ] CI passes
- [ ] Tests added or intentionally not applicable
- [ ] Reviewer empathy summary included when this is Phase 3
- [ ] Applicable Definition of Done is satisfied
