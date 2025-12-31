use anyhow::{Context, Result};
use clap::Parser;
use enkai::{detect_git_operation, find_conflicted_files, parse_conflicts, run_app, AppState};

#[derive(Parser, Debug)]
#[command(name = "enkai")]
#[command(about = "A TUI tool for handling git conflicts during merge or rebase", long_about = None)]
struct Args {
    /// Specific files to resolve (if not provided, all conflicted files will be shown)
    #[arg(value_name = "FILES")]
    files: Vec<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Open git repository
    let repo = enkai::git::detector::open_repository()
        .context("Failed to open git repository")?;

    // Detect the type of operation
    let git_operation = detect_git_operation(&repo)
        .context("No merge or rebase operation in progress")?;

    // Find conflicted files
    let conflicted_paths = if args.files.is_empty() {
        find_conflicted_files(&repo).context("Failed to find conflicted files")?
    } else {
        args.files
            .into_iter()
            .map(std::path::PathBuf::from)
            .collect()
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

    // Create app state and run
    let state = AppState::new(conflicted_files, git_operation);
    run_app(state)?;

    Ok(())
}
