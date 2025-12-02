use clap::Parser;
use inquire::{
    Confirm, Select, Text,
    error::InquireError,
    formatter::{self, StringFormatter},
};
use rayon::option;
use std::{path::PathBuf, process::exit};

use doc_simfinder::{
    analysis::analyse_files, cli::{CliArgs, build_config_from_args}, file_walker::walk_from_root, interactive::{self, interactive_picker}, presentation::present_file_score
};

fn main() {
    // Parse CLI args and build runtime config
    // Mut to change it in interactive mode
    let mut args = CliArgs::parse();
    let is_interactive = args.interactive;

    if is_interactive {
        // ask for search path if not provided
        // check if search path is default and prompt for confirmation
        // otherwise ask for a new path
        if args.search_path == PathBuf::from(".") {
            match Confirm::new("Search path is current directory. Proceed?")
                .with_default(true)
                .prompt()
            {
                Ok(true) => { /* proceed with current directory */ }
                Ok(false) => {
                    match Text::new("Specify a search path (-q or --quit to exit)").prompt() {
                        Ok(p) if { !p.trim().is_empty() } => {
                            if p.trim().eq_ignore_ascii_case("-q")
                                || p.trim().eq_ignore_ascii_case("--quit")
                            {
                                println!("Exiting as per user request.");
                                exit(0);
                            }
                            args.search_path = PathBuf::from(p);
                        }
                        Ok(_) => {
                            args.search_path = PathBuf::from(".");
                        }
                        Err(e) => {
                            eprintln!("Error reading search path: {}. Exiting.", e);
                            exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error reading confirmation: {}. Exiting.", e);
                    exit(1);
                }
            }
        }

        // ask for a query if not provided
        if args.query.is_none() {
            match Text::new("Specify a search query (-q or --quit to exit)").prompt() {
                Ok(q) if { !q.trim().is_empty() } => {
                    if q.trim().eq_ignore_ascii_case("-q")
                        || q.trim().eq_ignore_ascii_case("--quit")
                    {
                        println!("Exiting as per user request.");
                        exit(0);
                    }
                    args.query = Some(q);
                }
                Ok(_) => {
                    eprintln!("Query cannot be empty. Exiting.");
                    exit(1);
                }
                Err(e) => {
                    eprintln!("Error reading query: {}. Exiting.", e);
                    exit(1);
                }
            }
        }
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
            // TODO: create loop in interactive mode to ask for further detail on some file results
            // Mabye show an initial summary first then can go into more detail
            match analyse_files(&walk.files, &config) {
                Ok(file_scores) => {
                    if let Err(e) = interactive_picker(&file_scores, &config){
                        eprintln!("Interactive picker error: {}", e);
                        exit(1);
                    }                }
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
