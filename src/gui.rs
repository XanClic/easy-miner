use gdk_pixbuf::Pixbuf;
use gtk;
use gtk::prelude::*;
use std;
use std::cell::RefCell;
use std::rc::Rc;

use logic::{FieldState, Logic};


struct Field {
    button: gtk::Image,
    state: FieldState,
}


pub struct GUI {
    wnd: gtk::Window,
    buttons: Vec<Vec<Field>>,
    logic: Option<Rc<RefCell<Logic>>>,

    pxb_veiled: Pixbuf,
    pxb_flagged: Pixbuf,
    pxb_mine: Pixbuf,
    pxb_safe: Vec<Pixbuf>,
}


impl GUI {
    pub fn new(logic: Logic) -> Self {
        gtk::init().unwrap();

        let wnd = gtk::Window::new(gtk::WindowType::Toplevel);
        wnd.set_title("EasyMiner");
        wnd.set_default_size(780, 420);

        let dim = logic.get_dim();
        let wnd_size = wnd.get_size();

        // FIXME: This leaves some space so the user can make the
        // window smaller.  The user cannot shrink the window beyond
        // the current size of the grid, so if the grid fills the
        // whole window, it cannot be shrunk at all.  It would be nice
        // to fix this and drop this spacing here (and in the resize
        // handler).
        let fs = std::cmp::min((wnd_size.0 - 30) / (dim.0 as i32),
                               (wnd_size.1 - 20) / (dim.1 as i32));

        let mut safe_vec = Vec::<Pixbuf>::new();
        for i in 0..9 {
            safe_vec.push(Pixbuf::new_from_file_at_size(
                format!("images/safe-{}.png", i), fs, fs).unwrap());
        }

        GUI {
            wnd: wnd,
            buttons: Vec::new(),
            logic: Some(Rc::new(RefCell::new(logic))),

            pxb_veiled: Pixbuf::new_from_file_at_size("images/veiled.png",
                                                      fs, fs).unwrap(),
            pxb_flagged: Pixbuf::new_from_file_at_size("images/flagged.png",
                                                       fs, fs).unwrap(),
            pxb_mine: Pixbuf::new_from_file_at_size("images/mine.png",
                                                    fs, fs).unwrap(),
            pxb_safe: safe_vec,
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
                let btn =
                    gtk::Image::new_from_pixbuf(&this.borrow().pxb_veiled);

                let event = gtk::EventBox::new();
                event.add(&btn);

                let cloned_logic = logic.clone();
                let cloned_this = this.clone();
                event.connect_button_press_event(move |_, _mb| {
                    let mut cbl = cloned_logic.borrow_mut();
                    let cbs = &mut *cloned_this.borrow_mut();

                    cbl.pressed(cbs, (x, y));

                    Inhibit(false)
                });

                grid.attach(&event, x as i32, y as i32, 1, 1);

                btn_row.push(Field {
                    button: btn,
                    state: FieldState::Veiled,
                });
            }

            this.borrow_mut().buttons.push(btn_row);
        }

        let ratio = (dim.0 as f32) / (dim.1 as f32);
        let af = gtk::AspectFrame::new(None, 0.5, 0.5, ratio, false);

        grid.set_halign(gtk::Align::Center);
        grid.set_valign(gtk::Align::Center);

        af.add(&grid);
        this.borrow_mut().wnd.add(&af);
        this.borrow_mut().wnd.show_all();

        this.borrow_mut().wnd.connect_delete_event(|_, _| {
            gtk::main_quit();
            Inhibit(false)
        });

        {
            let cloned_logic = logic.clone();
            let cloned_this = this.clone();
            this.borrow_mut().wnd.connect_configure_event(move |_, evt| {
                let dim = cloned_logic.borrow().get_dim();
                let wnd_size = evt.get_size();

                let fs = std::cmp::min(((wnd_size.0 - 30) as i32) / (dim.0 as i32),
                                       ((wnd_size.1 - 20) as i32) / (dim.1 as i32));

                let cbs = &mut *cloned_this.borrow_mut();
                cbs.pxb_veiled =
                    Pixbuf::new_from_file_at_size("images/veiled.png",
                                                  fs, fs).unwrap();
                cbs.pxb_flagged =
                    Pixbuf::new_from_file_at_size("images/flagged.png",
                                                  fs, fs).unwrap();
                cbs.pxb_mine =
                    Pixbuf::new_from_file_at_size("images/mine.png",
                                                  fs, fs).unwrap();
                for i in 0..9 {
                    cbs.pxb_safe[i] = Pixbuf::new_from_file_at_size(
                        format!("images/safe-{}.png", i), fs, fs).unwrap();
                }

                for y in 0..dim.1 {
                    for x in 0..dim.0 {
                        let state = cbs.buttons[y][x].state;
                        cbs.set_field_state((x, y), state);
                    }
                }

                false
            });
        }

        gtk::main();
    }

    pub fn set_field_state(&mut self, pos: (usize, usize), state: FieldState) {
        let btn = &mut self.buttons[pos.1][pos.0];

        match state {
            FieldState::Veiled => {
                btn.button.set_from_pixbuf(&self.pxb_veiled);
            },

            FieldState::Flagged => {
                btn.button.set_from_pixbuf(&self.pxb_flagged);
            },

            FieldState::Mine => {
                btn.button.set_from_pixbuf(&self.pxb_mine);
            },

            FieldState::Safe(n) => {
                btn.button.set_from_pixbuf(&self.pxb_safe[n]);
            },
        }

        btn.state = state;
    }
}
