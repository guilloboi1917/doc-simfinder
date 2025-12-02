use crossterm::event::Event;
use inquire::{InquireError, Select};

use crate::{analysis::FileScore, config::Config, presentation::present_file_score};

// New type to implement Display for FileScore
#[derive(Clone)]
struct MenuItem<'a>(&'a FileScore);

// 2.  Display only for the new-type
impl<'a> std::fmt::Display for MenuItem<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - score {:.2}", self.0.path.display(), self.0.score)
    }
}

pub fn clear_screen() {
    println!("{esc}c", esc = 27 as char);
}

pub fn detail_view(file_score: &FileScore, config: &Config) -> std::io::Result<()> {
    // clear screen
    clear_screen();
    println!("{}\n", present_file_score(file_score, config));
    println!("Press <Backspace> to return to the list\n");

    wait_for_backspace()
}

pub fn interactive_picker(file_scores: &[FileScore], config: &Config) -> Result<(), InquireError> {
    let candidates: Vec<MenuItem> = file_scores
        .iter()
        .filter(|fs| fs.score > config.threshold)
        .map(MenuItem)
        .collect();

    if candidates.is_empty() {
        println!("No files above the threshold ({})", config.threshold);
        return Ok(());
    }

    loop {
        clear_screen();

        let msg = format!(
            "Found {} files above the threshold ({:.2}). Use arrow keys to navigate and Enter to select a file for detailed view.\n",
            candidates.len(),
            config.threshold
        );
        // ask the user
        let menu_items = Select::new(&msg, candidates.clone())
            .with_help_message("↑↓ navigate -- Enter pickclear")
            .prompt()?;

        // Show detail view
        detail_view(menu_items.0, config)?;
    }
}

fn wait_for_backspace() -> std::io::Result<()> {
    loop {
        if let Event::Key(key_event) = crossterm::event::read()? {
            if key_event.code == crossterm::event::KeyCode::Backspace {
                return Ok(());
            }
        }
    }
}
