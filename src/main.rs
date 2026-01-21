use std::io::{stdout};
use rand::Rng;
use crossterm::{ExecutableCommand, cursor};
use crossterm::terminal::{ClearType, Clear, enable_raw_mode, disable_raw_mode};
use crossterm::event::{read, Event, KeyEvent, KeyCode};
use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};

fn refresh_screen() {
    stdout().execute(Clear(ClearType::All)).unwrap();
}

enum Dir {
    up = 0,
    right = 1,
    down = 2,
    left = 3,
}

static LEVEL_HEIGHT: usize = 21;
static LEVEL_WIDTH: usize = 21;
static BLOCK: usize = 0;
static EMPTY: usize = 0b00001111;
static START: usize = 0b11111;
static EXIT: usize = 0b101111;

struct Matrix {
    rows: usize,
    cols: usize,
    data: Vec<usize>
}

struct AMatrix {
    matrix: Matrix,
    rng: rand::rngs::ThreadRng,
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
            matrix: Matrix::new( LEVEL_HEIGHT, LEVEL_WIDTH ),
            rng: rand::rng(),
        };
    }

    fn get(&self, row: usize, col: usize) -> usize {
        return self.matrix.get(row, col);
    }

    fn set(&mut self, row: usize, col: usize, val: usize) { 
        self.matrix.set(row, col, val);
    }

    fn block_one(&mut self, row: usize, col: usize, dir: Dir) {
        let mut new = self.get(row, col);
        new = new & !(1 << (dir as usize));
        self.set(row, col, new);
    }

    fn block_all(&mut self, row: usize, col: usize) {
        // block the direct block
        self.set(row, col, BLOCK);

        // neighbours
        // left
        if col > 0 {
            self.block_one(row, col - 1, Dir::right);
        }

        // right
        if col < self.matrix.cols - 1 {
            self.block_one(row, col+1, Dir::left);
        }

        // down
        if row < self.matrix.rows - 1 {
            self.block_one(row + 1, col, Dir::up);
        }

        // up
        if row > 0 {
            self.block_one(row-1, col, Dir::down);
        }
    }
    
    fn random_cell(&mut self) -> (usize, usize) {
        return (
            self.rng.random_range(1..self.matrix.rows-1),
            self.rng.random_range(1..self.matrix.cols-1)
        );
    }
}

fn main() -> () {
    let mut rng: rand::rngs::ThreadRng = rand::rng();
    println!("{:#?}", rng);

    let mut m: AMatrix = AMatrix::new();

    for col in 0..LEVEL_WIDTH {
        m.block_all(0, col);
        m.block_all(LEVEL_HEIGHT-1, col);
    }

    for row in 0..LEVEL_HEIGHT {
        m.block_all(row, 0);
        m.block_all(row, LEVEL_WIDTH-1);
    }

    loop {
        let (sx, sy) = m.random_cell();
        let (ex, ey) = m.random_cell();

        if (ex != sx) | (ey != sy) {
            m.set(sx, sy, START);
            m.set(ex, ey, EXIT);
            break;
        }
    }


    // START, EXIT
    // debug print - availability matrix is just an abstract
    for row in 0..LEVEL_HEIGHT {
        for col in 0..LEVEL_WIDTH {
            match m.get(row, col) {
                0b0000 => print!("X"),
                0b0011 => print!("└"),
                0b0101 => print!("|"),
                0b1001 => print!("┘"),
                0b0110 => print!("┌"),
                0b1010 => print!("-"),
                0b1100 => print!("┐"),

                0b0111 => print!("├"),
                0b1011 => print!("┴"),
                0b1110 => print!("┬"),
                0b1101 => print!("┤"),

                0b1111 => print!("┼"),
                0b11111 => print!("S"),
                0b101111 => print!("E"),
                _ => print!("."),
            }
        }
        print!("\n");
    }
}

fn terminal() -> Result<(), Box<dyn std::error::Error>>{
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen, Clear(ClearType::All))?;
    let x = 5;
    let y = 5;

    stdout().execute(cursor::MoveTo(x, y)).unwrap();

    loop {
        match read() {
            Ok(k) => match k {
                Event::Key(KeyEvent{code: KeyCode::Up, ..}) => stdout().execute(cursor::MoveUp(1)),
                Event::Key(KeyEvent{code: KeyCode::Down, ..}) => stdout().execute(cursor::MoveDown(1)),
                Event::Key(KeyEvent{code: KeyCode::Right, ..}) => stdout().execute(cursor::MoveRight(1)),
                Event::Key(KeyEvent{code: KeyCode::Left, ..}) => stdout().execute(cursor::MoveLeft(1)),
                Event::Key(KeyEvent{code: KeyCode::Char('k'), ..}) => break,
                _ => todo!()
            },
            Err(_) => todo!(),
        };
        refresh_screen();
    }

    execute!(stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?; 
    println!("raw mode disabled!");
    Ok(())
}
