# Vertex Brain System — GitHub Publish Hotfix 000002

## Root cause

The first BrainSystem publisher correctly refused to overwrite a non-empty remote `main`.

The remote was inspected after the failure and is not real competing source history. It contains only the GitHub-created initial README commit:

- SHA: `a24d11cd557709411230e62a22edcf4ebfe1dd7c`
- message: `Initial commit`
- content: `# vertex_brain_system`

## Fix

This hotfix:
- locks the repository namespace to `ACE-Vertex/vertex_brain_system`,
- re-runs the source safety scan,
- preserves any local source commit created by the first publisher,
- replaces the trivial GitHub initial commit only through an **exact force-with-lease** pinned to its SHA,
- blocks if remote `main` changed after diagnosis,
- verifies local HEAD equals remote HEAD.

No unconditional force push is used.

Source publication remains separate from BrainSystem build/runtime verification.
