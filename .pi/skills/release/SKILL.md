---
name: release
description: Create a new release by analyzing conventional commits, bumping version, and pushing a tag to trigger the release workflow.
---

# Release Skill

Creates a new release for the rott project based on conventional commits.

## When to Use

- User says "create a release", "make a release", "release", "bump version"
- User wants to publish a new version

## Pre-flight Checks

Before proceeding, verify:

1. **No unpushed commits**: Run `git status` and check for commits ahead of origin
2. **Clean working directory**: No uncommitted changes
3. **On main branch**: Releases should come from main

If any check fails, inform the user and abort.

```bash
# Check for unpushed commits
git status -sb

# Check current branch
git branch --show-current
```

## Workflow

### 1. Get Latest Release Tag

```bash
git fetch --tags
git describe --tags --abbrev=0 2>/dev/null || echo "v0.0.0"
```

### 2. Get Commits Since Last Tag

```bash
# If there's a previous tag
git log <last-tag>..HEAD --oneline --pretty=format:"%s"

# If no previous tag (first release)
git log --oneline --pretty=format:"%s"
```

### 3. Analyze Commits for Version Bump

Parse conventional commits to determine version bump:

| Commit Type | Version Bump | Examples |
|-------------|--------------|----------|
| `feat!:` or `BREAKING CHANGE:` | MAJOR | `feat!: redesign API` |
| `feat:` | MINOR | `feat: add sync command` |
| `fix:`, `perf:`, `refactor:` | PATCH | `fix: handle empty input` |
| `docs:`, `chore:`, `ci:`, `test:` | No bump | `docs: update readme` |

Rules:
- If ANY commit has breaking change → MAJOR bump
- Else if ANY commit is `feat:` → MINOR bump  
- Else if ANY commit is `fix:`/`perf:` → PATCH bump
- Else → No release needed (only docs/chore commits)

### 4. Calculate New Version

Given current version `vX.Y.Z`:
- MAJOR: `v(X+1).0.0`
- MINOR: `vX.(Y+1).0`
- PATCH: `vX.Y.(Z+1)`

### 5. Present Options to User

Show the user:
1. Summary of commits since last release
2. Recommended version bump with reasoning
3. Options:
   - Accept recommended version
   - Choose different bump level (patch/minor/major)
   - Cancel

Example output:
```
Last release: v0.2.1
Commits since last release:
  - feat: add TUI interface
  - fix: handle empty links
  - docs: update README

Recommended: v0.3.0 (MINOR - new features added)

Options:
1. Create v0.3.0 (recommended)
2. Create v0.2.2 (patch)
3. Create v1.0.0 (major)
4. Cancel
```

### 6. Update Version in Cargo.toml

Update the workspace version in the root `Cargo.toml`:

```bash
# The version is in [workspace.package] section
sed -i 's/^version = ".*"/version = "X.Y.Z"/' Cargo.toml
```

Or use a more precise edit to update:
```toml
[workspace.package]
version = "X.Y.Z"  # Update this line
```

### 7. Commit Version Bump

```bash
git add Cargo.toml Cargo.lock
git commit -m "chore: bump version to vX.Y.Z"
```

### 8. Create Annotated Tag

```bash
git tag -a vX.Y.Z -m "Release vX.Y.Z"
```

### 9. Push Commit and Tag

```bash
git push origin main
git push origin vX.Y.Z
```

This triggers the release workflow in `.github/workflows/release.yml`.

### 10. Confirm Release

Provide the user with:
- Link to the GitHub Actions workflow
- Link to the releases page (once complete)

```
Release v0.3.0 initiated!

Watch the build: https://github.com/evcraddock/rott/actions
Releases: https://github.com/evcraddock/rott/releases
```

## Error Handling

- **Unpushed commits**: "You have unpushed commits. Please push or reset before releasing."
- **Uncommitted changes**: "Working directory not clean. Please commit or stash changes."
- **Not on main**: "Releases should be created from main branch. Currently on: <branch>"
- **No commits since last tag**: "No new commits since last release (vX.Y.Z)."
- **Tag already exists**: "Tag vX.Y.Z already exists. Choose a different version."

## Notes

- Always use annotated tags (`git tag -a`) for releases
- The release workflow handles building and publishing artifacts
- Version format follows semver: `vMAJOR.MINOR.PATCH`
- First release should be `v0.1.0` unless explicitly choosing `v1.0.0`
