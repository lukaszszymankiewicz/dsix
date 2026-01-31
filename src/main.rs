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

#[derive(Clone)]
struct TerminalImage {
    data: String,
    pos_x: isize,
    pos_y: isize,
    rows: usize,
    cols: usize,
    end_x: usize,
    end_y: usize,
}

#[derive(Clone)]
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

impl TerminalImage {
    
    // TODO: some kind of RawImage should be used!
    fn new(data: String, rows: usize, cols: usize, pos_x: isize, pos_y: isize) -> TerminalImage {
        return TerminalImage{
            data: data,
            rows: rows,
            cols: cols,
            pos_x: pos_x,
            pos_y: pos_y,
            end_x: (pos_x + cols as isize) as usize,
            end_y: (pos_y + rows as isize) as usize,
        }
    }
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
    
    fn render_line(&mut self, x: usize, y: usize, line: &str) {
        self.screen.execute(cursor::MoveTo(x as u16, y as u16)).unwrap();
        self.screen.execute(style::Print(line)).unwrap();
    }

    fn render_window(&mut self, wind: &TerminalWindow) {

        for y in 0..wind.cols {
            self.render_line(wind.pos_x - 1, wind.pos_y + y, "@_____________@");
        }

        self.render_line(wind.pos_x - 1, wind.pos_y - 1, "X@@@@@@@@@@@@@X");
        self.render_line(wind.pos_x - 1, wind.pos_y + wind.cols, "X@@@@@@@@@@@@@X");

        for img in &wind.imgs {

            let mut trim_left: isize = 0;
            let mut trim_up: isize = 0;
            let mut trim_right: usize = 0;
            let mut trim_down: usize = 0;

            if img.pos_x < 0 {
                trim_left = img.pos_x * -1;
            } 
            
            if img.pos_y < 0 {
                trim_up = img.pos_y * -1;
            }
            // -7 + 9 = 2
            if img.pos_x + img.cols as isize >= wind.cols as isize {
                trim_right = (img.pos_x as isize + img.cols as isize - wind.cols as isize) as usize;
                assert!((img.pos_x as isize + img.cols as isize - wind.cols as isize)>=0, "img fail");
            }

            if img.pos_y + img.rows as isize >= wind.rows as isize {
                trim_down = (img.pos_y as isize + img.rows as isize - wind.rows as isize) as usize;
                assert!((img.pos_y as isize + img.rows as isize - wind.rows as isize + 1)>=0, "img fail");
            }

            for line in trim_up as usize..(img.rows - trim_down) {
                let left = line*img.rows + trim_left as usize;
                let right = line*img.rows + img.cols - trim_right;
                let img_line = &img.data[left .. right];

                self.render_line(
                    (wind.pos_x as isize + img.pos_x + trim_left) as usize,
                    (wind.pos_y as isize + img.pos_y + line as isize) as usize,
                    img_line
                );
            }
        }
    }

    fn render_all_windows(&mut self) {
        let winds = self.winds.clone(); 

        for wind in &winds {
            self.render_window(wind);
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

// TODO: add maginc numer 9
fn get_map_images(
    hero_pos_x: usize,
    hero_pos_y: usize,
    win_w: usize,
    win_h: usize,
    m: &AMatrix
) -> Vec<TerminalImage> {
    let mut imgs: Vec<TerminalImage> = Vec::new();

    let camera_st_x: usize = hero_pos_x - win_w / 2;
    let camera_end_x: usize = hero_pos_x + win_w / 2;
    let camera_st_y: usize = hero_pos_y - win_h / 2;
    let camera_end_y: usize = hero_pos_y + win_h / 2;

    let st_cell_left: usize = camera_st_x / 9;
    let end_cell_right: usize = camera_end_x / 9;
    let st_cell_up: usize = camera_st_y / 9;
    let end_cell_down: usize = camera_end_y / 9;
    
    // get images and their pos
    for row in st_cell_up..end_cell_down+1 {
        for col in st_cell_left..end_cell_right+1 {

            let x = ((col * 9) as isize) - camera_st_x as isize;
            let y = ((row * 9) as isize) - camera_st_y as isize;

            imgs.push(TerminalImage::new( (&WALLS[m.get(row, col)]).to_string(), 9, 9, x, y));

        }
    }

    return imgs; 
}

fn main() -> Result<(), Box<dyn std::error::Error>>{
    // GAME
    let x: usize = 4;
    let y: usize = 4;
    let mut hero_pos_x: usize = x * 9 + ROOM_GFX_W/2;
    let mut hero_pos_y: usize = y * 9 + ROOM_GFX_H/2;
    let map: AMatrix = generate_map(hero_pos_x, hero_pos_y);

    execute!(stdout(), EnterAlternateScreen, Clear(ClearType::All))?;

    // UI layout
    let mut map_window: TerminalWindow = TerminalWindow::new(13, 13, 4, 4); 
    let mut screen = TerminalScreen::new(20, 20);
    screen.add_window(map_window);

    let sample_img = TerminalImage::new((&WALLS[3]).to_string(), 9, 9, 8, 7);
    let map_imgs = get_map_images(hero_pos_x, hero_pos_y, 13, 13, &map);
    screen.winds[0].push_images(map_imgs);

    // GFX
    enable_raw_mode()?;
    stdout().execute(cursor::Hide).unwrap();

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
// d) add window with level
// e) add window with stats
// f) add method to add images to the level window
// g) implement queue!
// h) add one function for printing!
