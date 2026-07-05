---
name: merge-pr
description: Merge a GitHub PR by looping until it's green and mergeable — waits for CI, fixes failures with the Claude SDK, addresses CodeRabbit review comments, resolves conflicts, then merges. Use when the user asks to merge a PR, land a branch, get a PR green, or clear CI/review comments. Wraps `claudetm merge-pr`.
---

# merge-pr

Land a PR the hands-off way: `claudetm merge-pr` monitors the PR, fixes CI failures and review comments (CodeRabbit included) with the Claude SDK, resolves merge conflicts, and merges once everything is green — looping until mergeable. This is how [big fat PRs](../../../docs/mvp/v1/01-workflow-and-parallelization.md#pr-conventions) land without a human bottleneck.

## When to use
- "merge this PR" / "land my branch" / "get PR 52 green" / "handle the CodeRabbit comments and merge".
- After opening a fat, feature-complete PR for a plan batch ([workflow](../../../docs/mvp/v1/01-workflow-and-parallelization.md)).

## How to run
Run in the repo. It uses the current branch's PR unless you pass a number/URL.

```bash
claudetm merge-pr                 # merge the PR for the current branch
claudetm merge-pr 52              # merge PR #52
claudetm merge-pr https://github.com/owner/repo/pull/52
claudetm merge-pr 52 -m 5         # cap at 5 fix iterations (default 30)
claudetm merge-pr 52 --no-merge   # fix + make ready, but don't merge
```

Prefer running it in the background (long-running loop) and reporting the outcome:

```bash
claudetm merge-pr 52
```

## What it does each iteration
1. Wait for CI checks to report.
2. If CI is red → diagnose and fix with the Claude SDK, push.
3. If there are review comments (CodeRabbit / reviewers) → address them inline, push.
4. If the branch conflicts with base → resolve, push.
5. Re-check. Merge when green; otherwise loop (until `--max-iterations`).

## Options
- `-m, --max-iterations N` — max fix iterations before giving up (default 30).
- `--no-merge` — fix and make mergeable, but stop short of merging (use when a human wants the final click).

## Notes & guardrails
- Needs `gh` auth and push rights to the PR branch.
- It **pushes commits and merges** — an outward, hard-to-reverse action. Confirm the target PR with the user first unless they've clearly authorized this run.
- If it exhausts iterations without going green, report the last failing check/comment to the user rather than force-merging.
- Check availability with `claudetm merge-pr --help`.
