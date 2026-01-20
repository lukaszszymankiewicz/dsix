use std::io;
use crossterm::queue;
use std::io::{Write, stdout};
use rand::Rng;
use crossterm::{ExecutableCommand, cursor};
use crossterm::terminal::{ClearType, Clear, enable_raw_mode, disable_raw_mode};
use crossterm::event::{read, Event, KeyEvent, KeyCode};
use crossterm::execute;
use crossterm::style;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};


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

static BASE_ATTACK: usize = 2;
static BASE_ARMOR: usize = 0;
static BASE_EXP: usize = 0;
static BASE_SPEED: usize = 2;

static BASE_LEVEL_W: usize = 8;
static BASE_LEVEL_H: usize = 8;

static BASE: usize = 9;

static Graphics: [RawImage; 26] = [
    RawImage{gfx: "#################################################################################", rows: 9, cols: 9 },
    RawImage{gfx: "###...######...######...######...######...######...##############################", rows: 9, cols: 9 },
    RawImage{gfx: "##############################......###......###......###########################", rows: 9, cols: 9 },
    RawImage{gfx: "###...######...######...######......###......###......###########################", rows: 9, cols: 9 },
    RawImage{gfx: "##############################...######...######...######...######...######...###", rows: 9, cols: 9 },
    RawImage{gfx: "###...######...######...######...######...######...######...######...######...###", rows: 9, cols: 9 },
    RawImage{gfx: "##############################......###......###......###...######...######...###", rows: 9, cols: 9 },
    RawImage{gfx: "###...######...######...######......###......###......###...######...######...###", rows: 9, cols: 9 },
    RawImage{gfx: "###########################......###......###......##############################", rows: 9, cols: 9 },
    RawImage{gfx: "###...######...######...###......###......###......##############################", rows: 9, cols: 9 },
    RawImage{gfx: "###########################...........................###########################", rows: 9, cols: 9 },
    RawImage{gfx: "###...######...######...###...........................###########################", rows: 9, cols: 9 },
    RawImage{gfx: "###########################......###......###......######...######...######...###", rows: 9, cols: 9 },
    RawImage{gfx: "###...######...######...###......###......###......######...######...######...###", rows: 9, cols: 9 },
    RawImage{gfx: "###########################...........................###...######...######...###", rows: 9, cols: 9 },
    RawImage{gfx: "###...######...######...###...........................###...######...######...###", rows: 9, cols: 9 },
    RawImage{gfx: "@", rows: 1, cols: 1 },
    RawImage{gfx: "LEVEL: ", rows: 1, cols: 7 },
    RawImage{gfx: "------------", rows: 1, cols: 11 },
    RawImage{gfx: "ATTACK: ", rows: 1, cols: 8 },
    RawImage{gfx: "ARMOR: ", rows: 1, cols: 7 },
    RawImage{gfx: "SPEED: ", rows: 1, cols: 7 },
    RawImage{gfx: "EXP: ", rows: 1, cols: 5 },
    RawImage{gfx: r"     ______        -'      '-    /            \ |              ||,   -    -   ,|| )(__/  \__)( ||/     /\     \|(_     ^^     _) \__|IIIIII|__/   |-\IIIIII/-|    \          /     `--------`                   ", rows: 13, cols: 16 },
    RawImage{gfx: r"EXPLORATION", rows: 1, cols: 11 },
    RawImage{gfx: "?????????????????????????????????????????????????????????????????????????????????", rows: 9, cols: 9 },
];


struct RawImage {
    gfx: &'static str,
    rows: usize,
    cols: usize,
}

#[derive(Clone)]
struct TerminalImage {
    gfx: String,
    rows: usize,
    cols: usize,
    pos_x: isize,
    pos_y: isize,
    end_x: isize,
    end_y: isize,
}

trait RenderableContent {
    fn render(&self, game: &GameVars, rows: usize, cols: usize) -> Vec<TerminalImage>;
}

struct TerminalWindow {
    imgs: Vec<TerminalImage>,
    content: Box<dyn RenderableContent>,
    rows: usize,
    cols: usize,
    pos_x: usize,
    pos_y: usize,
    vborder: Option<String>,
    hborder: Option<String>,
}

struct TerminalScreen {
    rows: usize,
    cols: usize,
    winds: Vec<TerminalWindow>,
    screen: io::Stdout
}

struct GameVars {
    map: AMatrix,
    visit_map: VMatrix,
    base: usize,
    hero_pos_x: usize,
    hero_pos_y: usize,
    level: usize,
    attack: usize,
    armor: usize,
    speed: usize,
    exp: usize,
}

struct Game {
    screen: TerminalScreen,
    vars: GameVars
}

impl GameVars{
    fn set_st_hero_pos(&mut self) {
        self.map.unblock_all(self.hero_pos_y/self.base, self.hero_pos_x/self.base);
    }

    fn visit_room(&mut self) {
        self.visit_map.set(self.hero_pos_y/self.base, self.hero_pos_x/self.base, 1);
    }
}

impl TerminalImage {
    fn new(idx: usize, pos_x: isize, pos_y: isize) -> TerminalImage {
        let raw = &Graphics[idx];
        
        return TerminalImage{
            gfx: raw.gfx.to_string(),
            rows: raw.rows,
            cols: raw.cols,
            pos_x: pos_x,
            pos_y: pos_y,
            end_x: pos_x as isize + raw.cols as isize,
            end_y: pos_y as isize + raw.rows as isize,
        }
    }

    fn with_text(text: String, pos_x: isize, pos_y: isize) -> TerminalImage {
        let len = text.len();
        TerminalImage {
            gfx: text,
            rows: 1,
            cols: len,
            pos_x,
            pos_y,
            end_x: pos_x + len as isize,
            end_y: pos_y + 1,
        }
    }
}

fn create_borders(cols: usize, bg: char, border: bool) -> (Option<String>, Option<String>) {
    if !border {
        return (None, None);
    }

    let mut h = String::new();
    let mut v = String::new();

    h.push_str("+");
    v.push_str("|");

    for _ in 0..cols {
        h.push_str("-");
        v.push(bg);
    }
    h.push_str("+");
    v.push_str("|");

    (Some(h), Some(v))
}

impl TerminalWindow {

    fn new(
        content: Box<dyn RenderableContent>,
        rows: usize,
        cols: usize,
        pos_x: usize,
        pos_y: usize,
        border: bool,
        bg: char
    ) -> TerminalWindow {
        let (hborder, vborder) = create_borders(cols, bg, border);

        return TerminalWindow{
            imgs: Vec::new(),
            content: content,
            rows: rows,
            cols: cols,
            pos_x: pos_x,
            pos_y: pos_y,
            hborder: hborder,
            vborder: vborder,
        }
    }
    
    fn push_images(&mut self, mut imgs: Vec<TerminalImage>) {
        self.imgs.append(&mut imgs);
    }

    fn clear(&mut self) {
        self.imgs.clear();
    }

}

impl TerminalScreen {
    fn new(rows: usize, cols: usize) -> TerminalScreen {
        return TerminalScreen{
            rows: rows,
            cols: cols,
            winds: Vec::new(),
            screen: io::stdout()
        }
    }

    fn add_window<C: RenderableContent + 'static>(
        &mut self,
        content: C,
        rows: usize,
        cols: usize,
        pos_x: usize,
        pos_y: usize,
        border: bool,
        bg: char
    ) {
        self.winds.push(TerminalWindow::new(Box::new(content), rows, cols, pos_x, pos_y, border, bg));
    }
    
    fn render_window(&mut self, idx: usize) {
        let wind = &mut self.winds[idx];

        if wind.vborder.is_some() && wind.hborder.is_some() {
            for y in 0..wind.rows {
                queue!(self.screen, cursor::MoveTo( (wind.pos_x - 1) as u16, (wind.pos_y + y) as u16)).unwrap();
                queue!(self.screen, style::Print(&wind.vborder.as_ref().unwrap())).unwrap();
            }

            queue!(self.screen, cursor::MoveTo( (wind.pos_x - 1) as u16, (wind.pos_y - 1) as u16)).unwrap();
            queue!(self.screen, style::Print(&wind.hborder.as_ref().unwrap())).unwrap();
            queue!(self.screen, cursor::MoveTo( (wind.pos_x - 1) as u16, (wind.pos_y + wind.rows) as u16)).unwrap();
            queue!(self.screen, style::Print(&wind.hborder.as_ref().unwrap())).unwrap();
        }

        for img in &wind.imgs {

            let mut trim_left: isize = 0;
            let mut trim_up: isize = 0;
            let mut trim_down: usize = 0;
            let mut trim_right: usize = 0;

            if img.pos_x < 0 {
                trim_left = img.pos_x * -1;
            } 
            
            if img.pos_y < 0 {
                trim_up = img.pos_y * -1;
            }
            
            if img.end_x - wind.cols as isize >= 0 {
                trim_right = (img.end_x - wind.cols as isize) as usize;
                // assert!(trim_right <= img.cols, "end={}, cols={}", img.end_x, wind.cols);
            }

            if img.end_y - wind.rows as isize >= 0 {
                trim_down = (img.end_y - wind.rows as isize) as usize;
                // assert!(trim_down <= img.rows);
            }

            for line in trim_up as usize..(img.rows - trim_down) {
                let left = line*img.cols + trim_left as usize;
                let right = line*img.cols + img.cols - trim_right;
                
                if left < right && right <= img.gfx.len() {
                    let img_line = &img.gfx[left .. right];

                    queue!(self.screen, cursor::MoveTo(
                            (wind.pos_x as isize + img.pos_x + trim_left) as u16,
                            (wind.pos_y as isize + img.pos_y + line as isize) as u16,
                        )).unwrap();
                    queue!(self.screen, style::Print(img_line)).unwrap();
                }
            }
        }
    }

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

struct VMatrix {
    matrix: Matrix,
}

impl Matrix {
    fn new(rows: usize, cols: usize, fill: usize) -> Matrix {
        return Matrix{
            rows: rows,
            cols: cols,
            data: vec![fill; cols * rows],
        };
    }

    fn get(&self, row: usize, col: usize) -> usize {
        return self.data[self.rows * row + col];
    }

    fn set(&mut self, row: usize, col: usize, val: usize) { 
        self.data[self.rows * row + col] = val;
    }
}

impl VMatrix {
    fn new(w: usize, h: usize, start_x: usize, start_y: usize) -> VMatrix {
        let mut v: VMatrix = VMatrix{
            matrix: Matrix::new(h, w, 0)
        };

        v.visit(start_x, start_y);

        return v;
    }

    fn get(&self, row: usize, col: usize) -> usize {
        return self.matrix.get(row, col);
    }

    fn set(&mut self, row: usize, col: usize, val: usize) { 
        self.matrix.set(row, col, val);
    }

    fn visit(&mut self, row: usize, col: usize) { 
        self.set(row, col, 1);
    }

    fn forget(&mut self, row: usize, col: usize) { 
        self.set(row, col, 0);
    }
}

impl AMatrix {
    fn new(width: usize, height: usize) -> AMatrix {

        let mut map = AMatrix{
            matrix: Matrix::new(height, width, EMPTY),
            rng: rand::rng(),
        };
        
        // block vertical borders
        for col in 0..width {
            map.block_all(0, col);
            map.block_all(height-1, col);
        }

        // block horizontal borders
        for row in 0..height {
            map.block_all(row, 0);
            map.block_all(row, width-1);
        }
        
        // generate random paths
        for row in 0..height {
            for col in 0..width {
                map.block_random(row, col);
            }
        }

        return map;
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


struct MapWindowContent;

impl RenderableContent for MapWindowContent {
    fn render(&self, game: &GameVars, rows: usize, cols: usize) -> Vec<TerminalImage> {
        let win_h = rows;
        let win_w = cols;
        let mut imgs = Vec::new();

        let camera_st_x = if game.hero_pos_x >= win_w / 2 { game.hero_pos_x - win_w / 2 } else { 0 };
        let camera_end_x = game.hero_pos_x + win_w / 2;
        let camera_st_y = if game.hero_pos_y >= win_h / 2 { game.hero_pos_y - win_h / 2 } else { 0 };
        let camera_end_y = game.hero_pos_y + win_h / 2;

        let st_cell_left = camera_st_x / game.base;
        let end_cell_right = camera_end_x / game.base;
        let st_cell_up = camera_st_y / game.base;
        let end_cell_down = camera_end_y / game.base;
        
        // get wall images (0..16) and their pos
        for row in st_cell_up..end_cell_down+1 {
            for col in st_cell_left..end_cell_right+1 {
                
                let x_pad: isize = (game.hero_pos_x as isize) - (win_w / 2) as isize;
                let y_pad: isize = (game.hero_pos_y as isize) - (win_h / 2) as isize;

                let x = ((col * game.base) as isize) - x_pad;
                let y = ((row * game.base) as isize) - y_pad;

                let idx: usize = if game.visit_map.get(row, col) == 0 {
                    25
                } else {
                    game.map.get(row, col)
                };

                imgs.push(TerminalImage::new(idx, x, y));
            }
        }

        // Hero
        let pos_x = cols as isize / 2;
        let pos_y = rows as isize / 2;
        imgs.push(TerminalImage::new(16, pos_x, pos_y));

        imgs 
    }
}

struct StatWindowContent;

impl RenderableContent for StatWindowContent {
    fn render(&self, game: &GameVars, _rows: usize, _cols: usize) -> Vec<TerminalImage> {
        let mut imgs = Vec::new();

        // Labels
        imgs.push(TerminalImage::new(17, 1, 1)); // LEVEL:
        imgs.push(TerminalImage::new(18, 1, 2)); // 
        imgs.push(TerminalImage::new(19, 1, 3)); // ATTACK:
        imgs.push(TerminalImage::new(20, 1, 4)); // ARMOR:
        imgs.push(TerminalImage::new(21, 1, 5)); // SPEED:
        imgs.push(TerminalImage::new(22, 1, 6)); // EXP:

        // Values
        imgs.push(TerminalImage::with_text(game.level.to_string(), 10, 1));
        imgs.push(TerminalImage::with_text(game.attack.to_string(), 10, 3));
        imgs.push(TerminalImage::with_text(game.armor.to_string(), 10, 4));
        imgs.push(TerminalImage::with_text(game.speed.to_string(), 10, 5));
        imgs.push(TerminalImage::with_text(game.exp.to_string(), 10, 6));

        imgs 
    }
}

struct SkullWindowContent;
impl RenderableContent for SkullWindowContent {
    fn render(&self, _game: &GameVars, _rows: usize, _cols: usize) -> Vec<TerminalImage> {
        let mut imgs = Vec::new();
        imgs.push(TerminalImage::new(23, 2, 0));
        imgs 
    }
}

struct BannerWindowContent;

impl RenderableContent for BannerWindowContent {
    fn render(&self, _game: &GameVars, _rows: usize, _cols: usize) -> Vec<TerminalImage> {
        let mut imgs = Vec::new();
        imgs.push(TerminalImage::new(24, 32, 0));
        imgs 
    }
}

struct LogWindowContent;
impl RenderableContent for LogWindowContent {
    fn render(&self, _game: &GameVars, _rows: usize, _cols: usize) -> Vec<TerminalImage> {
        Vec::new() // Empty for now
    }
}

impl Game {
    fn new(screen_w: usize, screen_h: usize, base: usize) -> Game {
        let level: usize = 1;
        let base: usize = base;
        
        let w = BASE_LEVEL_W + level;
        let h = BASE_LEVEL_H + level;

        let hero_pos_x: usize = w/2 * base + base/2;
        let hero_pos_y: usize = h/2 * base + base/2;
        
        let map: AMatrix = AMatrix::new(w, h);
        let visit_map: VMatrix = VMatrix::new(w, h, w/2, h/2);

        let mut game: Game =  Game {
            screen: TerminalScreen::new(screen_w, screen_h),
            vars: GameVars {
                map: map,
                visit_map: visit_map,
                base: base,
                hero_pos_x: hero_pos_x,
                hero_pos_y: hero_pos_y,
                level: level,
                attack: BASE_ATTACK,
                armor: BASE_ARMOR,
                speed: BASE_SPEED,
                exp: BASE_EXP
            }
        };
        game.vars.set_st_hero_pos();

        return game;
    }
    
    fn prepare_pysical_terminal(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        execute!(&self.screen.screen, EnterAlternateScreen)?;
        execute!(&self.screen.screen, Clear(ClearType::All))?;
        execute!(&self.screen.screen, cursor::Hide)?;
        enable_raw_mode()?;
        Ok(())
    }

    fn leave_pysical_terminal(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        execute!(&self.screen.screen, LeaveAlternateScreen)?;
        disable_raw_mode()?; 
        execute!(&self.screen.screen, cursor::Show)?;
        Ok(())
    }

    fn render(&mut self) {
        for w in 0..self.screen.winds.len() {
            {
                let wind = &mut self.screen.winds[w];
                wind.clear();
                let imgs = wind.content.render(&self.vars, wind.rows, wind.cols);
                wind.push_images(imgs);
            }
            self.screen.render_window(w);
        }
    }

    fn flush_screen(&mut self) {
        self.screen.screen.flush();
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>>{
    // GAME
    let mut game: Game = Game::new(20, 20, 9);     
    game.prepare_pysical_terminal();

    // UI layout
    game.screen.add_window(SkullWindowContent, 13, 20, 2, 6, true, ' ');
    game.screen.add_window(MapWindowContent, 13, 30, 24, 6, true, '.');
    game.screen.add_window(StatWindowContent, 13, 15, 24+30+2, 6, true, ' ');
    game.screen.add_window(BannerWindowContent, 1, 69, 2, 3, true, ' ');
    game.screen.add_window(LogWindowContent, 3, 69, 2, 21, true, ' ');
    
    // update
    game.render();
    
    // game loop
    loop {
        match read() {
            Ok(k) => match k {
                // TODO: after each movemnet, check if entering new cell
                Event::Key(KeyEvent{code: KeyCode::Up, ..}) => game.vars.hero_pos_y -= 1,
                Event::Key(KeyEvent{code: KeyCode::Down, ..}) => game.vars.hero_pos_y += 1,
                Event::Key(KeyEvent{code: KeyCode::Right, ..}) => game.vars.hero_pos_x += 1,
                Event::Key(KeyEvent{code: KeyCode::Left, ..}) => game.vars.hero_pos_x -= 1,
                _ => break
            },
            Err(_) => todo!(),
        };

        game.vars.visit_room();
        game.render();
        game.flush_screen();
    }

    game.leave_pysical_terminal();
    Ok(())
}


// TODO:
// h) add GameState
// j) basic movement
// j) add start and exit to the level
// k) add new level after entering the exit!
