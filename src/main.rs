use std::io;
use crossterm::queue;

use std::collections::VecDeque;
use std::io::{Write};

use rand::Rng;
use crossterm::{cursor};
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

static CHANCE_FOR_ROOM_TO_HAVE_ANY_EXIT_BLOCKED: usize = 3;

static BASE_ATTACK: usize = 2;
static BASE_ARMOR: usize = 0;
static BASE_EXP: usize = 0;
static BASE_SPEED: usize = 2;
static LOG_SIZE: usize = 3;

static BASE_LEVEL_W: usize = 8;
static BASE_LEVEL_H: usize = 8;
static BASE_ROOM_SIZE: usize = 9;

static GRAPHICS: [RawImage; 26] = [
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
    fn render(&self, game: &mut GameVars, rows: usize, cols: usize) -> Vec<TerminalImage>;
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
    winds: Vec<TerminalWindow>,
    screen: io::Stdout
}

struct GameVars {
    dungeon: Dungeon,
    hero_pos_x: usize,
    hero_pos_y: usize,
    hero_room_x: usize,
    hero_room_y: usize,
    attack: usize,
    armor: usize,
    speed: usize,
    exp: usize,
    logs: VecDeque<String>,
}

struct Game {
    screen: TerminalScreen,
    vars: GameVars,
}

impl TerminalImage {
    fn new(idx: usize, pos_x: isize, pos_y: isize) -> TerminalImage {
        let raw = &GRAPHICS[idx];
        
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
    fn new() -> TerminalScreen {
        return TerminalScreen{
            winds: Vec::new(),
            screen: io::stdout()
        }
    }

    fn add_new_window_to_layout<C: RenderableContent + 'static>(
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


fn throw_k6_dice() -> usize {
    return rand::rng().random_range(0..6);
}

struct Room {
    visited: bool,
    exits: [bool; 4],
    img_idx: usize
}

impl Room {
    fn new() -> Room {
        let new_room: Room = Room{
            visited: false,
            exits: [true; 4],
            img_idx: 25
        };

        return new_room;
    }

    fn update_the_associated_img_idx(&mut self) {
        let mut idx: usize = 0b0000_0000;

        for (index, bit) in self.exits.iter().enumerate().rev() {
            idx += (*bit as usize) << index;
        }

        self.img_idx = idx;
    }

    fn visit(&mut self) -> usize {
        if self.visited == true {
            return 0;
        }
        self.visited = true; 
        self.update_the_associated_img_idx();
        return 1;
    }

    // fn unvisit(&mut self) {
    //     self.visited = false; 
    // }

    fn block_exit(&mut self, dir: Dir) {
        self.exits[dir as usize] = false; 
    }

    fn unblock_exit(&mut self, dir: Dir) {
        self.exits[dir as usize] = true; 
    }

}

struct Dungeon {
    level_number: usize,
    rows: usize,
    cols: usize,
    rooms: Vec<Room>
}

impl Dungeon {
    fn new(level_number: usize) -> Dungeon {
        let new_dungeon_cols = BASE_LEVEL_W + level_number;
        let new_dungeon_rows = BASE_LEVEL_H + level_number;
        let rooms_total = new_dungeon_cols * new_dungeon_rows;
        let mut new_rooms :Vec<Room> = Vec::new();

        for _ in 0..rooms_total {
            new_rooms.push(Room::new());
        }

        let new_dungeon = Dungeon {
            level_number: level_number,
            rows: new_dungeon_rows,
            cols: new_dungeon_cols,
            rooms: new_rooms
        };
        
        return new_dungeon;
    }

    fn init(&mut self, init_row: usize, init_col: usize) -> (usize, usize) {
        // block vertical borders of whole level
        for col in 0..self.cols {
            self.block_all_exists_from_room(0, col);
            self.block_all_exists_from_room(self.rows-1, col);
        }

        // block horizontal borders of whole level
        for row in 0..self.rows {
            self.block_all_exists_from_room(row, 0);
            self.block_all_exists_from_room(row, self.rows-1);
        }

        // generate random paths
        for row in 0..self.rows {
            for col in 0..self.cols {
                self.block_random_exit_of_room(row, col);
            }
        }
        
        self.unblock_all_exists_from_room(init_row, init_col);
        self.set_room_as_visited(init_row, init_col);

        // return the starting point of the hero, based on init room
        return (
            init_row * BASE_ROOM_SIZE + BASE_ROOM_SIZE/2,
            init_col * BASE_ROOM_SIZE + BASE_ROOM_SIZE/2
        );
    }

    fn get_room(&mut self, col: usize, row: usize) -> &mut Room {
        return self.rooms.get_mut(self.rows * row + col).unwrap()
    }

    fn get_neighbor_room_coords(&self, row: usize, col: usize, dir: Dir) -> Option<(usize, usize)> {
        match dir {
            Dir::Left if col > 0 => Some((row, col - 1)),
            Dir::Right if col < self.cols - 1 => Some((row, col + 1)),
            Dir::Up if row > 0 => Some((row - 1, col)),
            Dir::Down if row < self.rows - 1 => Some((row + 1, col)),
            _ => None,
        }
    }

    fn block_single_exit_from_room(&mut self, row: usize, col: usize, dir: Dir) {
        self.get_room(row, col).block_exit(dir);

        if let Some((neighboour_row, neighbour_col)) = self.get_neighbor_room_coords(row, col, dir) {
            self.get_room(neighboour_row, neighbour_col).block_exit(dir.opposite());
        }
    }

    fn block_all_exists_from_room(&mut self, row: usize, col: usize) {
        for dir in [Dir::Left, Dir::Right, Dir::Up, Dir::Down] {
            self.block_single_exit_from_room(row, col, dir);
        }
    }

    fn unblock_single_exit_from_room(&mut self, row: usize, col: usize, dir: Dir) {
        self.get_room(row, col).unblock_exit(dir);

        if let Some((neighboour_row, neighbour_col)) = self.get_neighbor_room_coords(row, col, dir) {
            self.get_room(neighboour_row, neighbour_col).unblock_exit(dir.opposite());
        }
    }
    
    fn unblock_all_exists_from_room(&mut self, row: usize, col: usize) {
        for dir in [Dir::Left, Dir::Right, Dir::Up, Dir::Down] {
            self.unblock_single_exit_from_room(row, col, dir);
        }
    }

    fn block_random_exit_of_room(&mut self, row: usize, col: usize) {
        let exits = self.get_room(row, col).exits;

        // nothing more to block
        if exits == [false; 4] {
            return; 
        } 

        if throw_k6_dice() > CHANCE_FOR_ROOM_TO_HAVE_ANY_EXIT_BLOCKED {
            return; 
        }

        match throw_k6_dice() {
            0 => self.block_single_exit_from_room(row, col, Dir::Left),
            1 => self.block_single_exit_from_room(row, col, Dir::Right),
            2 => self.block_single_exit_from_room(row, col, Dir::Down),
            3 => self.block_single_exit_from_room(row, col, Dir::Up),
            4 => {
                self.block_single_exit_from_room(row, col, Dir::Up);
                self.block_single_exit_from_room(row, col, Dir::Down);
            },
            5 => {
                self.block_single_exit_from_room(row, col, Dir::Right);
                self.block_single_exit_from_room(row, col, Dir::Left);
            }
            _ => unreachable!()
        }
    }

    fn set_room_as_visited(&mut self, row: usize, col: usize) -> usize {
        return self.get_room(row, col).visit();
    }
}


struct MapWindowContent;

impl RenderableContent for MapWindowContent {
    fn render(&self, vars: &mut GameVars, rows: usize, cols: usize) -> Vec<TerminalImage> {
        let win_h = rows;
        let win_w = cols;
        let mut imgs = Vec::new();

        let camera_st_x = if vars.hero_pos_x >= win_w / 2 { vars.hero_pos_x - win_w / 2 } else { 0 };
        let camera_end_x = vars.hero_pos_x + win_w / 2;
        let camera_st_y = if vars.hero_pos_y >= win_h / 2 { vars.hero_pos_y - win_h / 2 } else { 0 };
        let camera_end_y = vars.hero_pos_y + win_h / 2;

        let st_cell_left = camera_st_x / BASE_ROOM_SIZE;
        let end_cell_right = camera_end_x / BASE_ROOM_SIZE;
        let st_cell_up = camera_st_y / BASE_ROOM_SIZE;
        let end_cell_down = camera_end_y / BASE_ROOM_SIZE;
        
        // get wall images (0..16) and their pos
        for row in st_cell_up..end_cell_down+1 {
            for col in st_cell_left..end_cell_right+1 {
                
                let x_pad: isize = (vars.hero_pos_x as isize) - (win_w / 2) as isize;
                let y_pad: isize = (vars.hero_pos_y as isize) - (win_h / 2) as isize;

                let x = ((col * BASE_ROOM_SIZE) as isize) - x_pad;
                let y = ((row * BASE_ROOM_SIZE) as isize) - y_pad;

                let idx: usize = vars.dungeon.get_room(row, col).img_idx;
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
    fn render(&self, game: &mut GameVars, _rows: usize, _cols: usize) -> Vec<TerminalImage> {
        let mut imgs = Vec::new();

        // Labels
        imgs.push(TerminalImage::new(17, 1, 1)); // LEVEL:
        imgs.push(TerminalImage::new(18, 1, 2)); // 
        imgs.push(TerminalImage::new(19, 1, 3)); // ATTACK:
        imgs.push(TerminalImage::new(20, 1, 4)); // ARMOR:
        imgs.push(TerminalImage::new(21, 1, 5)); // SPEED:
        imgs.push(TerminalImage::new(22, 1, 6)); // EXP:

        // Values
        imgs.push(TerminalImage::with_text(game.dungeon.level_number.to_string(), 10, 1));
        imgs.push(TerminalImage::with_text(game.attack.to_string(), 10, 3));
        imgs.push(TerminalImage::with_text(game.armor.to_string(), 10, 4));
        imgs.push(TerminalImage::with_text(game.speed.to_string(), 10, 5));
        imgs.push(TerminalImage::with_text(game.exp.to_string(), 10, 6));

        imgs 
    }
}

struct SkullWindowContent;
impl RenderableContent for SkullWindowContent {
    fn render(&self, _game: &mut GameVars, _rows: usize, _cols: usize) -> Vec<TerminalImage> {
        let mut imgs = Vec::new();
        imgs.push(TerminalImage::new(23, 2, 0));
        imgs 
    }
}

struct BannerWindowContent;
impl RenderableContent for BannerWindowContent {
    fn render(&self, _game: &mut GameVars, _rows: usize, _cols: usize) -> Vec<TerminalImage> {
        let mut imgs = Vec::new();
        imgs.push(TerminalImage::new(24, 32, 0));
        imgs 
    }
}

struct LogWindowContent;
impl RenderableContent for LogWindowContent {
    fn render(&self, vars: &mut GameVars, _rows: usize, _cols: usize) -> Vec<TerminalImage> {
        let log_size: isize = 2;
        let mut i: isize = 0;
        
        let mut imgs = Vec::new();
        for s in vars.logs.iter().rev() { 

            if i > log_size { break; }

            imgs.push(TerminalImage::with_text(s.to_string(), 0, log_size-i));
            i+=1;
        }

        return imgs;
    }
}

impl Game{
    fn new() -> Game {
        
        let hero_init_pos_x: usize;
        let hero_init_pos_y: usize;

        let mut game: Game =  Game {
            screen: TerminalScreen::new(),
            vars: GameVars {
                dungeon: Dungeon::new(1),
                hero_pos_x: 0,
                hero_pos_y: 0,
                hero_room_x: 0,
                hero_room_y: 0,
                attack: BASE_ATTACK,
                armor: BASE_ARMOR,
                speed: BASE_SPEED,
                exp: BASE_EXP,
                logs: VecDeque::new(),
            }
        };

        (hero_init_pos_x, hero_init_pos_y) = game.vars.dungeon.init(game.vars.dungeon.rows/2, game.vars.dungeon.cols/2);

        game.vars.hero_pos_x = hero_init_pos_x;
        game.vars.hero_pos_y = hero_init_pos_y;

        game.vars.hero_room_x = hero_init_pos_x / BASE_ROOM_SIZE;
        game.vars.hero_room_y = hero_init_pos_y / BASE_ROOM_SIZE;

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
                let imgs = wind.content.render(&mut self.vars, wind.rows, wind.cols);
                wind.push_images(imgs);
            }

            self.screen.render_window(w);
        }
    }

    fn flush_screen(&mut self) {
        let _ = self.screen.screen.flush();
    }

}

impl GameVars {
    fn move_hero_up(&mut self) -> usize {
        self.hero_pos_y -= 1;
        self.hero_room_y = self.hero_pos_y / BASE_ROOM_SIZE;
        return 1;
    }

    fn move_hero_down(&mut self) -> usize {
        self.hero_pos_y += 1;
        self.hero_room_y = self.hero_pos_y / BASE_ROOM_SIZE;
        return 1;
    }

    fn move_hero_right(&mut self) -> usize {
        self.hero_pos_x += 1;
        self.hero_room_x = self.hero_pos_x / BASE_ROOM_SIZE;
        return 1;
    }

    fn move_hero_left(&mut self) -> usize {
        self.hero_pos_x -= 1;
        self.hero_room_x = self.hero_pos_x / BASE_ROOM_SIZE;
        return 1;
    }
    
    fn visit_room_in_current_hero_pos(&mut self) -> usize {
        let res: usize = self.dungeon.set_room_as_visited(self.hero_room_y, self.hero_room_x);
        return res;
    }

    fn add_log(&mut self, log: String) {
        self.logs.push_back(log);

        if self.logs.len() > LOG_SIZE {
            self.logs.pop_front(); 
        }
    }

}
fn main() -> Result<(), Box<dyn std::error::Error>>{
    // GAME
    let mut game: Game = Game::new();     
    let _ = game.prepare_pysical_terminal();

    // UI layout
    game.screen.add_new_window_to_layout(SkullWindowContent, 13, 20, 2, 6, true, ' ');
    game.screen.add_new_window_to_layout(MapWindowContent, 13, 30, 24, 6, true, '.');
    game.screen.add_new_window_to_layout(StatWindowContent, 13, 15, 24+30+2, 6, true, ' ');
    game.screen.add_new_window_to_layout(BannerWindowContent, 1, 69, 2, 3, true, ' ');
    game.screen.add_new_window_to_layout(LogWindowContent, 3, 69, 2, 21, true, '.');
    
    // game loop
    loop {

        // update
        game.render();
        game.flush_screen();

        // controls
        match read() {
            Ok(k) => match k {
                // TODO: after each movemnet, check if entering new cell
                Event::Key(KeyEvent{code: KeyCode::Up, ..}) => {
                    let _ = game.vars.move_hero_up();
                    game.vars.add_log("You have moved up...".to_string());
                }
                Event::Key(KeyEvent{code: KeyCode::Down, ..}) => {
                    let _ = game.vars.move_hero_down();
                    game.vars.add_log("You have moved down...".to_string());
                }
                Event::Key(KeyEvent{code: KeyCode::Right, ..}) => {
                    let _ = game.vars.move_hero_right();
                    game.vars.add_log("You have moved right...".to_string());
                }
                Event::Key(KeyEvent{code: KeyCode::Left, ..}) => {
                    let _ = game.vars.move_hero_left();
                    game.vars.add_log("You have moved left...".to_string());
                }
                _ => break
            },
            Err(_) => todo!(),
        };
        
        // check the room
        let res = game.vars.visit_room_in_current_hero_pos();

        if res == 1 {
            game.vars.add_log("You have entered new room!".to_string());
        }
    }

    let _ = game.leave_pysical_terminal();

    Ok(())
}
