mod ai;
mod game;
mod tui;

fn main() {
    let x = crate::tui::SinglePlayerTui::new();

    tui::SinglePlayerTui::init(x);

    ratatui::restore();
}
