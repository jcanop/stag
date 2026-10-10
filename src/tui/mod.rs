pub mod input;
pub mod password;

use ratatui::style::{Color, Modifier, Style};
use ratatui_widgets::{
    block::Block,
    borders::Borders,
};

#[derive(Copy, Clone, Debug)]
enum ElementStyle {
    Normal,
    Focus,
}

/// Creates a new style depending the element style.
///
/// Params:
/// * es: Element style desired.
///
/// Returns:
/// The element style.
fn new_style(es: ElementStyle) -> Style {
    match es {
        ElementStyle::Normal => Style::default().fg(Color::DarkGray),
        ElementStyle::Focus => Style::default().fg(Color::White)
            .add_modifier(Modifier::BOLD),
    }
}

/// Creates a new block depending the element style.
///
/// Params:
/// * es: Element style desired.
///
/// Returns:
/// The styled block.
fn new_block(title: &str, es: ElementStyle) -> Block<'_> {
    let style = new_style(es);
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(style)
}
