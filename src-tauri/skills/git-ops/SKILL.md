---
name: git-ops
description: Manage Git repositories within the workspace. Check status, commit changes, view diffs, and create branches.
license: MIT
---

# Git Ops Skill

This skill provides utilities for interacting with Git repositories directly from the workspace.

## Core Capabilities

1.  **Status**: Check repository state (modified files, staged changes).
2.  **Commit**: Create meaningful commits with messages.
3.  **Diff**: Inspect changes before committing.
4.  **Branch Management**: Create, list, and switch branches.
5.  **Log**: View commit history.

## Dependencies

*   `git` (command-line tool)
*   User must authorize git commands (standard Agent permission flow).

## Workflow Examples

### 1. Check Status

```bash
git status
```

### 2. Stage and Commit

```bash
# Add specific files
git add path/to/file.ext

# Or add all changes
git add .

# Commit with message
git commit -m "feat: implement user login"
```

### 3. View Changes

```bash
# Unstaged changes
git diff

# Staged changes
git diff --staged
```

### 4. Create and Checkout Branch

```bash
# Create new branch
git branch feature/new-api

# Switch to branch
git checkout feature/new-api

# Or create and switch in one command
git checkout -b feature/new-api
```

### 5. View Recent Commits

```bash
# Concise log
git log --oneline -n 5
```

## Best Practices

*   **Atomic Commits**: Keep changes small and focused.
*   **Clear Messages**: Use conventional commits format (feat:, fix:, docs:).
*   **Check Before Committing**: Always run `git status` or `git diff` first to avoid unintended inclusions.
*   **Branching**: Work in feature branches, not main/master directly.
