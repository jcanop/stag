use ratatui_core::{
    style::{ Color, Style },
    terminal::Terminal,
};
use ratatui_crossterm::crossterm::{
    event::{ DisableMouseCapture, EnableMouseCapture },
    terminal::{ EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode },
};
use ratatui_crossterm::{ CrosstermBackend, crossterm };
use ratatui_textarea::{ Input, Key, TextArea };
use std::io;

// --- Constants ---
const TITLE: &'static str = " Enter the text to embed. [Press Ctrl-D when finished or ESC to cancel] ";

/// Ask for the user for a text.
///
/// Returns:
/// A Result with an Option or an I/O error. The option can be a Some(String) with the input
/// or None if the user quited the form.
pub fn read_input() -> io::Result<Option<String>> {
    // --- Setup terminal raw mode ---
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    enable_raw_mode()?;
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut term = Terminal::new(backend)?;

    // --- Form elements and variables ---
    let mut textarea = TextArea::default();
    textarea.set_line_number_style(Style::default().fg(Color::DarkGray));
    textarea.set_block(super::new_block(TITLE, super::ElementStyle::Focus));
    let mut text: Option<String> = None;

    // --- Event Loop ---
    loop {
        // --- Draw the TextArea in the terminal ---
        term.draw(|f| {
            f.render_widget(&textarea, f.area());
        })?;

        // --- Process the keybord events ---
        match crossterm::event::read()? {
            event => match event.into() {
                Input { key: Key::Esc, .. } => break,
                Input { key: Key::Char('d'), ctrl: true, .. } => {
                    text = Some(textarea.lines().join("\n"));
                    break;
                },
                input => {
                    textarea.input(input);
                }
            },
        }
    }

    // --- Disable terminal raw mode ---
    disable_raw_mode()?;
    crossterm::execute!(term.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    term.show_cursor()?;

    // --- Return the result ---
    Ok(text)
}
