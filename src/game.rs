use rand;
use rand::Rng;

pub enum FieldLabel {
    Mine,
    Safe(usize),
}


pub struct Game {
    dim: (usize, usize),

    mines: Vec<Vec<bool>>,
    unspread_mines: usize,
    total_mines: usize,
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
            total_mines: mine_count,
        }
    }

    pub fn get_dim(&self) -> (usize, usize) {
        self.dim
    }

    fn has_mine(&self, pos: (i32, i32)) -> bool {
        if pos.0 >= 0 && (pos.0 as usize) < self.dim.0 &&
           pos.1 >= 0 && (pos.1 as usize) < self.dim.1
        {
            self.mines[pos.1 as usize][pos.0 as usize]
        } else {
            false
        }
    }

    pub fn get_field_label(&mut self, pos: (usize, usize)) -> FieldLabel {
        if self.mines[pos.1][pos.0] {
            FieldLabel::Mine
        } else {
            let mut mine_count = 0;
            let ipos = (pos.0 as i32, pos.1 as i32);
            for yd in -1..2 {
                for xd in -1..2 {
                    if self.has_mine((ipos.0 + xd, ipos.1 + yd)) {
                        mine_count += 1;
                    }
                }
            }

            FieldLabel::Safe(mine_count)
        }
    }

    pub fn spread_mines(&mut self, keep_free: (usize, usize)) {
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

    pub fn get_mine_count(&self) -> usize {
        self.total_mines
    }
}
