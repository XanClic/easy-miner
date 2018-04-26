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
    let mut args: Vec<String> = std::env::args().collect();
    let mut free_args = Vec::<String>::new();

    args.remove(0);

    let mut width = 30;
    let mut height = 16;
    let mut mine_count = 99;

    let mut auto_unveil = false;
    let mut touch_mode = false;

    for arg in args {
        if arg.starts_with("--") {
            match arg.as_ref() {
                "--auto-unveil" => {
                    auto_unveil = true;
                },

                "--touch-mode" => {
                    touch_mode = true;
                },

                "--help" => {
                    println!("Available switches:");
                    println!("  --auto-unveil:");
                    println!("    Automatically unveil/flag surrounding cells \
                                  of an unveiled/flagged");
                    println!("    cell if the number of veiled non-flagged \
                                  surrounding cells is");
                    println!("    equal to the cell’s number label (i.e. what \
                                  happens when manually");
                    println!("    clicking on an unveiled cell).");
                    println!("");
                    println!("  --touch-mode:");
                    println!("    When clicking on a veiled cell, try to \
                                  figure out whether you");
                    println!("    would think it is safe or a mine.  Depending \
                                  on the answer, the");
                    println!("    field will be unveiled of flagged.");

                    return;
                },

                _ => {
                    panic!("Unrecognized parameter {}", arg);
                }
            }
        } else {
            free_args.push(arg);
        }
    }

    if free_args.len() > 0 {
        if free_args.len() < 3 {
            panic!("Either no or all of the field dimensions must be \
                    specified");
        }

        width = free_args[0].parse::<usize>().unwrap();
        height = free_args[1].parse::<usize>().unwrap();
        mine_count = free_args[2].parse::<usize>().unwrap();

        if free_args.len() > 3 {
            panic!("Garbage arguments");
        }
    }

    let game = Game::new((width, height), mine_count);
    let logic = Logic::new(game, auto_unveil, touch_mode);
    let gui = GUI::new(logic);

    gui.main_loop();
}
