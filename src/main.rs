extern crate glib;
extern crate gtk;
extern crate rand;

mod game;
mod gui;
mod logic;

use game::Game;
use gui::GUI;
use logic::Logic;


fn main() {
    let game = Game::new((9, 9), 10);
    let logic = Logic::new(game);
    let gui = GUI::new(logic);

    gui.main_loop();
}
