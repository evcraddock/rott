---
description: Create a new release by analyzing conventional commits and pushing a tag
---

Create a release for rott based on conventional commits.

## Pre-flight Checks

Before proceeding, verify:
1. No unpushed commits (`git status -sb`)
2. Clean working directory
3. On main branch (`git branch --show-current`)

If any check fails, inform me and abort.

## Workflow

1. **Get latest tag**: `git fetch --tags && git describe --tags --abbrev=0 2>/dev/null || echo "v0.0.0"`

2. **Get commits since last tag**: `git log <last-tag>..HEAD --oneline --pretty=format:"%s"`

3. **Analyze commits for version bump**:
   - `feat!:` or `BREAKING CHANGE:` → MAJOR
   - `feat:` → MINOR
   - `fix:`, `perf:`, `refactor:` → PATCH
   - `docs:`, `chore:`, `ci:`, `test:` → No bump

4. **Present options**: Show commits, recommended version, and let me choose

5. **Update version**: Edit `[workspace.package] version` in root `Cargo.toml`

6. **Commit**: `git add Cargo.toml Cargo.lock && git commit -m "chore: bump version to vX.Y.Z"`

7. **Tag**: `git tag -a vX.Y.Z -m "Release vX.Y.Z"`

8. **Push**: `git push origin main && git push origin vX.Y.Z`

9. **Confirm**: Provide links to GitHub Actions and releases page
