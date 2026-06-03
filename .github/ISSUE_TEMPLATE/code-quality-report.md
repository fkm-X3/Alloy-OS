---
title: Code Quality Report
labels:
  - automated
  - code-quality
---

## Code Quality Report

**Generated:** <%= new Date().toUTCString() %>
**Commit:** [`<%= process.env.COMMIT %>`](<%= 'https://github.com/' + (process.env.GITHUB_REPOSITORY || 'unknown') + '/commit/' + process.env.COMMIT %>)
**Workflow:** [<%= process.env.WORKFLOW %> #<%= process.env.RUN_ID %>](<%= 'https://github.com/' + (process.env.GITHUB_REPOSITORY || 'unknown') + '/actions/runs/' + process.env.RUN_ID %>)

### Summary

| Category | Count | Status |
|----------|-------|--------|
| 🔴 Clippy Errors | **<%= process.env.CLIPPY_ERRORS %>** | <%= parseInt(process.env.CLIPPY_ERRORS) > 0 ? 'Needs attention' : 'Clean' %> |
| 🟡 Clippy Warnings | **<%= process.env.CLIPPY_WARNINGS %>** | <%= parseInt(process.env.CLIPPY_WARNINGS) > 0 ? 'Needs attention' : 'Clean' %> |
| 📝 TODO Comments | <%= process.env.TODO_COUNT %> | — |
| ⚠️ FIXME Comments | <%= process.env.FIXME_COUNT %> | <%= parseInt(process.env.FIXME_COUNT) > 10 ? 'High' : 'OK' %> |
| 🔧 HACK/XXX Comments | <%= process.env.HACK_COUNT %> | — |
| 🔍 NULL/nullptr in C | <%= process.env.NULL_COUNT %> | — |
| 🦀 `unsafe` blocks (lib.rs) | <%= process.env.UNSAFE_COUNT %> | — |

### Priority Items

<% if (parseInt(process.env.CLIPPY_ERRORS) > 0) { %>
**🔴 Clippy Errors:** <%= process.env.CLIPPY_ERRORS %> error(s) found.
Run `cd kernel/rust && cargo clippy --target i686-alloy.json -Zbuild-std=core,alloc` to inspect.
<% } %>

<% if (parseInt(process.env.CLIPPY_WARNINGS) > 0) { %>
**🟡 Clippy Warnings:** <%= process.env.CLIPPY_WARNINGS %> warning(s) found.
Run `cd kernel/rust && cargo clippy --target i686-alloy.json -Zbuild-std=core,alloc -- --deny warnings` to review.
<% } %>

<% if (parseInt(process.env.FIXME_COUNT) > 10) { %>
**⚠️ High FIXME Count:** <%= process.env.FIXME_COUNT %> FIXME comments. Consider addressing or triaging them.
<% } %>

### Notes

- This issue is auto-generated and updated by the `code-quality-issue-tracker.yml` workflow.
- To fix the reported issues, check the linked workflow run for detailed logs.
