---
trigger: always_on
description: "Safe refactoring rules and code smell detection. Enforces test-first approach, incremental changes, and behavior preservation."
---

# Refactoring Rules

> Rules for safely improving code quality without changing behavior. For detailed refactoring patterns, refer to `skills/clean-code/` and `skills/code-refactoring-refactor-clean/`.

## Principles

- Understand before changing
- Never refactor without tests
- One small change at a time
- Behavior must be preserved
- Refactoring and feature work are separate commits

## Rules

### Before Refactoring

- Document what the code does and why it was written this way
- Identify all inputs, outputs, and side effects
- Verify test coverage exists; write tests first if missing
- Identify all dependents of the code being changed

### Code Smells to Address

| Smell | Threshold | Solution |
|-------|-----------|----------|
| Long methods | > 20 lines | Extract smaller functions |
| Large classes | Violates SRP | Split into focused classes |
| Duplicate code | Same logic in 2+ places | Extract common code |
| Long parameter lists | > 3–4 parameters | Introduce parameter objects |
| Feature envy | Method uses another class more | Move method to correct class |
| Primitive obsession | Strings/numbers for domain concepts | Create domain objects |
| Nested conditionals | Deep if/else nesting | Guard clauses, polymorphism |
| Dead code | Unused vars, functions, imports | Remove immediately |

### Safe Process

1. Ensure all tests pass **before** starting
2. Make **one change** at a time
3. Run tests **after each change**
4. Commit **after each successful step**
5. Update documentation if interfaces changed
6. Verify code is clearer/simpler than before
7. Confirm **no behavior change** occurred

### Common Refactoring Patterns

- **Extract Function**: Pull cohesive code into named function
- **Inline Function**: Remove unnecessary indirection
- **Extract Variable**: Name complex expressions for clarity
- **Rename**: Improve naming to reveal intent
- **Move Function**: Relocate code to where it belongs
- **Replace Conditional with Polymorphism**: Eliminate type-switching
- **Introduce Parameter Object**: Group related parameters
- **Replace Magic Number with Constant**: Named constants for clarity
- **Decompose Conditional**: Break complex conditions into named booleans

### When NOT to Refactor

- No tests exist and no time to write them
- Deadline pressure (risk of introducing bugs)
- Code is about to be replaced
- You don't understand what the code does
- Code works and nobody needs to change it

## Anti-Patterns

- ❌ Refactoring and adding features in the same commit
- ❌ Large-scale refactoring without tests
- ❌ Refactoring code you don't understand
- ❌ Ignoring performance implications of structural changes
- ❌ Not documenting why the refactoring was done

## Checklist

- [ ] Do I understand what this code does?
- [ ] Are there tests covering this code?
- [ ] Are all tests passing before I start?
- [ ] Am I making one small change at a time?
- [ ] Are tests still passing after each change?
- [ ] Did I update documentation if needed?
- [ ] Is the code clearer/simpler than before?
- [ ] Did I NOT change the behavior?

## References

- Clean code: [skills/clean-code/](../skills/clean-code/)
- Code refactoring: [skills/code-refactoring-refactor-clean/](../skills/code-refactoring-refactor-clean/)