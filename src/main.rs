extern crate gdk_pixbuf;
extern crate gtk;
extern crate rand;

mod game;
mod gui;
mod logic;

use game::Game;
use gui::GUI;
use logic::Logic;


fn main() {
    let args: Vec<String> = std::env::args().collect();

    let width;
    let height;
    let mine_count;
    if args.len() <= 1 {
        width = 30;
        height = 16;
        mine_count = 99;
    } else {
        width = args[1].parse::<usize>().unwrap();
        height = args[2].parse::<usize>().unwrap();
        mine_count = args[3].parse::<usize>().unwrap();
    }

    let game = Game::new((width, height), mine_count);
    let logic = Logic::new(game);
    let gui = GUI::new(logic);

    gui.main_loop();
}
