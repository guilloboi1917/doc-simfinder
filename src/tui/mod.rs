// TUI module - Terminal User Interface
//
// See docs/copilot/ui.md and docs/copilot/tui-integration.md for architecture

pub mod layout;
pub mod widgets;
pub mod focus;
pub mod app;

pub use app::App;
pub use focus::{Focus, FocusManager};
pub use widgets::Dashboard;

use std::io;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

/// Setup the terminal for TUI mode
pub fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

/// Restore the terminal to normal mode
pub fn restore_terminal(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

#[cfg(test)]
mod tests;
