use anyhow::{Context, Result};
use clap::Parser;
use murasaki_rs::git::get_repository_status;
use murasaki_rs::{
    check_for_updates, detect_git_operation, find_conflicted_files, parse_conflicts, run_app,
    AppState, UpdateInfo,
};

#[derive(Parser, Debug)]
#[command(name = "saki")]
#[command(about = "A TUI tool for handling git conflicts during merge or rebase", long_about = None)]
struct Args {
    /// Specific files to resolve (if not provided, all conflicted files will be shown)
    #[arg(value_name = "FILES")]
    files: Vec<String>,
}

fn print_update_notification(update_info: &UpdateInfo) {
    let current = &update_info.current_version;
    let latest = &update_info.latest_version;

    eprintln!("╭─────────────────────────────────────────────────────────╮");
    eprintln!("│  A new version of saki is available!                    │");
    eprintln!(
        "│  Current: {:<10} →  Latest: {:<10}              │",
        current, latest
    );
    eprintln!("│                                                         │");
    eprintln!("│  Run: cargo install murasaki_rs                         │");
    eprintln!("╰─────────────────────────────────────────────────────────╯");
    eprintln!();
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Check for updates at startup (non-blocking with 3s timeout)
    if let Some(update_info) = check_for_updates() {
        print_update_notification(&update_info);
    }

    // Open git repository
    let repo =
        murasaki_rs::git::detector::open_repository().context("Failed to open git repository")?;

    // Check if there's an ongoing git operation (merge or rebase)
    let git_operation = detect_git_operation(&repo);

    // If there's a git operation, handle conflicts
    if let Ok(operation) = git_operation {
        // Find conflicted files with validation
        let conflicted_paths = if args.files.is_empty() {
            find_conflicted_files(&repo).context("Failed to find conflicted files")?
        } else {
            // Validate user-provided file paths
            let workdir = repo
                .workdir()
                .context("Repository has no working directory")?;

            args.files
                .into_iter()
                .map(|file_str| {
                    let path = std::path::PathBuf::from(&file_str);

                    // Canonicalize to resolve symlinks and .. components
                    let canonical_path = path
                        .canonicalize()
                        .with_context(|| format!("File not found: {}", file_str))?;

                    // Ensure the file is within the repository
                    if !canonical_path.starts_with(workdir) {
                        anyhow::bail!(
                            "File {} is outside repository: {}",
                            file_str,
                            canonical_path.display()
                        );
                    }

                    // Ensure it's a file, not a directory
                    if !canonical_path.is_file() {
                        anyhow::bail!("{} is not a file", file_str);
                    }

                    Ok(canonical_path)
                })
                .collect::<Result<Vec<_>>>()?
        };

        // Parse conflicts from each file
        let mut conflicted_files = Vec::new();
        for path in conflicted_paths {
            match parse_conflicts(&path) {
                Ok(file) => conflicted_files.push(file),
                Err(e) => {
                    eprintln!("Warning: Failed to parse {}: {}", path.display(), e);
                }
            }
        }

        if conflicted_files.is_empty() {
            println!("No conflicted files found!");
            return Ok(());
        }

        // Create app state in conflict mode and run
        let state = AppState::new(conflicted_files, operation);
        run_app(state)?;
    } else {
        // No git operation in progress, run in staging mode
        let file_statuses = get_repository_status(&repo)?;

        // Create app state in staging mode
        let mut state = AppState::new_staging(file_statuses);

        // Load diff for the first file if there are any files
        if !state.file_statuses.is_empty() {
            load_initial_diff(&mut state)?;
        }

        run_app(state)?;
    }

    Ok(())
}

fn load_initial_diff(state: &mut AppState) -> Result<()> {
    use murasaki_rs::git::get_file_diff;

    if let Some(file_status) = state.current_file_status() {
        let path = file_status.path.to_string_lossy();
        let staged = file_status.is_staged() && !file_status.is_modified_in_workdir();

        match get_file_diff(&path, staged) {
            Ok(diff) => {
                if diff.is_empty() {
                    if let Ok(content) = std::fs::read_to_string(&file_status.path) {
                        state.diff_content = Some(format!("New file:\n\n{}", content));
                    } else {
                        state.diff_content = Some("No changes to display".to_string());
                    }
                } else {
                    state.diff_content = Some(diff);
                }
            }
            Err(e) => {
                state.diff_content = Some(format!("Error getting diff: {}", e));
            }
        }
    }

    Ok(())
}
