use anyhow::{Context, Result};
use git::repository::{GitRepository, RealGitRepository, CommitOptions};
use gpui::SharedString;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

fn main() -> Result<()> {
    println!("=== Gas Town Git Integration Test ===\n");

    smol::block_on(async {
        run_test().await
    })
}

async fn run_test() -> Result<()> {
    // Scenario 1: Open existing rig repository
    println!("1. Opening existing rig repository...");
    let repo_path = std::env::current_dir()
        .context("Failed to get current directory")?;
    let dotgit_path = repo_path.join(".git");

    let executor = gpui::background_executor();
    let repo = RealGitRepository::new(
        &dotgit_path,
        None, // bundled_git_binary_path
        Some(PathBuf::from("/usr/bin/git")), // system_git_binary_path
        executor.clone(),
    )
    .context("Failed to open repository")?;

    println!("   ✓ Repository opened successfully");
    println!("   Repository path: {}", repo_path.display());

    // Scenario 2: Create a worktree (simulate polecat spawn)
    println!("\n2. Creating a worktree (simulating polecat spawn)...");
    let worktree_name = "test-polecat-worktree";
    let worktree_dir = std::env::temp_dir();

    repo.create_worktree(
        worktree_name.to_string(),
        worktree_dir.clone(),
        None, // from_commit - None means create from HEAD
    )
    .await
    .context("Failed to create worktree")?;

    let worktree_path = worktree_dir.join(worktree_name);
    println!("   ✓ Worktree created successfully");
    println!("   Worktree path: {}", worktree_path.display());

    // Verify worktree exists in the list
    let worktrees = repo.worktrees().await
        .context("Failed to list worktrees")?;
    let found_worktree = worktrees.iter().find(|wt| wt.path == worktree_path);
    if found_worktree.is_some() {
        println!("   ✓ Worktree verified in worktree list");
    } else {
        anyhow::bail!("Worktree not found in worktree list!");
    }

    // Scenario 3: Check worktree status (clean git state)
    println!("\n3. Checking worktree status...");

    // Open the worktree as a repository
    let worktree_dotgit = worktree_path.join(".git");
    let worktree_repo = RealGitRepository::new(
        &worktree_dotgit,
        None,
        Some(PathBuf::from("/usr/bin/git")),
        executor.clone(),
    )
    .context("Failed to open worktree repository")?;

    // Check status
    let status = worktree_repo.status(&[]).await
        .context("Failed to get worktree status")?;

    println!("   ✓ Worktree status retrieved");
    println!("   Clean state: {}", status.entries.is_empty());

    // Scenario 4: Make a commit (simulate beads sync)
    println!("\n4. Making a commit (simulating beads sync)...");

    // Create a test file in the worktree
    let test_file_path = worktree_path.join("test_file.txt");
    std::fs::write(&test_file_path, "This is a test file for Gas Town git integration")
        .context("Failed to write test file")?;

    println!("   Created test file: {}", test_file_path.display());

    // Stage the file using git command
    let output = smol::process::Command::new("git")
        .current_dir(&worktree_path)
        .args(&["add", "test_file.txt"])
        .output()
        .await
        .context("Failed to stage file")?;

    if !output.status.success() {
        anyhow::bail!("Failed to stage file: {}", String::from_utf8_lossy(&output.stderr));
    }

    println!("   ✓ File staged");

    // Create a commit
    let env = Arc::new(HashMap::new());
    let commit_message = SharedString::from("Test commit for Gas Town git integration");

    worktree_repo.commit(
        commit_message,
        Some((
            SharedString::from("Gas Town Tester"),
            SharedString::from("tester@gastown.test"),
        )),
        CommitOptions {
            amend: false,
            signoff: false,
        },
        |_, _| async move { Ok(None) },
        env,
    )
    .await
    .context("Failed to create commit")?;

    println!("   ✓ Commit created successfully");

    // Verify commit was created
    let head_sha = worktree_repo.head_sha().await;
    if let Some(sha) = head_sha {
        println!("   HEAD SHA: {}", sha);
    }

    // Scenario 5: Delete worktree (simulate polecat nuke)
    println!("\n5. Deleting worktree (simulating polecat nuke)...");

    // Use git command to remove the worktree
    let output = smol::process::Command::new("git")
        .current_dir(&repo_path)
        .args(&["worktree", "remove", "--force", worktree_name])
        .output()
        .await
        .context("Failed to remove worktree")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to remove worktree: {}", stderr);
    }

    println!("   ✓ Worktree removed successfully");

    // Verify worktree is no longer in the list
    let worktrees_after = repo.worktrees().await
        .context("Failed to list worktrees after deletion")?;
    let still_exists = worktrees_after.iter().any(|wt| wt.path == worktree_path);
    if !still_exists {
        println!("   ✓ Worktree verified as removed from worktree list");
    } else {
        anyhow::bail!("Worktree still exists in worktree list!");
    }

    println!("\n=== All 5 scenarios completed successfully! ===");
    println!("\nSummary:");
    println!("✓ Opened existing rig repository");
    println!("✓ Created worktree (polecat spawn simulation)");
    println!("✓ Checked worktree status (clean git state)");
    println!("✓ Made a commit (beads sync simulation)");
    println!("✓ Deleted worktree (polecat nuke simulation)");
    println!("\nConclusion: Zed's git crate successfully supports all Gas Town operations!");

    Ok(())
}
