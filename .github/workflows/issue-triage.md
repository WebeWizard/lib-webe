---
on:
  issues:
    types: [opened]
  roles: all

permissions:
  contents: read
  issues: read
  pull-requests: read

engine: copilot

tools:
  github:
    mode: remote
    toolsets: [default]

network: defaults

safe-outputs:
  add-comment:
    max: 2
  add-labels:
    allowed: [bug, enhancement, question, duplicate]
    max: 3
  update-issue:
    max: 1
---

# Issue triage

Triage every newly opened issue in this repository.

## Goals

For each new issue, do all of the following in a single pass whenever possible:

1. classify the issue by **type**
2. estimate its **priority**
3. detect likely **duplicates**
4. ask concise **clarifying questions** if the report is incomplete or ambiguous
5. assign the issue to the most appropriate maintainer

## Repository-specific guidance

- This repository already has these useful labels available: `bug`, `enhancement`, `question`, and `duplicate`.
- There do **not** appear to be dedicated priority labels in the repository. You should still determine a recommended priority (`high`, `medium`, or `low`) and include it in your comment even when you cannot apply a priority label.
- This appears to be a very small repository. If you cannot identify a more specific maintainer from recent history or existing assignments, fall back to assigning the issue to the repository owner `${{ github.repository_owner }}`.

## Triage procedure

1. Read the issue title and body carefully.
2. Search existing open and closed issues for the same error, request, or symptom.
3. If a near-certain duplicate exists:
   - add the `duplicate` label
   - add a short comment linking the likely duplicate issue(s)
   - still add a type label if it is obvious from the report
4. Pick the best available **type** label:
   - `bug` for broken behavior, crashes, regressions, errors, or incorrect output
   - `enhancement` for feature requests, improvements, or missing capabilities
   - `question` when the issue is primarily support, clarification, or the report is too vague to classify confidently
5. Determine a recommended **priority**:
   - `high` for security issues, data loss, crashes, build breaks, or blockers
   - `medium` for clear defects or useful enhancements that are not blocking normal use
   - `low` for minor polish, nice-to-have requests, or issues needing more context before action
6. Decide whether the description is clear enough to act on:
   - if details are missing, ask up to 3 specific questions in one comment
   - keep questions concrete and easy to answer
7. Assign the issue:
   - prefer the maintainer who most likely owns the affected area based on repository history, previous related issues, or surrounding code ownership signals
   - if there is no clear owner, assign `${{ github.repository_owner }}`

## Commenting rules

- Always leave exactly one helpful triage comment unless the issue is obvious and fully classified without any useful extra context.
- The comment should summarize:
  - the chosen type
  - the recommended priority
  - any suspected duplicate(s)
  - any clarifying questions, if needed
  - who the issue was assigned to
- Be brief, friendly, and actionable.

## Safety rules

- Only use labels that already exist and are explicitly allowed by this workflow.
- Do not close issues automatically.
- Do not remove user content.
- If you are uncertain between labels, prefer `question`.
