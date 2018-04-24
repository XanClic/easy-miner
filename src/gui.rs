use glib;
use gtk;
use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use logic::{FieldState, Logic};


struct Field {
    button: gtk::ToggleButton,
    handler: glib::signal::SignalHandlerId,
}


pub struct GUI {
    wnd: gtk::Window,
    buttons: Vec<Vec<Field>>,
    logic: Option<Rc<RefCell<Logic>>>,
}


impl GUI {
    pub fn new(logic: Logic) -> Self {
        gtk::init().unwrap();

        let wnd = gtk::Window::new(gtk::WindowType::Toplevel);
        wnd.set_title("EasyMiner");

        GUI {
            wnd: wnd,
            buttons: Vec::new(),
            logic: Some(Rc::new(RefCell::new(logic))),
        }
    }

    pub fn main_loop(mut self) {
        let grid = gtk::Grid::new();

        let logic = self.logic.unwrap();
        self.logic = None;

        let dim = logic.borrow().get_dim();

        let this = Rc::new(RefCell::new(self));

        for y in 0..dim.1 {
            let mut btn_row = Vec::<Field>::new();

            for x in 0..dim.0 {
                let btn = gtk::ToggleButton::new_with_label(" ");

                let cloned_logic = logic.clone();
                let cloned_this = this.clone();
                let handler = btn.connect_clicked(move |_| {
                    let mut cbl = cloned_logic.borrow_mut();
                    let cbs = &mut *cloned_this.borrow_mut();

                    cbl.pressed(cbs, (x, y));
                });

                btn.set_hexpand(true);
                btn.set_halign(gtk::Align::Fill);
                btn.set_vexpand(true);
                btn.set_valign(gtk::Align::Fill);

                grid.attach(&btn, x as i32, y as i32, 1, 1);

                btn_row.push(Field {
                    button: btn,
                    handler: handler,
                });
            }

            this.borrow_mut().buttons.push(btn_row);
        }

        grid.set_hexpand(true);
        grid.set_halign(gtk::Align::Fill);
        grid.set_vexpand(true);
        grid.set_valign(gtk::Align::Fill);

        let ratio = (dim.0 as f32) / (dim.1 as f32);
        let af = gtk::AspectFrame::new(None, 0.5, 0.5, ratio, false);

        af.add(&grid);
        this.borrow_mut().wnd.add(&af);
        this.borrow_mut().wnd.show_all();

        this.borrow_mut().wnd.connect_delete_event(|_, _| {
            gtk::main_quit();
            Inhibit(false)
        });

        gtk::main();
    }

    pub fn set_field_state(&mut self, pos: (usize, usize), state: FieldState) {
        let btn = &mut self.buttons[pos.1][pos.0];

        btn.button.block_signal(&btn.handler);

        match state {
            FieldState::Veiled => {
                btn.button.set_active(false);
                btn.button.set_label(" ");
            },

            FieldState::Flagged => {
                btn.button.set_active(true);
                btn.button.set_label("🚩");
            },

            FieldState::Mine => {
                btn.button.set_active(true);
                btn.button.set_label("💣");
            },

            FieldState::Safe(0) => {
                btn.button.set_active(true);
                btn.button.set_label(" ");
            },

            FieldState::Safe(n) => {
                btn.button.set_active(true);
                btn.button.set_label(&n.to_string());
            },
        }

        btn.button.unblock_signal(&btn.handler);
    }
}
