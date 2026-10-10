use stag::app;
use log::error;

/// Main function called from the terminal
fn main() {
    // --- Executes the app ---
    if let Err(e) = app::run() {
        error!("{:#}", e);
    }
}
