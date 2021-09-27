#![allow(dead_code)]

mod utils;

use std::fmt;

use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

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
        let mut cells: Vec<Cell> = (0..width * height)
            .map(|i| {
                if i % 2 == 0 || i % 7 == 0 {
                    Cell::alive()
                } else {
                    Cell::dead()
                }
            })
            .collect();

        // This is a very heavy operation, but it should only need to be performed in some rare
        // occasions such as this one.
        for row in 0..height {
            for col in 0..width {
                let mut count = 0;
                for d_row in [width - 1, 0, 1] {
                    for d_col in [height - 1, 0, 1] {
                        if d_row == 0 && d_col == 0 { continue };
                        let neighbor_row = (row + d_row) % width;
                        let neighbor_col = (col + d_col) % height;
                        let neighbor_idx = neighbor_row * width + neighbor_col;
                        if cells[neighbor_idx as usize].is_alive() {
                            count += 1;
                        }
                    }
                }
                cells[(row * width + col) as usize].set_count(count);
            }
        }

        let alt_buf = cells.clone();
        let update_list = (0..width * height).collect();
        let img_buf = (0..width * height * cell_size * cell_size)
            .flat_map(|_| [0x00, 0x00, 0x00, 0xff])
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

        grid.draw_all();
        grid
    }

    #[inline(always)]
    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }

    #[inline(always)]
    fn get_cell(&self, row: u32, col: u32) -> Cell {
        self.cells[self.get_index(row, col)]
    }

    fn live_neighbor_count(&self, row: u32, col: u32) -> u8 {
        let mut count = 0;

        for d_row in [self.width - 1, 0, 1] {
            for d_col in [self.height - 1, 0, 1] {
                if d_row == 0 && d_col == 0 { continue };
                let neighbor_row = (row + d_row) % self.width;
                let neighbor_col = (col + d_col) % self.height;
                let neighbor_idx = neighbor_row * self.width + neighbor_col;
                if self.cells[neighbor_idx as usize].is_alive() {
                    count += 1;
                }
            }
        }

        count
    }

    // Goes through every cell on the list
    pub fn step(&mut self) {
        self.update_list.clear();

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.get_cell(row, col);

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

                    // This should be faster than checking at every iteration of the loop if the
                    // cell was alive.
                    let update_fun = if cell.is_alive() { Cell::dec_count } else { Cell::inc_count };

                    for d_row in [self.width - 1, 0, 1] {
                        for d_col in [self.height - 1, 0, 1] {
                            if d_row == 0 && d_col == 0 { continue };
                            let neighbor_row = (row + d_row) % self.width;
                            let neighbor_col = (col + d_col) % self.height;
                            let neighbor_idx = self.get_index(neighbor_row, neighbor_col);
                            update_fun(&mut self.alt_buf[neighbor_idx]);
                        }
                    }

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
        let &mut Grid {
            // ref mut update_list,
            ref mut img_buf,
            ref cells,
            width,
            height,
            cell_size,
            ..
        } = self;

        // TODO: Use the update list in order to only redraw what is necessary.

        let mut i = 0;

        for row in 0..height {
            for _ in 0..cell_size {
                for col in 0..width {
                    let color = if cells[(row * width + col) as usize].is_alive() {
                        ALIVE_COLOR
                    } else {
                        DEAD_COLOR
                    };

                    for _ in 0..cell_size {
                        img_buf[i    ] = color[0];
                        img_buf[i + 1] = color[1];
                        img_buf[i + 2] = color[2];
                        img_buf[i + 3] = color[3];
                        i += 4;
                    }
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
