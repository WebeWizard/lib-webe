---
on:
  schedule: daily # fuzzy schedule; compiler maps to a deterministic daily cron in the lock file
  workflow_dispatch:

permissions:
  contents: read
  issues: read
  pull-requests: write
  security-events: read

tools:
  github:
    toolsets: [default, dependabot]

network: defaults

safe-outputs:
  create-pull-request:
---

# Dependabot Triage Agent

Generate and maintain daily Dependabot triage artifacts for open Dependabot alerts.

## Required Behavior

1. Enumerate all **open Dependabot alerts** for the current repository.
2. For each alert, collect:
   - alert number/state/severity
   - dependency package name/ecosystem/manifest path/scope
   - dependency relationship (`direct` or `transitive`, when available)
   - advisory metadata (GHSA/CVE/summary/url)
3. Infer likely ownership:
   - if the vulnerable dependency is **transitive**, resolve **all top-level direct dependencies** in the manifest/dependency tree that introduce it
   - use the vulnerable package **and** all resolved top-level parent packages as search terms for usage/blame evidence
   - search tracked repository files for package usage mentions
   - for each usage line, use `git blame` to collect author name/email
   - choose the most frequent author as `recommended_user`
   - if no owner evidence exists, use `unknown` values
4. Write one JSON artifact per alert to `triage/dependabot/` using:
   - filename: `alert-<number>-<package-slug>.json`
   - stable, pretty JSON format with deterministic key ordering
5. Remove stale JSON files in `triage/dependabot/` that do not correspond to currently open alerts.

## Output Contract

Each JSON file must include:

- `alert_number`
- `state`
- `severity`
- `dependency` (`name`, `ecosystem`, `manifest_path`, `scope`)
- `advisory` (`ghsa_id`, `cve_id`, `summary`, `url`)
- `recommended_user` (`name`, `email`)
- `evidence` (`matching_usage_count`, `matching_usages`, `method`, `top_level_dependencies_considered`)
- `analyzed_at` (workflow run timestamp in ISO-8601 when available)

## Pull Request Rules

- If no triage output changes are needed, do **not** create a pull request.
- If files changed under `triage/dependabot/`, create a single pull request with:
  - title: `chore: update dependabot triage output`
  - concise body summarizing alert count and changed triage files
  - only the relevant triage artifact changes
