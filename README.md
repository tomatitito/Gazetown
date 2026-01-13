# Gazetown

Gas Town workspace and tooling experiments.

## Git Integration Test

Proof of concept: Verify git2 crate (used by Zed) works for Gas Town operations.

### Running the Test

```bash
cargo run -p gastown_git_test
```

### Test Scenarios

1. **Open existing rig repository** - Verify we can open and read repo state
2. **Create a worktree** - Simulate polecat spawn with isolated workspace
3. **Check worktree status** - Verify clean git state in worktree
4. **Make a commit** - Simulate beads sync operation
5. **Delete worktree** - Simulate polecat nuke cleanup

All scenarios test core operations needed for Gas Town's multi-agent git workflow.
