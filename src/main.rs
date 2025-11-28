use clap::Parser;
use std::process::exit;

use doc_simfinder::{
    analysis::score_file,
    cli::{build_config_from_args, CliArgs},
    file_walker::walk_from_root,
    presentation::present_file_score,
};

fn main() {
    // Parse CLI args and build runtime config
    let args = CliArgs::parse();
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

            for file in &walk.files {
                match score_file(file.as_path(), &config) {
                    Ok(file_score) => {
                        let out = present_file_score(&file_score, &config);
                        println!("{}", out);
                    }
                    Err(err) => eprintln!("Failed to score {}: {}", file.display(), err),
                }
            }
        }
        Err(err) => {
            eprintln!("Failed to walk files: {}", err);
            exit(1);
        }
    }
}
