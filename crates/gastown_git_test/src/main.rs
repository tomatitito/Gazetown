use anyhow::{Context, Result};
use git2::{Repository, Signature};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn main() -> Result<()> {
    println!("🧪 Gas Town Git Integration Test");
    println!("Testing git2 operations for polecat lifecycle\n");

    // Create a temporary directory for our test rig
    let temp_dir = TempDir::new().context("Failed to create temp directory")?;
    let rig_path = temp_dir.path().join("test-rig");

    println!("📁 Test rig location: {}", rig_path.display());

    // Initialize test rig repository
    setup_test_rig(&rig_path)?;

    // Run the 5 test scenarios
    println!("\n{}", "=".repeat(60));
    test_scenario_1_open_repository(&rig_path)?;

    println!("\n{}", "=".repeat(60));
    test_scenario_2_create_worktree(&rig_path)?;

    println!("\n{}", "=".repeat(60));
    test_scenario_3_check_worktree_status(&rig_path)?;

    println!("\n{}", "=".repeat(60));
    test_scenario_4_make_commit(&rig_path)?;

    println!("\n{}", "=".repeat(60));
    test_scenario_5_delete_worktree(&rig_path)?;

    println!("\n{}", "=".repeat(60));
    println!("\n✅ All 5 scenarios passed!");
    println!("🎯 git2 crate is suitable for Gas Town operations");

    Ok(())
}

/// Setup: Create a test rig repository with initial commit
fn setup_test_rig(path: &Path) -> Result<()> {
    println!("🔧 Setting up test rig repository...");

    // Initialize repository
    let repo = Repository::init(path).context("Failed to initialize repository")?;

    // Create initial file
    let readme_path = path.join("README.md");
    fs::write(&readme_path, "# Test Rig\n").context("Failed to write README")?;

    // Stage and commit
    let mut index = repo.index().context("Failed to get index")?;
    index.add_path(Path::new("README.md")).context("Failed to stage file")?;
    index.write().context("Failed to write index")?;

    let tree_id = index.write_tree().context("Failed to write tree")?;
    let tree = repo.find_tree(tree_id).context("Failed to find tree")?;

    let sig = Signature::now("Test User", "test@example.com")
        .context("Failed to create signature")?;

    repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        "Initial commit",
        &tree,
        &[],
    ).context("Failed to create initial commit")?;

    println!("  ✓ Created repository with initial commit");
    Ok(())
}

/// Scenario 1: Open existing rig repository
fn test_scenario_1_open_repository(rig_path: &Path) -> Result<()> {
    println!("📖 Scenario 1: Open existing rig repository");

    let repo = Repository::open(rig_path)
        .context("Failed to open repository")?;

    let head = repo.head().context("Failed to get HEAD")?;
    let commit = head.peel_to_commit().context("Failed to get commit")?;

    println!("  ✓ Opened repository at: {}", rig_path.display());
    println!("  ✓ HEAD commit: {}", commit.id());
    println!("  ✓ Commit message: {}", commit.message().unwrap_or("(no message)"));

    Ok(())
}

/// Scenario 2: Create a worktree (simulate polecat spawn)
fn test_scenario_2_create_worktree(rig_path: &Path) -> Result<()> {
    println!("🌲 Scenario 2: Create a worktree (simulate polecat spawn)");

    let repo = Repository::open(rig_path)?;

    // Create worktree path
    let worktree_path = rig_path.parent().unwrap().join("polecat-worker-1");

    // Create the worktree using git2
    // Note: git2 requires the directory to not exist
    let worktree = repo.worktree(
        "polecat-worker-1",
        &worktree_path,
        None,
    ).context("Failed to create worktree")?;

    println!("  ✓ Created worktree: {}", worktree.name().unwrap_or("(unnamed)"));
    println!("  ✓ Worktree path: {}", worktree_path.display());
    println!("  ✓ Is valid: {}", worktree.validate().is_ok());

    Ok(())
}

/// Scenario 3: Check worktree status (clean git state)
fn test_scenario_3_check_worktree_status(rig_path: &Path) -> Result<()> {
    println!("📊 Scenario 3: Check worktree status (clean git state)");

    let worktree_path = rig_path.parent().unwrap().join("polecat-worker-1");
    let repo = Repository::open(&worktree_path)
        .context("Failed to open worktree repository")?;

    // Check status
    let statuses = repo.statuses(None)
        .context("Failed to get status")?;

    println!("  ✓ Opened worktree repository");
    println!("  ✓ Status entries: {}", statuses.len());
    println!("  ✓ Working tree is clean: {}", statuses.is_empty());

    // Verify HEAD
    let head = repo.head().context("Failed to get HEAD")?;
    println!("  ✓ HEAD reference: {}", head.name().unwrap_or("(detached)"));

    Ok(())
}

/// Scenario 4: Make a commit (simulate beads sync)
fn test_scenario_4_make_commit(rig_path: &Path) -> Result<()> {
    println!("💾 Scenario 4: Make a commit (simulate beads sync)");

    let worktree_path = rig_path.parent().unwrap().join("polecat-worker-1");
    let repo = Repository::open(&worktree_path)?;

    // Create a new file (simulating beads changes)
    let beads_file = worktree_path.join("beads-update.txt");
    fs::write(&beads_file, "Beads sync: Updated issue status\n")
        .context("Failed to write beads file")?;

    // Stage the file
    let mut index = repo.index()?;
    index.add_path(Path::new("beads-update.txt"))?;
    index.write()?;

    // Create commit
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;
    let sig = Signature::now("Polecat Worker", "polecat@gastown")
        .context("Failed to create signature")?;

    let parent_commit = repo.head()?.peel_to_commit()?;
    let commit_id = repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        "Beads sync: Update issue status",
        &tree,
        &[&parent_commit],
    )?;

    println!("  ✓ Created new file: beads-update.txt");
    println!("  ✓ Staged and committed changes");
    println!("  ✓ Commit ID: {}", commit_id);

    // Verify the commit
    let new_commit = repo.find_commit(commit_id)?;
    println!("  ✓ Commit message: {}", new_commit.message().unwrap_or("(no message)"));

    Ok(())
}

/// Scenario 5: Delete worktree (simulate polecat nuke)
fn test_scenario_5_delete_worktree(rig_path: &Path) -> Result<()> {
    println!("🗑️  Scenario 5: Delete worktree (simulate polecat nuke)");

    let repo = Repository::open(rig_path)?;
    let worktree_path = rig_path.parent().unwrap().join("polecat-worker-1");

    // Find the worktree
    let worktree = repo.find_worktree("polecat-worker-1")
        .context("Failed to find worktree")?;

    println!("  ✓ Found worktree: {}", worktree.name().unwrap_or("(unnamed)"));

    // First, remove the directory (this is what a polecat nuke does)
    if worktree_path.exists() {
        fs::remove_dir_all(&worktree_path)
            .context("Failed to remove worktree directory")?;
        println!("  ✓ Removed worktree directory");
    }

    // Then prune the worktree from git's tracking (now that it's gone)
    worktree.prune(None)
        .context("Failed to prune worktree")?;

    println!("  ✓ Pruned worktree from git tracking");

    // Verify worktree is gone
    let worktrees = repo.worktrees()
        .context("Failed to list worktrees")?;
    println!("  ✓ Remaining worktrees: {}", worktrees.len());

    Ok(())
}
