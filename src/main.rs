use std::io::{stdout};

use rand::Rng;
use crossterm::{ExecutableCommand, cursor};
use crossterm::terminal::{ClearType, Clear, enable_raw_mode, disable_raw_mode};
use crossterm::event::{read, Event, KeyEvent, KeyCode};
use crossterm::execute;
use crossterm::style;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen, size};

fn refresh_screen() {
    stdout().execute(Clear(ClearType::All)).unwrap();
}

#[derive(Copy, Clone)]
enum Dir {
    up = 0,
    right = 1,
    down = 2,
    left = 3,
}

impl Dir {
    fn opposite(&self) -> Dir {
        match self {
            Dir::left => Dir::right,
            Dir::right => Dir::left,
            Dir::up => Dir::down,
            Dir::down => Dir::up,
        }
    }
}

static LEVEL_HEIGHT: usize = 9;
static LEVEL_WIDTH: usize = 9;
static BLOCK: usize = 0;
static EMPTY: usize = 0b00001111;
static START: usize = 0b11111;
static EXIT: usize = 0b101111;
static BLOCK_CHANCE: usize = 5;
static ROOM_GFX_W: usize = 9;
static ROOM_GFX_H: usize = 9;
static WIN_GFX_W: usize = 13;
static WIN_GFX_H: usize = 13;

static WALLS: [&str; 16] = [
    "#################################################################################",
    "###...######...######...######...######...######...##############################",
    "##############################......###......###......###########################",
    "###...######...######...######......###......###......###########################",
    "##############################...######...######...######...######...######...###",
    "###...######...######...######...######...######...######...######...######...###",
    "##############################......###......###......###...######...######...###",
    "###...######...######...######......###......###......###...######...######...###",
    "###########################......###......###......##############################",
    "###...######...######...###......###......###......##############################",
    "###########################...........................###########################",
    "###...######...######...###...........................###########################",
    "###########################......###......###......######...######...######...###",
    "###...######...######...###......###......###......######...######...######...###",
    "###########################...........................###...######...######...###",
    "###...######...######...###...........................###...######...######...###",
];

struct Matrix {
    rows: usize,
    cols: usize,
    data: Vec<usize>
}

struct AMatrix {
    matrix: Matrix,
    rng: rand::rngs::ThreadRng,
}

struct TWindow {
    matrix: Matrix,
    rows: usize,
    cols: usize,
}

impl Matrix {
    fn new(rows: usize, cols: usize) -> Matrix {
        return Matrix{
            rows: rows,
            cols: cols,
            data: vec![EMPTY; cols * rows],
        };
    }

    fn get(&self, row: usize, col: usize) -> usize {
        return self.data[self.rows * row + col];
    }

    fn set(&mut self, row: usize, col: usize, val: usize) { 
        self.data[self.rows * row + col] = val;
    }
}

impl AMatrix {
    fn new() -> AMatrix {
        return AMatrix{
            matrix: Matrix::new(LEVEL_HEIGHT, LEVEL_WIDTH),
            rng: rand::rng(),
        };
    }

    fn get(&self, row: usize, col: usize) -> usize {
        return self.matrix.get(row, col);
    }

    fn set(&mut self, row: usize, col: usize, val: usize) { 
        self.matrix.set(row, col, val);
    }

    fn modify_and(&mut self, row: usize, col: usize, val: usize) { 
        let mut new = self.get(row, col);
        new &= !(1 << val);
        self.set(row, col, new);
    }

    fn modify_or(&mut self, row: usize, col: usize, val: usize) { 
        let mut new = self.get(row, col);
        new |= 1 << val;
        self.set(row, col, new);
    }

    fn get_neighbor(&self, row: usize, col: usize, dir: Dir) -> Option<(usize, usize)> {
        match dir {
            Dir::left if col > 0 => Some((row, col - 1)),
            Dir::right if col < self.matrix.cols - 1 => Some((row, col + 1)),
            Dir::up if row > 0 => Some((row - 1, col)),
            Dir::down if row < self.matrix.rows - 1 => Some((row + 1, col)),
            _ => None,
        }
    }

    fn block(&mut self, row: usize, col: usize, dir: Dir) {
        self.modify_and(row, col, dir as usize);

        if let Some((n_row, n_col)) = self.get_neighbor(row, col, dir) {
            self.modify_and(n_row, n_col, dir.opposite() as usize);
        }
    }

    fn unblock(&mut self, row: usize, col: usize, dir: Dir) {
        self.modify_or(row, col, dir as usize);

        if let Some((n_row, n_col)) = self.get_neighbor(row, col, dir) {
            self.modify_or(n_row, n_col, dir.opposite() as usize);
        }
    }

    fn block_all(&mut self, row: usize, col: usize) {
        for dir in [Dir::left, Dir::right, Dir::up, Dir::down] {
            self.block(row, col, dir);
        }
    }
    
    fn unblock_all(&mut self, row: usize, col: usize) {
        for dir in [Dir::left, Dir::right, Dir::up, Dir::down] {
            self.unblock(row, col, dir);
        }
    }

    fn random_cell(&mut self) -> (usize, usize) {
        return (
            self.rng.random_range(1..self.matrix.rows-1),
            self.rng.random_range(1..self.matrix.cols-1)
        );
    }

    fn block_random(&mut self, row: usize, col: usize) {
        let v = self.get(row, col);

        // Check if fully blocked (assuming 0 means all WALLS blocked)
        // Adjust these masks based on your specific bit logic
        if v & 0b1111 == 0 { return; } 

        if self.rng.random_range(0..10) < BLOCK_CHANCE {
            let choice = self.rng.random_range(0..6);
            match choice {
                0 => self.block(row, col, Dir::left),
                1 => self.block(row, col, Dir::right),
                2 => self.block(row, col, Dir::down),
                3 => self.block(row, col, Dir::up),
                4 => {
                    self.block(row, col, Dir::up);
                    self.block(row, col, Dir::down);
                },
                5 => {
                    self.block(row, col, Dir::right);
                    self.block(row, col, Dir::left);
                }
                _ => unreachable!()
            }
        }
    }
}

fn render_map() -> String {
    // level generation
    let mut m: AMatrix = AMatrix::new();
    let mut buf: String = String::new();

    for col in 0..LEVEL_WIDTH {
        m.block_all(0, col);
        m.block_all(LEVEL_HEIGHT-1, col);
    }

    for row in 0..LEVEL_HEIGHT {
        m.block_all(row, 0);
        m.block_all(row, LEVEL_WIDTH-1);
    }

    // random start cell
    // loop {
    //     let (sx, sy) = m.random_cell();
    //     let (ex, ey) = m.random_cell();

    //     if (ex != sx) | (ey != sy) {
    //         // m.set(sx, sy, START);
    //         // m.set(ex, ey, EXIT);
    //         break;
    //     }
    // }

    for row in 0..LEVEL_HEIGHT {
        for col in 0..LEVEL_WIDTH {
            m.block_random(row, col);
            // m.block_random(row, col);
        }
    }
    //hero state 
    let x: usize = 4;
    let y: usize = 4;

    m.unblock_all(x, y);

    let xp: usize = x * ROOM_GFX_W + ROOM_GFX_W/2;
    let yp: usize = y * ROOM_GFX_H + ROOM_GFX_H/2;
    
    let camera_st_x: usize = xp - WIN_GFX_W / 2;
    let camera_st_y: usize = yp - WIN_GFX_W / 2;
    let camera_end_x: usize = xp + WIN_GFX_H / 2;
    let camera_end_y: usize = yp + WIN_GFX_H / 2;

    let st_cell_left: usize = camera_st_x / ROOM_GFX_W;
    let end_cell_right: usize = camera_end_x / ROOM_GFX_W;
    let st_cell_up: usize = camera_st_y / ROOM_GFX_H;
    let end_cell_down: usize = camera_end_y / ROOM_GFX_H;

    // render gfx
    for row in st_cell_up..end_cell_down+1 {
        let mut trim_left: usize = 0;
        let mut trim_up: usize = 0;
        let mut trim_right: usize = ROOM_GFX_W;
        let mut trim_down: usize = ROOM_GFX_H;

        if ((row * ROOM_GFX_H) < camera_st_y) & (camera_st_y < ((row+1) * ROOM_GFX_H)) {
            trim_up = camera_st_y - (row * ROOM_GFX_H);
        }

        if ((row * ROOM_GFX_H) < camera_end_y) & (camera_end_y < ((row+1) * ROOM_GFX_H)) {
            trim_down = camera_end_y - (row * ROOM_GFX_H) + 1;
        }

        for i in (0+trim_up)..trim_down {
            for col in st_cell_left..end_cell_right+1 {

                if ((col * ROOM_GFX_W) <=camera_st_x) & (camera_st_x < ((col+1) * ROOM_GFX_W)) {
                    trim_left = camera_st_x - (col * ROOM_GFX_W);
                }

                if ((col * ROOM_GFX_W) <= camera_end_x) & (camera_end_x < ((col+1) * ROOM_GFX_W)) {
                    trim_right = camera_end_x - (col * ROOM_GFX_W) + 1;
                }
                
                // this function translates availability matrix to walls representation
                // for one draw line only
                let left_bound = (i*ROOM_GFX_W) + trim_left;
                let right_bound = (i*ROOM_GFX_W) + trim_right;
                let line = vec!(&WALLS[m.get(row, col)] [left_bound .. right_bound]);

                buf.push_str(&WALLS[m.get(row, col)][left_bound .. right_bound]);

                trim_left = 0;
                trim_right = 9;
            }
        }
    }
    return buf; 
}

fn main() -> Result<(), Box<dyn std::error::Error>>{
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen, Clear(ClearType::All))?;

    // This is the start window coords
    let x = 10;
    let y = 6;
    // 126, 39
    // window -> 80, 30
    // println!("terminal size: {:?}", size().unwrap());

    stdout().execute(cursor::MoveTo(x, y)).unwrap();
    let map: String = render_map();
    
    for winy in 0..WIN_GFX_H {
        stdout().execute(style::Print(&map[winy*WIN_GFX_W..(winy+1)*WIN_GFX_W])).unwrap();
        stdout().execute(cursor::MoveTo(x, y+winy as u16)).unwrap();
    }

    loop {
        match read() {
            Ok(k) => match k {
                Event::Key(KeyEvent{code: KeyCode::Up, ..}) => stdout().execute(cursor::MoveUp(1)),
                Event::Key(KeyEvent{code: KeyCode::Down, ..}) => stdout().execute(cursor::MoveDown(1)),
                Event::Key(KeyEvent{code: KeyCode::Right, ..}) => stdout().execute(cursor::MoveRight(1)),
                Event::Key(KeyEvent{code: KeyCode::Left, ..}) => stdout().execute(cursor::MoveLeft(1)),
                Event::Key(KeyEvent{code: KeyCode::Char('k'), ..}) => break,
                _ => break
            },
            Err(_) => todo!(),
        };
        refresh_screen();
    }

    execute!(stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?; 
    Ok(())
}

// TODO:
// check moving
// wrap rendering level into somekind of Window
// Window should have method to add somekind of image/text on in
// add window with level
// add window with stats
// add method to add images to the level window
//
