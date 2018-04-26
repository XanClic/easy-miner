use std;

use game::{CellLabel, Game};


#[derive(PartialEq, Clone, Copy)]
pub enum CellState {
    Veiled,
    Flagged,
    Mine,
    Safe(usize),
}

#[derive(PartialEq, Clone, Copy)]
enum ICellState { // Internal CellState (with additional states)
    Veiled,
    Flagged,
    Mine,
    Safe(usize),
    DefinitelySafe,
}

enum CellEnvironment {
    AllMines,
    AllSafe,
    Unsure,
    Impossible,
}


pub struct UIUpdate {
    pub pos: (usize, usize),
    pub state: CellState,
}


pub struct Logic {
    game: Game,

    auto_unveil: bool,
    touch_mode: bool,

    mines_spread: bool,
    flag_count: usize,
    mine_count: usize,
    unveiled_count: usize,
    game_state: Vec<Vec<ICellState>>,

    game_over: bool,
    ui_updates: Vec<UIUpdate>,
}


impl Logic {
    pub fn new(game: Game, auto_unveil: bool, touch_mode: bool) -> Self {
        let dim = game.get_dim();
        let mut state_vec = Vec::<Vec<ICellState>>::new();

        for _ in 0..dim.1 {
            let mut row = Vec::<ICellState>::new();
            for _ in 0..dim.0 {
                row.push(ICellState::Veiled);
            }
            state_vec.push(row);
        }

        let mine_count = game.get_mine_count();

        Logic {
            game: game,

            auto_unveil: auto_unveil,
            touch_mode: touch_mode,

            mines_spread: false,
            flag_count: 0,
            mine_count: mine_count,
            unveiled_count: 0,
            game_state: state_vec,

            game_over: false,
            ui_updates: Vec::<UIUpdate>::new(),
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

    fn unveil_surrounding(&mut self, pos: (usize, usize)) {
        let ipos = (pos.0 as i32, pos.1 as i32);
        for yd in -1..2 {
            for xd in -1..2 {
                let dpos = (ipos.0 + xd, ipos.1 + yd);
                if let Some(upos) = self.pos_in_bounds(dpos) {
                    self.unveil(upos);
                }
            }
        }
    }

    fn flag_surrounding(&mut self, pos: (usize, usize)) {
        let ipos = (pos.0 as i32, pos.1 as i32);
        for yd in -1..2 {
            for xd in -1..2 {
                let dpos = (ipos.0 + xd, ipos.1 + yd);
                if let Some(upos) = self.pos_in_bounds(dpos) {
                    self.flag(upos);
                }
            }
        }
    }

    fn unveil(&mut self, pos: (usize, usize)) {
        assert!(self.game_state[pos.1][pos.0] != ICellState::DefinitelySafe);

        if self.game_state[pos.1][pos.0] != ICellState::Veiled {
            return;
        }

        if !self.mines_spread {
            self.game.spread_mines(pos);
            self.mines_spread = true;
        }

        let label = self.game.get_cell_label(pos);

        let state = match label {
            CellLabel::Mine    => ICellState::Mine,
            CellLabel::Safe(n) => ICellState::Safe(n),
        };
        self.game_state[pos.1][pos.0] = state;

        match label {
            CellLabel::Mine => {
                // Hit a mine, so the game has been lost
                self.game_over = true;

                /* Unveil all mines */
                let dim = self.game.get_dim();
                for y in 0..dim.1 {
                    for x in 0..dim.0 {
                        match self.game.get_cell_label((x, y)) {
                            CellLabel::Mine =>
                                self.ui_updates.push(UIUpdate {
                                    pos: (x, y),
                                    state: CellState::Mine,
                                }),

                            _ => ()
                        }
                    }
                }
                return;
            },

            CellLabel::Safe(0) => {
                self.unveil_surrounding(pos);
            },

            CellLabel::Safe(_) => (),
        }

        self.unveiled_count += 1;
        self.ui_updates.push(UIUpdate {
            pos: pos,
            state: CellState::from(state)
        });

        let dim = self.game.get_dim();
        if self.unveiled_count + self.mine_count == dim.0 * dim.1 {
            // Unveiled all safe cells, so the game has been won
            self.game_over = true;

            if self.flag_count < self.mine_count {
                // Auto-flag the rest
                for y in 0..dim.1 {
                    for x in 0..dim.0 {
                        if self.game_state[y][x] == ICellState::Veiled {
                            self.flag((x, y));
                        }
                    }
                }
            }
        }

        if self.auto_unveil {
            for yd in -1..2 {
                for xd in -1..2 {
                    let dpos = (pos.0 as i32 + xd, pos.1 as i32 + yd);
                    if let Some(upos) = self.pos_in_bounds(dpos) {
                        self.unveil_surrounding_if_safe(upos);
                    }
                }
            }
        }
    }

    fn flag(&mut self, pos: (usize, usize)) {
        assert!(self.game_state[pos.1][pos.0] != ICellState::DefinitelySafe);

        if self.game_state[pos.1][pos.0] != ICellState::Veiled {
            return;
        }

        self.game_state[pos.1][pos.0] = ICellState::Flagged;
        self.flag_count += 1;
        self.ui_updates.push(UIUpdate { pos: pos, state: CellState::Flagged });

        if self.auto_unveil {
            for yd in -1..2 {
                for xd in -1..2 {
                    let dpos = (pos.0 as i32 + xd, pos.1 as i32 + yd);
                    if let Some(upos) = self.pos_in_bounds(dpos) {
                        self.unveil_surrounding_if_safe(upos);
                    }
                }
            }
        }
    }

    fn unflag(&mut self, pos: (usize, usize)) {
        if self.game_state[pos.1][pos.0] != ICellState::Flagged {
            return;
        }

        self.game_state[pos.1][pos.0] = ICellState::Veiled;
        self.flag_count -= 1;
        self.ui_updates.push(UIUpdate { pos: pos, state: CellState::Veiled });
    }

    fn definitely_mined(&self, pos: (usize, usize)) -> bool {
        let ipos = (pos.0 as i32, pos.1 as i32);
        for yd in -1..2 {
            for xd in -1..2 {
                let dpos = (ipos.0 + xd, ipos.1 + yd);
                if let Some(upos) = self.pos_in_bounds(dpos) {
                    match self.safe_cell_environment(upos) {
                        CellEnvironment::AllSafe  => return false,
                        CellEnvironment::AllMines => return true,

                        _ => ()
                    }
                }
            }
        }

        return false;
    }

    fn get_state(&self, pos: (i32, i32)) -> Option<ICellState> {
        if let Some(upos) = self.pos_in_bounds(pos) {
            Some(self.game_state[upos.1][upos.0])
        } else {
            None
        }
    }

    fn safe_cell_environment(&self, pos: (usize, usize)) -> CellEnvironment {
        let n;
        match self.game_state[pos.1][pos.0] {
            ICellState::Safe(x) => { n = x; },

            _ => return CellEnvironment::Unsure
        };

        let mut flag_count = 0;
        let mut potential_mine_count = 0;
        let ipos = (pos.0 as i32, pos.1 as i32);
        for yd in -1..2 {
            for xd in -1..2 {
                let state = self.get_state((ipos.0 + xd, ipos.1 + yd));
                match state {
                    Some(ICellState::Veiled) => {
                        potential_mine_count += 1;
                    }

                    Some(ICellState::Flagged) => {
                        flag_count += 1;
                        potential_mine_count += 1;
                    },

                    Some(ICellState::Mine) => {
                        potential_mine_count += 1;
                    },

                    _ => ()
                };
            }
        }

        if flag_count == n {
            CellEnvironment::AllSafe
        } else if potential_mine_count == n {
            CellEnvironment::AllMines
        } else if flag_count > n || potential_mine_count < n {
            CellEnvironment::Impossible
        } else {
            CellEnvironment::Unsure
        }
    }

    fn unveil_surrounding_if_safe(&mut self, pos: (usize, usize))
    {
        match self.safe_cell_environment(pos) {
            CellEnvironment::AllSafe  => self.unveil_surrounding(pos),
            CellEnvironment::AllMines => self.flag_surrounding(pos),

            _ => ()
        }
    }

    pub fn pressed(&mut self, pos: (usize, usize)) {
        if self.game_over {
            return;
        }

        assert!(self.game_state[pos.1][pos.0] != ICellState::DefinitelySafe);

        match self.game_state[pos.1][pos.0] {
            ICellState::Veiled => {
                if self.touch_mode {
                    if self.definitely_mined(pos) {
                        self.flag(pos);
                    } else {
                        self.unveil(pos);
                    }
                } else {
                    self.unveil(pos);
                }
            },

            ICellState::Safe(_) => {
                self.unveil_surrounding_if_safe(pos);
            },

            _ => ()
        }
    }

    pub fn toggle_flag(&mut self, pos: (usize, usize)) {
        if self.game_over {
            return;
        }

        assert!(self.game_state[pos.1][pos.0] != ICellState::DefinitelySafe);

        match self.game_state[pos.1][pos.0] {
            ICellState::Veiled => self.flag(pos),
            ICellState::Flagged => self.unflag(pos),

            _ => ()
        }
    }

    pub fn get_mine_count(&self) -> usize {
        self.mine_count
    }

    pub fn get_flag_count(&self) -> usize {
        self.flag_count
    }

    pub fn get_ui_updates(&mut self) -> Vec<UIUpdate> {
        std::mem::replace(&mut self.ui_updates, Vec::<UIUpdate>::new())
    }

    pub fn new_game(&mut self) {
        self.game.new_game();

        let dim = self.game.get_dim();
        for y in 0..dim.1 {
            for x in 0..dim.0 {
                self.game_state[y][x] = ICellState::Veiled;
            }
        }

        self.mines_spread = false;
        self.flag_count = 0;
        self.unveiled_count = 0;

        self.game_over = false;
    }
}


impl CellState {
    fn from(ics: ICellState) -> Self {
        match ics {
            ICellState::Veiled  => CellState::Veiled,
            ICellState::Flagged => CellState::Flagged,
            ICellState::Mine    => CellState::Mine,
            ICellState::Safe(n) => CellState::Safe(n),

            _ => panic!("Cannot convert ICellState to CellState")
        }
    }
}
