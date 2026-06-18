<!-- Template for delta behavior specs -->
<!-- Output location: changes/<name>/specs/<domain>/spec.md -->
<!-- <domain> = module name (saving, transfer, card, auth, etc.) -->

# Delta for [Domain Name]

<!-- First, check if openspec/specs/<domain>/spec.md exists.
     If yes, review existing requirements and only describe changes.
     If no, treat all requirements as ADDED. -->

## ADDED Requirements

<!-- New behavior introduced by this feature.
     Each requirement MUST have at least one Scenario. -->

### Requirement: [Requirement Name]
The system [MUST/SHALL/SHOULD] [behavior description].

#### Scenario: [Happy path scenario]
- GIVEN [initial context]
- WHEN [action occurs]
- THEN [expected outcome]
- AND [additional outcomes if any]

#### Scenario: [Error/edge case scenario]
- GIVEN [initial context]
- WHEN [error condition]
- THEN [error handling behavior]

## MODIFIED Requirements

<!-- Existing behavior that changes.
     State the new behavior and note what changed. -->

### Requirement: [Existing Requirement Name]
The system [MUST/SHALL/SHOULD] [updated behavior].
(Previously: [old behavior description])

#### Scenario: [Updated scenario]
- GIVEN [context]
- WHEN [action]
- THEN [new expected outcome]

## REMOVED Requirements

<!-- Deprecated behavior being removed.
     Explain why and what replaces it if applicable. -->

### Requirement: [Deprecated Requirement]
(Reason: [why this is being removed])
