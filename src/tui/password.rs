use ratatui_core::{
    layout::{ Constraint, Direction, Layout },
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
const PASSWORD_1_TITLE: &'static str = " Enter Password ";
const PASSWORD_2_TITLE: &'static str = " Confirm Password ";

/// Ask for a password to the user.
///
/// Params:
/// * confirm: true if you want password confirmation.
///
/// Returns:
/// A Result with an Option or an I/O error. The option can be a Some(String) with the input
/// password or None if the user quited the form.
pub fn read_password(confirm: bool) -> io::Result<Option<String>> {
    // --- Setup terminal raw mode ---
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    enable_raw_mode()?;
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut term = Terminal::new(backend)?;

    // --- Form elements and variables ---
    let mut current = 0;
    let mut password_1 = TextArea::default();
    let mut password_2 = TextArea::default();
    let mut equals = false;
    let mut result: Option<String> = None;

    password_1.set_mask_char('*');
    password_2.set_mask_char('*');

    // --- Event Loop ---
    loop {
        // --- Style the form depending which element has the focus ---
        if current == 0 {
            let es = super::ElementStyle::Focus;
            password_1.set_block(super::new_block(PASSWORD_1_TITLE, es));
            password_1.set_style(super::new_style(es));
            let es = super::ElementStyle::Normal;
            password_2.set_block(super::new_block(PASSWORD_2_TITLE, es));
            password_2.set_style(super::new_style(es));
        } else {
            let es = super::ElementStyle::Normal;
            password_1.set_block(super::new_block(PASSWORD_1_TITLE, es));
            password_1.set_style(super::new_style(es));
            let es = super::ElementStyle::Focus;
            password_2.set_block(super::new_block(PASSWORD_2_TITLE, es));
            if equals {
                password_2.set_style(Style::default().fg(Color::LightGreen));
            } else {
                password_2.set_style(Style::default().fg(Color::LightRed));
            }
        }

        // --- Draw the form in the terminal ---
        term.draw(|frame| {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3),Constraint::Length(3)])
                .split(frame.area());
            frame.render_widget(&password_1, layout[0]);
            if confirm {
                frame.render_widget(&password_2, layout[1]);
            }
        })?;

        // --- Process the keybord events ---
        match crossterm::event::read()?.into() {
            Input{ key: Key::Esc, .. } => break,
            Input{ key: Key::Tab, .. } => if confirm { current = (current + 1) % 2; },
            Input{ key: Key::Up, .. } => current = 0,
            Input{ key: Key::Down, ..} => if confirm { current = 1 },
            Input{ key: Key::Enter, ..} => {
                let pass = &password_1.lines()[0];
                if confirm  {
                    if current == 0 {
                        current += 1;
                    } else if current == 1 && equals {
                        result = Some(pass.clone());
                        break;
                    }
                } else if pass != "" {
                    result = Some(pass.clone());
                    break;
                }
            },
            input => {
                let ta = if current == 0 { &mut password_1 } else { &mut password_2 };
                if ta.input(input) {
                    let p1 = &password_1.lines()[0];
                    let p2 = &password_2.lines()[0];
                    equals = p1 != "" && p1 == p2;
                }
            }
        }
    }

    // --- Disable terminal raw mode ---
    disable_raw_mode()?;
    crossterm::execute!(term.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    term.show_cursor()?;

    // --- Return the result ---
    Ok(result)
}
