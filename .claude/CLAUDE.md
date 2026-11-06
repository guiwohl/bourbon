# Bourbon Codebase Rules

## Coding Principles

1. Apply the Clean Code and Clean Architecture methods from Uncle Bob, all the way. Always.
2. DO NOT WRITE COMMENTS, never, If the code is not readable by itself, and needs comments to explain it, it is a bad code. Also do NOT ADD "TODOs" (or similars) in the code. If something needs to be documented, document it on docs/, not on the code. Functions, variable, classes (and etc), should have nice names, so that it DOES NOT HAVE TO BE DISCOVERED what it does, only read.
3. Write decoupled code, always, think of easy to mantain code first when writing anything.
4. KISS, YAGNI, DRY, Should ALWAYS be putten in first place, they are helpers to a better code.
5. THINK and DEFIN
6. Simplicity First; Minimum code that solves the problem. Nothing speculative.
    No features beyond what was asked.
    Do abstractions only when you KNOW it could expand, if not known, ask.
    No "flexibility" or "configurability" that wasn't requested.
    No error handling for impossible scenarios.
Ask yourself: "What would a 20+ years of service senior would do to a better, cleaner, and mantainable code?".
7. Goal-Driven Execution. Define success criteria. Loop until verified.
Transform tasks into verifiable goals:
    "Add validation" → "Write tests for invalid inputs, then make them pass"
    "Fix the bug" → "Write a test that reproduces it, then make it pass"
    "Refactor X" → "Ensure tests pass before and after"
For multi-step tasks, state a brief plan:
    1. Step → verify: check
    2. Step → verify: check
    3. Step → verify: check
    4. ...
Strong success criteria let you loop independently. Weak criteria ("make it work") require constant clarification.

## Writing style

Write few words that can make the user understand, be simple, short, ULTRA SUMMARIZED, but do not miss details, never.

## Git

Work in git worktrees on ".claude/worktrees/*"
Use Conventional Commits, Conventional Branches Names, Conventional PRs Names.

## Documentation

When on doubt, searching for an answer, thinking about something, deciding something, or anything like this, ALWAYS look into the "docs/" folder on the repo, it may have your answers or tips in there.

When something is found, decided, explained, or anything that is considerable important to be documented, so that future users do not have to discover it again, DOCUMENT IT IN THERE.And add it to the index.

Also, update the documentation on each PR, this should be a step of the PR self-review. (almost/basically) NO PR COMES IN WITHOUT NEW DOCS OR DOCS UPDATES.

**USE THE DOCUMENTATION AT MAX.**
