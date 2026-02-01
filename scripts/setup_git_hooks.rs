use std::fs;
use std::path::Path;
use std::process;

fn main() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let hooks_source = repo_root.join("scripts").join("git-hooks");
    let hooks_target = repo_root.join(".git").join("hooks");

    if !hooks_target.exists() {
        eprintln!("Error: .git/hooks directory does not exist. Are you in a git repository?");
        process::exit(1);
    }

    println!("Setting up git hooks...");

    // Install pre-push hook
    let pre_push_source = hooks_source.join("pre-push");
    let pre_push_target = hooks_target.join("pre-push");

    if !pre_push_source.exists() {
        eprintln!("Error: pre-push hook not found at {:?}", pre_push_source);
        process::exit(1);
    }

    // Remove existing hook if it exists
    if pre_push_target.exists() {
        fs::remove_file(&pre_push_target).unwrap_or_else(|e| {
            eprintln!("Warning: Could not remove existing hook: {}", e);
        });
    }

    // Copy the hook (symlinks can be problematic on some systems)
    fs::copy(&pre_push_source, &pre_push_target).unwrap_or_else(|e| {
        eprintln!("Error: Could not install pre-push hook: {}", e);
        process::exit(1);
    });

    // Make it executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&pre_push_target).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&pre_push_target, perms).unwrap_or_else(|e| {
            eprintln!("Warning: Could not set executable permissions: {}", e);
        });
    }

    println!("✓ Git hooks installed successfully!");
    println!("  Pre-push hook: {:?}", pre_push_target);
}
