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

#[derive(Clone)]
struct GameState {
    board: Vec<Vec<ICellState>>,
    dim: (usize, usize),
    flag_count: usize,
    mine_count: usize,
    unveiled_count: usize,
}

pub struct Logic {
    game: Game,

    auto_unveil: bool,
    touch_mode: bool,

    mines_spread: bool,
    flag_count: usize,
    mine_count: usize,
    unveiled_count: usize,
    game_state: GameState,

    game_over: bool,
    ui_updates: Vec<UIUpdate>,
}


impl Logic {
    pub fn new(game: Game, auto_unveil: bool, touch_mode: bool) -> Self {
        let dim = game.get_dim();
        let mine_count = game.get_mine_count();

        Logic {
            game: game,

            auto_unveil: auto_unveil,
            touch_mode: touch_mode,

            mines_spread: false,
            flag_count: 0,
            mine_count: mine_count,
            unveiled_count: 0,
            game_state: GameState::new(dim, mine_count),

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
        assert!(self.game_state.get(pos) != ICellState::DefinitelySafe);

        if self.game_state.get(pos) != ICellState::Veiled {
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
        self.game_state.set(pos, state);

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
                        if self.game_state.get((x, y)) == ICellState::Veiled {
                            self.flag((x, y));
                        }
                    }
                }
            }
        }

        if self.auto_unveil {
            self.unveil_around_sis(pos);
        }
    }

    fn flag(&mut self, pos: (usize, usize)) {
        assert!(self.game_state.get(pos) != ICellState::DefinitelySafe);

        if self.game_state.get(pos) != ICellState::Veiled {
            return;
        }

        self.game_state.set(pos, ICellState::Flagged);
        self.flag_count += 1;
        self.ui_updates.push(UIUpdate { pos: pos, state: CellState::Flagged });

        if self.auto_unveil {
            self.unveil_around_sis(pos);
        }
    }

    fn unflag(&mut self, pos: (usize, usize)) {
        if self.game_state.get(pos) != ICellState::Flagged {
            return;
        }

        self.game_state.set(pos, ICellState::Veiled);
        self.flag_count -= 1;
        self.ui_updates.push(UIUpdate { pos: pos, state: CellState::Veiled });
    }

    fn unveil_around_sis(&mut self, center: (usize, usize)) {
        for yd in -1..2 {
            for xd in -1..2 {
                let dpos = (center.0 as i32 + xd, center.1 as i32 + yd);
                if let Some(upos) = self.pos_in_bounds(dpos) {
                    self.unveil_surrounding_if_safe(upos);
                }
            }
        }
    }

    fn definitely_mined(&self, pos: (usize, usize)) -> bool {
        let mut hypothetical_state = self.game_state.clone();

        hypothetical_state.set(pos, ICellState::DefinitelySafe);
        return !hypothetical_state.environment_propagate(pos);
    }

    fn unveil_surrounding_if_safe(&mut self, pos: (usize, usize))
    {
        match self.game_state.safe_cell_environment(pos) {
            CellEnvironment::AllSafe  => self.unveil_surrounding(pos),
            CellEnvironment::AllMines => self.flag_surrounding(pos),

            _ => ()
        }
    }

    pub fn pressed(&mut self, pos: (usize, usize)) {
        if self.game_over {
            return;
        }

        assert!(self.game_state.get(pos) != ICellState::DefinitelySafe);

        match self.game_state.get(pos) {
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

        assert!(self.game_state.get(pos) != ICellState::DefinitelySafe);

        match self.game_state.get(pos) {
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

        self.game_state.clear();
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


impl GameState {
    fn new(dim: (usize, usize), mine_count: usize) -> Self {
        let mut board = Vec::<Vec<ICellState>>::new();

        for _ in 0..dim.1 {
            let mut row = Vec::<ICellState>::new();
            for _ in 0..dim.0 {
                row.push(ICellState::Veiled);
            }
            board.push(row);
        }

        GameState {
            board: board,
            dim: dim,
            flag_count: 0,
            mine_count: mine_count,
            unveiled_count: 0,
        }
    }

    fn clear(&mut self) {
        for y in 0..self.dim.1 {
            for x in 0..self.dim.0 {
                self.board[y][x] = ICellState::Veiled;
            }
        }

        self.flag_count = 0;
        self.unveiled_count = 0;
    }

    fn get(&self, pos: (usize, usize)) -> ICellState {
        self.board[pos.1][pos.0]
    }

    fn set(&mut self, pos: (usize, usize), state: ICellState) {
        let old_state = self.get(pos);

        if old_state == state {
            return;
        }

        match state {
            ICellState::Flagged => {
                self.flag_count += 1;
            },

            ICellState::DefinitelySafe | ICellState::Safe(_) => {
                self.unveiled_count += 1;
            },

            _ => ()
        }

        self.board[pos.1][pos.0] = state;
    }

    fn pos_in_bounds(&self, pos: (i32, i32)) -> Option<(usize, usize)> {
        if pos.0 >= 0 && (pos.0 as usize) < self.dim.0 &&
           pos.1 >= 0 && (pos.1 as usize) < self.dim.1
        {
            Some((pos.0 as usize, pos.1 as usize))
        } else {
            None
        }
    }

    fn get_i32(&self, pos: (i32, i32)) -> Option<ICellState> {
        if let Some(upos) = self.pos_in_bounds(pos) {
            Some(self.board[upos.1][upos.0])
        } else {
            None
        }
    }

    fn safe_cell_environment(&self, pos: (usize, usize)) -> CellEnvironment {
        let n;
        match self.get(pos) {
            ICellState::Safe(x) => { n = x; },

            _ => return CellEnvironment::Unsure
        };

        let mut flag_count = 0;
        let mut potential_mine_count = 0;
        let ipos = (pos.0 as i32, pos.1 as i32);
        for yd in -1..2 {
            for xd in -1..2 {
                match self.get_i32((ipos.0 + xd, ipos.1 + yd)) {
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

    // Analyzes the environment of @pos.  If everything must be mines,
    // they are all flagged.  If everything must be safe, it is marked
    // as DefinitelySafe.
    // If the state is impossible, false is returned.  Otherwise, true
    // is returned.
    fn propagate(&mut self, pos: (usize, usize)) -> bool {
        match self.safe_cell_environment(pos) {
            CellEnvironment::AllSafe =>
                self.mark_environment_safe(pos),

            CellEnvironment::AllMines =>
                self.mark_environment_mines(pos),

            CellEnvironment::Unsure =>
                true,

            CellEnvironment::Impossible =>
                false,
        }
    }

    fn environment_propagate(&mut self, center: (usize, usize)) -> bool {
        if !self.sanity_check() {
            return false;
        }

        for yd in -1..2 {
            for xd in -1..2 {
                if xd == 0 && yd == 0 {
                    continue;
                }

                let dpos = (center.0 as i32 + xd, center.1 as i32 + yd);
                if let Some(upos) = self.pos_in_bounds(dpos) {
                    if !self.propagate(upos) {
                        return false;
                    }
                }
            }
        }

        return true;
    }

    fn mark_environment_safe(&mut self, pos: (usize, usize)) -> bool {
        for yd in -1..2 {
            for xd in -1..2 {
                if xd == 0 && yd == 0 {
                    continue;
                }

                let dpos = (pos.0 as i32 + xd, pos.1 as i32 + yd);
                if let Some(upos) = self.pos_in_bounds(dpos) {
                    match self.get(upos) {
                        ICellState::Veiled => {
                            self.set(upos, ICellState::DefinitelySafe);
                            if !self.sanity_check() {
                                return false;
                            }
                            if !self.environment_propagate(upos) {
                                return false;
                            }
                        },

                        _ => ()
                    }
                }
            }
        }

        return true;
    }

    fn mark_environment_mines(&mut self, pos: (usize, usize)) -> bool {
        for yd in -1..2 {
            for xd in -1..2 {
                if xd == 0 && yd == 0 {
                    continue;
                }

                let dpos = (pos.0 as i32 + xd, pos.1 as i32 + yd);
                if let Some(upos) = self.pos_in_bounds(dpos) {
                    match self.get(upos) {
                        ICellState::Veiled => {
                            self.set(upos, ICellState::Flagged);
                            if !self.sanity_check() {
                                return false;
                            }
                            if !self.environment_propagate(upos) {
                                return false;
                            }
                        },

                        _ => ()
                    }
                }
            }
        }

        return true;
    }

    fn sanity_check(&self) -> bool {
        if self.flag_count > self.mine_count {
            return false;
        }

        if self.unveiled_count + self.mine_count > self.dim.0 * self.dim.1 {
            return false;
        }

        return true;
    }
}
