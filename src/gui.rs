use gtk;
use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use game::Game;


pub enum ClickedFieldState {
    Mine,
    ProxCount(usize),
}


pub struct GUI {
}


impl GUI {
    pub fn new(game: Game) -> Self {
        gtk::init().unwrap();

        let wnd = gtk::Window::new(gtk::WindowType::Toplevel);
        wnd.set_title("EasyMiner");

        let rcd_game = Rc::new(RefCell::new(game));

        let grid = gtk::Grid::new();
        let dim = rcd_game.borrow().get_dim();

        for y in 0..dim.1 {
            for x in 0..dim.0 {
                let btn = gtk::ToggleButton::new_with_label(" ");

                let game_clone = rcd_game.clone();
                btn.connect_clicked(move |btn| {
                    if !btn.get_active() {
                        // Cannot unpress
                        btn.set_active(true);
                    } else {
                        // Pressed
                        match game_clone.borrow_mut().pressed((x, y)) {
                            ClickedFieldState::Mine =>
                                btn.set_label("💣"),

                            ClickedFieldState::ProxCount(n) =>
                                btn.set_label(&n.to_string()),
                        }
                    }
                });

                btn.set_hexpand(true);
                btn.set_halign(gtk::Align::Fill);
                btn.set_vexpand(true);
                btn.set_valign(gtk::Align::Fill);

                grid.attach(&btn, x as i32, y as i32, 1, 1);
            }
        }

        grid.set_hexpand(true);
        grid.set_halign(gtk::Align::Fill);
        grid.set_vexpand(true);
        grid.set_valign(gtk::Align::Fill);

        let ratio = (dim.0 as f32) / (dim.1 as f32);
        let af = gtk::AspectFrame::new(None, 0.5, 0.5, ratio, false);

        af.add(&grid);
        wnd.add(&af);
        wnd.show_all();

        wnd.connect_delete_event(|_, _| {
            gtk::main_quit();
            Inhibit(false)
        });

        GUI {
        }
    }


    pub fn main_loop(self) {
        gtk::main();
    }
}
