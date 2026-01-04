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
                let canonical_path = path.canonicalize()
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

    // Create app state and run
    let state = AppState::new(conflicted_files, git_operation);
    run_app(state)?;

    Ok(())
}
