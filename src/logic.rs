use game::{FieldLabel, Game};
use gui::GUI;


pub enum FieldState {
    Veiled,
    Flagged,
    Mine,
    Safe(usize),
}


pub struct Logic {
    game: Game,

    mines_spread: bool,
    unveiled: Vec<Vec<bool>>,
}


impl Logic {
    pub fn new(game: Game) -> Self {
        let dim = game.get_dim();
        let mut unveiled_vec = Vec::<Vec<bool>>::new();

        for _ in 0..dim.1 {
            let mut row = Vec::<bool>::new();
            for _ in 0..dim.0 {
                row.push(false);
            }
            unveiled_vec.push(row);
        }

        Logic {
            game: game,

            mines_spread: false,
            unveiled: unveiled_vec,
        }
    }

    pub fn get_dim(&self) -> (usize, usize) {
        self.game.get_dim()
    }

    fn unveil_signed(&mut self, gui: &mut GUI, pos: (i32, i32)) {
        let dim = self.game.get_dim();

        if pos.0 >= 0 && (pos.0 as usize) < dim.0 &&
           pos.1 >= 0 && (pos.1 as usize) < dim.1
        {
            self.unveil(gui, (pos.0 as usize, pos.1 as usize));
        }
    }

    fn unveil(&mut self, gui: &mut GUI, pos: (usize, usize)) {
        if self.unveiled[pos.1][pos.0] {
            return;
        }
        self.unveiled[pos.1][pos.0] = true;

        if !self.mines_spread {
            self.game.spread_mines(pos);
            self.mines_spread = true;
        }

        let label = self.game.get_field_label(pos);

        match label {
            FieldLabel::Mine => {
                /* Unveil all mines */
            },

            FieldLabel::Safe(0) => {
                /* Unveil surrounding area */
                let ipos = (pos.0 as i32, pos.1 as i32);
                for yd in -1..2 {
                    for xd in -1..2 {
                        self.unveil_signed(gui, (ipos.0 + xd, ipos.1 + yd));
                    }
                }
            },

            FieldLabel::Safe(_) => (),
        }

        let state = match label {
            FieldLabel::Mine    => FieldState::Mine,
            FieldLabel::Safe(n) => FieldState::Safe(n),
        };

        gui.set_field_state(pos, state);
    }

    pub fn pressed(&mut self, gui: &mut GUI, pos: (usize, usize)) {
        self.unveil(gui, pos);
    }
}
