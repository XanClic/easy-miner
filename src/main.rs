extern crate gtk;
extern crate rand;

mod game;
mod gui;

use game::Game;
use gui::GUI;


fn main() {
    let game = Game::new((9, 9), 10);
    let gui = GUI::new(game);

    gui.main_loop();
}
