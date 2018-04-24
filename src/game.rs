use rand;
use rand::Rng;

use gui::ClickedFieldState;


pub struct Game {
    dim: (usize, usize),

    mines: Vec<Vec<bool>>,
    unspread_mines: usize,
}


impl Game {
    pub fn new(dim: (usize, usize), mine_count: usize) -> Self {
        let mut mine_vec = Vec::<Vec<bool>>::new();

        for _ in 0..dim.1 {
            let mut row = Vec::<bool>::new();
            for _ in 0..dim.0 {
                row.push(false);
            }
            mine_vec.push(row);
        }

        Game {
            dim: dim,

            mines: mine_vec,
            unspread_mines: mine_count,
        }
    }

    pub fn get_dim(&self) -> (usize, usize) {
        self.dim
    }

    pub fn pressed(&mut self, pos: (usize, usize)) -> ClickedFieldState {
        if self.unspread_mines > 0 {
            self.spread_mines(pos);
        }

        if self.mines[pos.1][pos.0] {
            ClickedFieldState::Mine
        } else {
            let mut mine_count = 0;
            for yd in -1..2 {
                let y = pos.1 as i32 + yd;
                if y >= 0 && (y as usize) < self.dim.1 {
                    for xd in -1..2 {
                        let x = pos.0 as i32 + xd;
                        if x >= 0 && (x as usize) < self.dim.0 {
                            if self.mines[y as usize][x as usize] {
                                mine_count += 1;
                            }
                        }
                    }
                }
            }

            ClickedFieldState::ProxCount(mine_count)
        }
    }

    fn spread_mines(&mut self, keep_free: (usize, usize)) {
        let mut rng = rand::thread_rng();

        while self.unspread_mines > 0 {
            let x = rng.gen_range(0, self.dim.0);
            let y = rng.gen_range(0, self.dim.1);

            if (keep_free.0 as i32 - x as i32).abs() <= 1 &&
               (keep_free.1 as i32 - y as i32).abs() <= 1
            {
                continue;
            }

            if self.mines[y][x] {
                continue;
            }

            self.mines[y][x] = true;
            self.unspread_mines -= 1;
        }
    }
}
