# Agent Instructions

This project uses **bd** (beads) for issue tracking. Run `bd onboard` to get started.

## Development Commands

### Formatting

```bash
cargo fmt --all              # Format all code
cargo fmt --all -- --check   # Check formatting without modifying files
```

### Linting

```bash
./script/clippy              # Run clippy (use this instead of cargo clippy)
```

### Testing

```bash
cargo test                   # Run all tests
cargo test -p <crate>        # Run tests for a specific crate
```

### CI Checks

Before pushing, run these checks locally to catch CI failures:

```bash
cargo fmt --all -- --check   # Check formatting
cargo machete                # Check for unused dependencies (requires cargo-machete)
./script/check-licenses      # Verify license files exist in all crates
cargo xtask workflows        # Regenerate GitHub workflow files after editing xtask
```

## Quick Reference

```bash
bd ready              # Find available work
bd show <id>          # View issue details
bd update <id> --status in_progress  # Claim work
bd close <id>         # Complete work
bd sync               # Sync with git
```

## Landing the Plane (Session Completion)

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY:
   ```bash
   git pull --rebase
   bd sync
   git push
   git status  # MUST show "up to date with origin"
   ```
5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**
- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing - that leaves work stranded locally
- NEVER say "ready to push when you are" - YOU must push
- If push fails, resolve and retry until it succeeds

