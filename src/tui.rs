use anyhow::Result;
use crossterm::{
    event::{
        KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
    },
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::io::{self, stdout};

pub type Tui = Terminal<CrosstermBackend<io::Stdout>>;

pub fn init() -> Result<Tui> {
    terminal::enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;

    // Enable keyboard enhancement for terminals that support it (iTerm2, Kitty, WezTerm, etc.)
    // This allows distinguishing Ctrl+Enter from plain Enter.
    // Silently ignored by terminals that don't support the Kitty protocol.
    let _ = execute!(
        stdout(),
        PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES)
    );

    let backend = CrosstermBackend::new(stdout());
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

pub fn restore() -> Result<()> {
    let _ = execute!(stdout(), PopKeyboardEnhancementFlags);
    terminal::disable_raw_mode()?;
    execute!(stdout(), LeaveAlternateScreen)?;
    Ok(())
}
