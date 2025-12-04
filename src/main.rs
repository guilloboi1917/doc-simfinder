use clap::Parser;
use std::process::exit;

use doc_simfinder::{
    analysis::analyse_files,
    cli::{CliArgs, build_config_from_args},
    file_walker::walk_from_root,
};

#[tokio::main]
async fn main() {
    let args = CliArgs::parse();
    
    // Check if TUI mode is requested
    if args.tui {
        if let Err(e) = run_tui_mode(&args).await {
            eprintln!("TUI mode error: {}", e);
            exit(1);
        }
        return;
    }
    
    // CLI mode requires a query
    if args.query.is_none() {
        eprintln!("Error: --query is required in CLI mode. Use --tui for interactive mode.");
        exit(1);
    }

    let config = build_config_from_args(&args);

    if let Err(_) = config.validate() {
        eprintln!("Invalid configuration. Check search path, query and window sizes.");
        exit(1);
    }

    match walk_from_root(&config) {
        Ok(walk) => {
            if walk.files.is_empty() {
                println!("No files found under {}", config.search_path.display());
                return;
            }

            // Use analyse_files to process all files in parallel
            match analyse_files(&walk.files, &config) {
                Ok(file_scores) => {
                    // Print results in CLI mode
                    for score in file_scores.iter() {
                        println!("File: {} (score: {:.2})", score.path.display(), score.score);
                    }
                }
                Err(err) => {
                    eprintln!("Failed to analyse files: {}", err);
                    exit(1);
                }
            }
        }
        Err(err) => {
            eprintln!("Failed to walk files: {}", err);
            exit(1);
        }
    }
}

/// Run the advanced TUI mode with state machine
async fn run_tui_mode(args: &CliArgs) -> Result<(), Box<dyn std::error::Error>> {
    use doc_simfinder::{
        state_machine::AppState,
        tui::{App, setup_terminal, restore_terminal},
    };

    // Build initial config
    let config = build_config_from_args(args);
    
    // Create initial state
    let initial_state = AppState::Configuring {
        config,
        validation_errors: vec![],
    };
    
    // Setup terminal
    let mut terminal = setup_terminal()?;
    
    // Create and run app
    let mut app = App::new(initial_state);
    let result = app.run(&mut terminal);
    
    // Restore terminal
    restore_terminal(&mut terminal)?;
    
    result?;
    Ok(())
}
