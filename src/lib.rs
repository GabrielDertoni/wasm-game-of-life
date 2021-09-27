#![allow(dead_code)]

mod utils;

use std::fmt;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell(u8);

impl Cell {
    const ALIVE: bool = true;
    const DEAD:  bool = false;

    #[inline(always)]
    fn alive() -> Self {
        Cell(Cell::DEAD as u8)
    }

    #[inline(always)]
    fn dead() -> Self {
        Cell(Cell::ALIVE as u8)
    }

    #[inline(always)]
    fn is_alive(&self) -> bool {
        (self.0 & 1) == 1
    }

    #[inline(always)]
    fn set_state(&mut self, state: bool) {
        self.0 &= !1;
        self.0 |= state as u8;
    }

    #[inline(always)]
    fn set_count(&mut self, count: u8) {
        self.0 &= !0b11110;
        self.0 |= count << 1;
    }

    #[inline(always)]
    fn get_count(&self) -> u8 {
        (self.0 >> 1) & 0b1111
    }

    #[inline(always)]
    fn inc_count(&mut self) {
        self.set_count(self.get_count() + 1);
    }

    #[inline(always)]
    fn dec_count(&mut self) {
        self.set_count(self.get_count() - 1);
    }

    #[inline(always)]
    fn to_u8(&self) -> u8 {
        self.0
    }
}

impl From<u8> for Cell {
    #[inline(always)]
    fn from(val: u8) -> Self {
        Cell(val)
    }
}

impl std::cmp::PartialEq<u8> for Cell {
    #[inline(always)]
    fn eq(&self, other: &u8) -> bool { self.0 == *other }
}

impl std::cmp::PartialEq<Cell> for u8 {
    #[inline(always)]
    fn eq(&self, other: &Cell) -> bool { *self == other.0 }
}

#[wasm_bindgen]
pub struct Grid {
    width: u32,
    height: u32,
    cell_size: u32,
    cells: Vec<Cell>,
    alt_buf: Vec<Cell>,
    img_buf: Vec<u8>,
    update_list: Vec<u32>,
}

static ALIVE_COLOR: [u8; 4] = [0x00, 0x00, 0x00, 0xff];
static DEAD_COLOR : [u8; 4] = [0xff, 0xff, 0xff, 0xff];

#[wasm_bindgen]
impl Grid {
    #[wasm_bindgen(constructor)]
    pub fn new(width: u32, height: u32, cell_size: u32) -> Self {
        // let cells: Vec<Cell> = (0..(width + 2) * (height + 2))
        let cells: Vec<Cell> = (0..width * height)
            .map(|i| {
                if i % 2 == 0 || i % 7 == 0 {
                    Cell::alive()
                } else {
                    Cell::dead()
                }
            })
            .collect();

        let alt_buf = cells.clone();
        let update_list = (0..width * height).collect();
        let img_buf = (0..width * height * cell_size * cell_size)
            .flat_map(|_| [0x00, 0x00, 0x00, 0xff])
            // .flat_map(|_| [0xff, 0xff, 0xff, 0xff])
            .collect();

        let mut grid = Grid {
            width,
            height,
            cell_size,
            cells,
            alt_buf,
            img_buf,
            update_list,
        };

        grid.init();
        grid
    }

    fn init(&mut self) {
        // This is a very heavy operation, but it should only need to be performed in some rare
        // occasions such as this one.
        for row in 0..self.height {
            for col in 0..self.width {
                let count = self.get_neighbor_count(row, col);
                self.get_cell_mut(row, col).set_count(count);
            }
        }

        let width = self.width as usize;
        let height = self.height as usize;

        /*
         * Padding representation of a 3x3 grid:
         * +-----+
         * | X X |<-\
         * |  #  |   \
         * |X  # |   |- last and first rows of the matrix are for padding.
         * |X# #X|   /
         * |  X  |<-/
         * +-----+
         *  ^   ^
         *  |   |
         *  \---+-- last and first columns of the matrix are for padding.
         *
         * note that the '#' are the actual cells on the grid. They occupy only the center 3x3 grid
         * on the matrix, however. This is because there are some cells at the border for padding.
         * These padding cells are there in order to prevent the need of a division or remainder 
         * calculation when wrapping. In the representation, the padding cells are the Xs. So, when
         * there is a # in one side of the grid, there will be an X in the padding cell on the
         * opposite side.
         *
         */

        /*
        for row in 1..=height {
            // Copy to the column index 0 the cells from the last valid column (column `width`).
            self.cells[row * (width + 2)] = self.cells[row * (width + 2) + width];
            // Copy to the column index `width + 1` the cells from the first valid column (column 1).
            self.cells[row * (width + 2) + width + 1] = self.cells[row * (width + 2) + 1];
        }

        // Copy to the row index 0 the cells from the last valid row (row `height`).
        self.cells[0..width + 2].copy_from_slice(&self.alt_buf[(width + 2) * (height + 1)..(width + 2) * (height + 2)]);
        // Copy to the row index `height + 1` the cells from the first valid row (row 1).
        self.cells[(width + 2) * (height + 1)..(width + 2) * (height + 2)].copy_from_slice(&self.alt_buf[0..width + 2]);
        */

        // self.draw_all();
    }

    #[inline(always)]
    fn get_index(&self, row: u32, column: u32) -> usize {
        // ((row + 1) * (self.width + 2) + column + 1) as usize
        // (row * (self.width + 2) + column) as usize
        (row * self.width + column) as usize
    }

    #[inline(always)]
    fn get_cell(&self, row: u32, col: u32) -> &Cell {
        &self.cells[self.get_index(row, col)]
    }

    #[inline(always)]
    fn get_cell_mut(&mut self, row: u32, col: u32) -> &mut Cell {
        let idx = self.get_index(row, col);
        &mut self.cells[idx]
    }

    fn get_neighbor_count(&self, row: u32, col: u32) -> u8 {
        let mut count = 0;
        for d_row in [self.width - 1, 0, 1] {
            for d_col in [self.height - 1, 0, 1] {
                if d_row == 0 && d_col == 0 { continue };
                // let neighbor_row = row as i32 + d_row;
                // let neighbor_col = col as i32 + d_col;

                let neighbor_row = (row + d_row) % self.width;
                let neighbor_col = (col + d_col) % self.height;

                if self.get_cell(neighbor_row as u32, neighbor_col as u32).is_alive() {
                    count += 1;
                }
            }
        }
        count
    }

    #[inline(always)]
    fn update_neighbors_counts(&mut self, row: u32, col: u32, delta: i8) {
        for d_row in [self.width - 1, 0, 1] {
            for d_col in [self.height - 1, 0, 1] {
                if d_row == 0 && d_col == 0 { continue };
                // let neighbor_row = row as i32 + d_row;
                // let neighbor_col = col as i32 + d_col;

                let neighbor_row = (row + d_row) % self.width;
                let neighbor_col = (col + d_col) % self.height;

                let idx = self.get_index(neighbor_row as u32, neighbor_col as u32);
                let count = self.alt_buf[idx].get_count();
                self.alt_buf[idx].set_count((count as i8 + delta) as u8);
            }
        }
    }


    // Goes through every cell on the list
    pub fn step(&mut self) {
        self.update_list.clear();

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = *self.get_cell(row, col);

                // This means that the cell is dead AND has no neighbors.
                if cell == 0 { continue };

                // let n_neighbors = self.live_neighbor_count(row, col);
                let n_neighbors = cell.get_count();

                let mut has_changed = true;

                let new_cell_state = match cell.is_alive() {
                    Cell::ALIVE if n_neighbors >  3 => Cell::DEAD,
                    Cell::ALIVE if n_neighbors <  2 => Cell::DEAD,
                    Cell::DEAD  if n_neighbors == 3 => Cell::ALIVE,
                    unchaged                        => {
                        has_changed = false;
                        unchaged
                    },
                };

                if has_changed {
                    self.update_list.push(self.get_index(row, col) as u32);

                    let delta = if cell.is_alive() { -1 } else { 1 };
                    self.update_neighbors_counts(row, col, delta);

                    self.draw_cell(row, col, new_cell_state);
                }

                self.alt_buf[idx].set_state(new_cell_state);
            }
        }

        self.cells.copy_from_slice(&self.alt_buf);
    }

    ///! Draws the cell at `row` and `col` according to the `is_alive` parameter. Will not check
    ///! `cells` vector.
    pub fn draw_cell(&mut self, row: u32, col: u32, is_alive: bool) {
        let &mut Grid {
            ref mut img_buf,
            width,
            cell_size,
            ..
        } = self;

        let color = if is_alive {
            ALIVE_COLOR
        } else {
            DEAD_COLOR
        };

        for i in 0..cell_size {
            for j in 0..cell_size {
                let idx = row * width * cell_size.pow(2) // Select the cell row
                        + i * width * cell_size          // Select the pixel row
                        + col * cell_size                // Select the cell column
                        + j;                             // Select the pixel column
                // Each pixel has 4 channels, so we need to multiply that into the index.
                let idx = 4 * idx as usize;
                img_buf[idx..idx + 4].copy_from_slice(&color);
            }
        }
    }

    pub fn draw_all(&mut self) {
        // TODO: Use the update list in order to only redraw what is necessary.

        let mut i = 0;

        for row in 0..self.height {
            for _ in 0..self.cell_size {
                for col in 0..self.width {
                    let color = if self.get_cell(row + 1, col + 1).is_alive() {
                        ALIVE_COLOR
                    } else {
                        DEAD_COLOR
                    };

                    self.img_buf[i..i + 4].copy_from_slice(&color);
                    i += 4;
                }
            }
        }
    }

    pub fn cells(&self) -> *const Cell {
        self.cells.as_ptr()
    }
    
    pub fn n_updated(&self) -> u32 {
        self.update_list.len() as u32
    }

    pub fn updated_list(&self) -> *const u32 {
        self.update_list.as_ptr()
    }

    pub fn img_buf(&self) -> *const u8 {
        self.img_buf.as_ptr()
    }
}

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in self.cells.as_slice().chunks(self.width as usize) {
            for &cell in line {
                let symbol = if cell.is_alive() { '◼' } else { '◻' };
                write!(f, "{}", symbol)?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

#[wasm_bindgen]
pub fn init() {
    utils::set_panic_hook();
}
