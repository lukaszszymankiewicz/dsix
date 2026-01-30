use std::io;
use std::io::{stdout};

use rand::Rng;
use crossterm::{ExecutableCommand, cursor};
use crossterm::terminal::{ClearType, Clear, enable_raw_mode, disable_raw_mode};
use crossterm::event::{read, Event, KeyEvent, KeyCode};
use crossterm::execute;
use crossterm::style;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};

#[allow(dead_code)]
fn refresh_screen() {
    stdout().execute(Clear(ClearType::All)).unwrap();
}

#[derive(Copy, Clone)]
enum Dir {
    Up = 0,
    Right = 1,
    Down = 2,
    Left = 3,
}

impl Dir {
    fn opposite(&self) -> Dir {
        match self {
            Dir::Left => Dir::Right,
            Dir::Right => Dir::Left,
            Dir::Up => Dir::Down,
            Dir::Down => Dir::Up,
        }
    }
}

static LEVEL_HEIGHT: usize = 9;
static LEVEL_WIDTH: usize = 9;
static EMPTY: usize = 0b00001111;
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

struct TerminalImage {
    data: String,
    pos_x: isize,
    pos_y: isize,
    rows: usize,
    cols: usize,
}

struct TerminalWindow {
    imgs: Vec<TerminalImage>,
    rows: usize,
    cols: usize,
    pos_x: usize,
    pos_y: usize,
}

struct TerminalScreen {
    rows: usize,
    cols: usize,
    n_winds: usize,
    winds: Vec<TerminalWindow>,
    screen: io::Stdout
}

struct Matrix {
    rows: usize,
    cols: usize,
    data: Vec<usize>
}

struct AMatrix {
    matrix: Matrix,
    rng: rand::rngs::ThreadRng,
}

impl TerminalWindow {
    fn new(rows: usize, cols: usize, pos_x: usize, pos_y: usize) -> TerminalWindow {
        return TerminalWindow{
            imgs: Vec::new(),
            rows: rows,
            cols: cols,
            pos_x: pos_x,
            pos_y: pos_y,
        }
    }

    fn push_images(&mut self, imgs: Vec<TerminalImage>) {
        self.imgs = imgs;
    }

    fn push_image(&mut self, img: TerminalImage) {
        self.imgs.push(img);
    }
}

impl TerminalScreen {
    fn new(rows: usize, cols: usize) -> TerminalScreen {
        return TerminalScreen{
            rows: rows,
            cols: cols,
            winds: Vec::new(),
            n_winds: 0,
            screen: io::stdout()
        }
    }

    fn add_window(&mut self, window: TerminalWindow) {
        self.winds.push(window);
    }

    fn render_all_windows(&mut self) {
        for wind in &self.winds {
            
            self.screen.execute(cursor::MoveTo(
                (wind.pos_x - 1) as u16,
                (wind.pos_y - 1) as u16)
            ).unwrap();
            // border
            stdout().execute(style::Print("X-------------X")).unwrap();

            for y in 0..wind.cols {

                self.screen.execute(cursor::MoveTo(
                    (wind.pos_x - 1) as u16,
                    (wind.pos_y + y) as u16)
                ).unwrap();
                stdout().execute(style::Print("|")).unwrap();

                self.screen.execute(cursor::MoveTo(
                    (wind.pos_x + wind.rows) as u16,
                    (wind.pos_y + y) as u16)
                ).unwrap();
                stdout().execute(style::Print("|")).unwrap();
            }

            let mut trim_left: isize = 0;
            let mut trim_up: isize = 0;

            for img in &wind.imgs {

                let mut trim_right: usize = img.cols;
                let mut trim_down: usize = img.rows;

                if img.pos_x < 0 {
                    trim_left = img.pos_x * -1;
                } 
                
                if img.pos_y < 0 {
                    trim_up = img.pos_y * -1;
                }

                if img.pos_x as usize + img.cols > wind.cols {
                    trim_right = (img.pos_x as usize + img.cols as usize) - wind.cols as usize;
                }

                if img.pos_y as usize + img.rows > wind.rows {
                    trim_down = (img.pos_y as usize + img.rows as usize) - wind.rows as usize - 1;
                }

                self.screen.execute(cursor::MoveTo(
                    (wind.pos_x as isize + img.pos_x + trim_left) as u16,
                    (wind.pos_y as isize + img.pos_y + trim_up) as u16)
                ).unwrap();

                for line in 0..(img.rows - trim_down) {
                    stdout().execute(style::Print(
                            &img.data[
                            line*img.rows
                            ..
                            line*img.rows + img.cols - trim_right
                        ])).unwrap();
                    stdout().execute(cursor::MoveTo(
                            (wind.pos_x as isize + img.pos_x + trim_left) as u16,
                            (wind.pos_y as isize + img.pos_y + trim_up + line as isize) as u16)
                    ).unwrap();
                }
            }
            
            // border
            self.screen.execute(cursor::MoveTo(
                (wind.pos_x - 1) as u16,
                (wind.pos_y as isize + wind.cols as isize) as u16)
            ).unwrap();
            stdout().execute(style::Print("X-------------X")).unwrap();
        }
    }
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
            Dir::Left if col > 0 => Some((row, col - 1)),
            Dir::Right if col < self.matrix.cols - 1 => Some((row, col + 1)),
            Dir::Up if row > 0 => Some((row - 1, col)),
            Dir::Down if row < self.matrix.rows - 1 => Some((row + 1, col)),
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
        for dir in [Dir::Left, Dir::Right, Dir::Up, Dir::Down] {
            self.block(row, col, dir);
        }
    }
    
    fn unblock_all(&mut self, row: usize, col: usize) {
        for dir in [Dir::Left, Dir::Right, Dir::Up, Dir::Down] {
            self.unblock(row, col, dir);
        }
    }

    #[allow(dead_code)]
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
                0 => self.block(row, col, Dir::Left),
                1 => self.block(row, col, Dir::Right),
                2 => self.block(row, col, Dir::Down),
                3 => self.block(row, col, Dir::Up),
                4 => {
                    self.block(row, col, Dir::Up);
                    self.block(row, col, Dir::Down);
                },
                5 => {
                    self.block(row, col, Dir::Right);
                    self.block(row, col, Dir::Left);
                }
                _ => unreachable!()
            }
        }
    }
}

fn generate_map(hero_pos_x: usize, hero_pos_y: usize) -> AMatrix {
    let mut map: AMatrix = AMatrix::new();
    
    // block vertical borders
    for col in 0..LEVEL_WIDTH {
        map.block_all(0, col);
        map.block_all(LEVEL_HEIGHT-1, col);
    }

    // block horizontal borders
    for row in 0..LEVEL_HEIGHT {
        map.block_all(row, 0);
        map.block_all(row, LEVEL_WIDTH-1);
    }
    
    // generate random paths
    for row in 0..LEVEL_HEIGHT {
        for col in 0..LEVEL_WIDTH {
            map.block_random(row, col);
        }
    }

    // hero position should be transalted to cell
    map.unblock_all(hero_pos_x/ROOM_GFX_W, hero_pos_y/ROOM_GFX_H);

    return map;
}

fn get_map_image(
    hero_pos_x: usize,
    hero_pos_y: usize,
    win_w: usize,
    win_h: usize,
    m: &AMatrix
) -> Vec<TerminalImage> {
    let mut imgs: Vec<TerminalImage> = Vec::new();

    let camera_st_x: usize = hero_pos_x - win_w / 2;
    let camera_st_y: usize = hero_pos_y - win_w / 2;
    let camera_end_x: usize = hero_pos_x + win_h / 2;
    let camera_end_y: usize = hero_pos_y + win_h / 2;

    let st_cell_left: usize = camera_st_x / win_w;
    let end_cell_right: usize = camera_end_x / win_w;
    let st_cell_up: usize = camera_st_y / win_h;
    let end_cell_down: usize = camera_end_y / win_h;
    
    // get images and their pos
    for row in st_cell_up..end_cell_down+1 {

        for col in st_cell_left..end_cell_right+1 {

            imgs.push(
                TerminalImage{
                    data: (&WALLS[m.get(row, col)]).to_string(),
                    rows: 9,
                    cols: 9,
                    pos_x: ((col * ROOM_GFX_W) as isize) - camera_st_x as isize,
                    pos_y: ((row * ROOM_GFX_H) as isize) - camera_st_y as isize
                }
            );

        }
    }

    return imgs; 
}

fn render_map(hero_pos_x: usize, hero_pos_y: usize, m: &AMatrix) -> String {
    let mut buf: String = String::new();

    let camera_st_x: usize = hero_pos_x - WIN_GFX_W / 2;
    let camera_st_y: usize = hero_pos_y - WIN_GFX_W / 2;
    let camera_end_x: usize = hero_pos_x + WIN_GFX_H / 2;
    let camera_end_y: usize = hero_pos_y + WIN_GFX_H / 2;

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
                let line = &WALLS[m.get(row, col)][left_bound .. right_bound];

                buf.push_str(line);

                trim_left = 0;
                trim_right = 9;
            }
        }
    }
    return buf; 
}

fn main() -> Result<(), Box<dyn std::error::Error>>{
    // GAME
    let x: usize = 4;
    let y: usize = 4;
    let mut hero_pos_x: usize = x * ROOM_GFX_W + ROOM_GFX_W/2;
    let mut hero_pos_y: usize = y * ROOM_GFX_H + ROOM_GFX_H/2;

    let map: AMatrix = generate_map(hero_pos_x, hero_pos_y);

    // UI layout
    let mut map_window: TerminalWindow = TerminalWindow::new(13, 13, 4, 4); 
    let mut screen = TerminalScreen::new(20, 20);

    screen.add_window(map_window);

    let sample_img = TerminalImage{data: (&WALLS[3]).to_string(), rows: 9, cols: 9, pos_x: 8, pos_y: 7 };
    screen.winds[0].push_image(sample_img);

    // GFX
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen, Clear(ClearType::All))?;
    stdout().execute(cursor::Hide).unwrap();

    // This is the start window coords
    let map_window_pos_x: u16 = 10;
    let map_window_pos_y: u16 = 6;

    screen.render_all_windows();

    loop {
        match read() {
            Ok(k) => match k {
                Event::Key(KeyEvent{code: KeyCode::Up, ..}) => hero_pos_y-=1,
                Event::Key(KeyEvent{code: KeyCode::Down, ..}) => hero_pos_y+=1,
                Event::Key(KeyEvent{code: KeyCode::Right, ..}) => hero_pos_x+=1,
                Event::Key(KeyEvent{code: KeyCode::Left, ..}) => hero_pos_x-=1,
                _ => break
            },
            Err(_) => todo!(),
        };

    }

    execute!(stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?; 
    stdout().execute(cursor::Show).unwrap();
    Ok(())
}

// TODO:
// b) wrap rendering level into somekind of Window
// c) Window should have method to add somekind of image/text on in
// d) add window with level
// e) add window with stats
// f) add method to add images to the level window
// g) implement queue!
// h) add one function for printing!
