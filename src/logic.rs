use game::{FieldLabel, Game};
use gui::GUI;


#[derive(PartialEq, Clone, Copy)]
pub enum FieldState {
    Veiled,
    Flagged,
    Mine,
    Safe(usize),
}


pub struct Logic {
    game: Game,

    mines_spread: bool,
    flag_count: usize,
    game_state: Vec<Vec<FieldState>>,
}


impl Logic {
    pub fn new(game: Game) -> Self {
        let dim = game.get_dim();
        let mut state_vec = Vec::<Vec<FieldState>>::new();

        for _ in 0..dim.1 {
            let mut row = Vec::<FieldState>::new();
            for _ in 0..dim.0 {
                row.push(FieldState::Veiled);
            }
            state_vec.push(row);
        }

        Logic {
            game: game,

            mines_spread: false,
            flag_count: 0,
            game_state: state_vec,
        }
    }

    pub fn get_dim(&self) -> (usize, usize) {
        self.game.get_dim()
    }

    fn pos_in_bounds(&self, pos: (i32, i32)) -> Option<(usize, usize)> {
        let dim = self.game.get_dim();

        if pos.0 >= 0 && (pos.0 as usize) < dim.0 &&
           pos.1 >= 0 && (pos.1 as usize) < dim.1
        {
            Some((pos.0 as usize, pos.1 as usize))
        } else {
            None
        }
    }

    fn unveil_surrounding(&mut self, gui: &mut GUI, pos: (usize, usize)) {
        let ipos = (pos.0 as i32, pos.1 as i32);
        for yd in -1..2 {
            for xd in -1..2 {
                let dpos = (ipos.0 + xd, ipos.1 + yd);
                if let Some(upos) = self.pos_in_bounds(dpos) {
                    self.unveil(gui, upos);
                }
            }
        }
    }

    fn flag_surrounding(&mut self, gui: &mut GUI, pos: (usize, usize)) {
        let ipos = (pos.0 as i32, pos.1 as i32);
        for yd in -1..2 {
            for xd in -1..2 {
                let dpos = (ipos.0 + xd, ipos.1 + yd);
                if let Some(upos) = self.pos_in_bounds(dpos) {
                    self.flag(gui, upos);
                }
            }
        }
    }

    fn unveil(&mut self, gui: &mut GUI, pos: (usize, usize)) {
        if self.game_state[pos.1][pos.0] != FieldState::Veiled {
            return;
        }

        if !self.mines_spread {
            self.game.spread_mines(pos);
            self.mines_spread = true;
        }

        let label = self.game.get_field_label(pos);

        let state = match label {
            FieldLabel::Mine    => FieldState::Mine,
            FieldLabel::Safe(n) => FieldState::Safe(n),
        };
        self.game_state[pos.1][pos.0] = state;

        match label {
            FieldLabel::Mine => {
                /* Unveil all mines */
                let dim = self.game.get_dim();
                for y in 0..dim.1 {
                    for x in 0..dim.0 {
                        match self.game.get_field_label((x, y)) {
                            FieldLabel::Mine =>
                                gui.set_field_state((x, y), FieldState::Mine),

                            _ => ()
                        }
                    }
                }
                return;
            },

            FieldLabel::Safe(0) => {
                self.unveil_surrounding(gui, pos);
            },

            FieldLabel::Safe(_) => (),
        }

        gui.set_field_state(pos, state);

        for yd in -1..2 {
            for xd in -1..2 {
                let dpos = (pos.0 as i32 + xd, pos.1 as i32 + yd);
                if let Some(upos) = self.pos_in_bounds(dpos) {
                    self.unveil_surrounding_if_safe(gui, upos);
                }
            }
        }
    }

    fn flag(&mut self, gui: &mut GUI, pos: (usize, usize)) {
        if self.game_state[pos.1][pos.0] != FieldState::Veiled {
            return;
        }

        self.game_state[pos.1][pos.0] = FieldState::Flagged;
        self.flag_count += 1;
        gui.set_field_state(pos, FieldState::Flagged);
        gui.set_flag_count(self.flag_count);

        for yd in -1..2 {
            for xd in -1..2 {
                let dpos = (pos.0 as i32 + xd, pos.1 as i32 + yd);
                if let Some(upos) = self.pos_in_bounds(dpos) {
                    self.unveil_surrounding_if_safe(gui, upos);
                }
            }
        }
    }

    fn unflag(&mut self, gui: &mut GUI, pos: (usize, usize)) {
        if self.game_state[pos.1][pos.0] != FieldState::Flagged {
            return;
        }

        self.game_state[pos.1][pos.0] = FieldState::Veiled;
        self.flag_count -= 1;
        gui.set_field_state(pos, FieldState::Veiled);
        gui.set_flag_count(self.flag_count);
    }

    fn definitely_mined(&self, _pos: (usize, usize)) -> bool {
        false
    }

    fn get_state(&self, pos: (i32, i32)) -> Option<FieldState> {
        if let Some(upos) = self.pos_in_bounds(pos) {
            Some(self.game_state[upos.1][upos.0])
        } else {
            None
        }
    }

    fn unveil_surrounding_if_safe(&mut self, gui: &mut GUI, pos: (usize, usize))
    {
        let n;
        match self.game_state[pos.1][pos.0] {
            FieldState::Safe(x) => n = x,

            _ => return
        };

        let mut flag_count = 0;
        let mut potential_mine_count = 0;
        let ipos = (pos.0 as i32, pos.1 as i32);
        for yd in -1..2 {
            for xd in -1..2 {
                let state = self.get_state((ipos.0 + xd, ipos.1 + yd));
                match state {
                    Some(FieldState::Veiled) => {
                        potential_mine_count += 1;
                    }

                    Some(FieldState::Flagged) => {
                        flag_count += 1;
                        potential_mine_count += 1;
                    },

                    Some(FieldState::Mine) => {
                        potential_mine_count += 1;
                    },

                    _ => ()
                };
            }
        }

        if flag_count == n {
            self.unveil_surrounding(gui, pos);
        } else if potential_mine_count == n {
            self.flag_surrounding(gui, pos);
        }
    }

    pub fn pressed(&mut self, gui: &mut GUI, pos: (usize, usize)) {
        match self.game_state[pos.1][pos.0] {
            FieldState::Veiled => {
                if self.definitely_mined(pos) {
                    self.flag(gui, pos);
                } else {
                    self.unveil(gui, pos);
                }
            },

            FieldState::Safe(_) => {
                self.unveil_surrounding_if_safe(gui, pos);
            },

            _ => ()
        }
    }

    pub fn toggle_flag(&mut self, gui: &mut GUI, pos: (usize, usize)) {
        match self.game_state[pos.1][pos.0] {
            FieldState::Veiled => self.flag(gui, pos),
            FieldState::Flagged => self.unflag(gui, pos),

            _ => ()
        }
    }

    pub fn get_mine_count(&self) -> usize {
        self.game.get_mine_count()
    }

    pub fn new_game(&mut self) {
        self.game.new_game();

        let dim = self.game.get_dim();
        for y in 0..dim.1 {
            for x in 0..dim.0 {
                self.game_state[y][x] = FieldState::Veiled;
            }
        }

        self.mines_spread = false;
        self.flag_count = 0;
    }
}
