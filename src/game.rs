use std::collections::VecDeque;
use std::io::{Write};

use rand::Rng;
use crossterm::{cursor};
use crossterm::terminal::{ClearType, Clear, enable_raw_mode, disable_raw_mode};
use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};

use crate::gfx::{TerminalScreen, TerminalImage, RenderableContent};

#[derive(Copy, Clone)]
pub enum Dir {
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

pub struct Game {
    pub screen: TerminalScreen,
    pub vars: GameVars,
}

pub struct GameVars {
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

struct Room {
    visited: bool,
    exits: [bool; 4],
    img_idx: usize
}

fn throw_k6_dice() -> usize {
    return rand::rng().random_range(0..6);
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


pub struct MapWindowContent;
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

pub struct StatWindowContent;
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

pub struct SkullWindowContent;
impl RenderableContent for SkullWindowContent {
    fn render(&self, _game: &mut GameVars, _rows: usize, _cols: usize) -> Vec<TerminalImage> {
        let mut imgs = Vec::new();
        imgs.push(TerminalImage::new(23, 2, 0));
        imgs 
    }
}

pub struct BannerWindowContent;
impl RenderableContent for BannerWindowContent {
    fn render(&self, _game: &mut GameVars, _rows: usize, _cols: usize) -> Vec<TerminalImage> {
        let mut imgs = Vec::new();
        imgs.push(TerminalImage::new(24, 32, 0));
        imgs 
    }
}

pub struct LogWindowContent;
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
    pub fn new() -> Game {
        
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
    
    pub fn prepare_pysical_terminal(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        execute!(&self.screen.screen, EnterAlternateScreen)?;
        execute!(&self.screen.screen, Clear(ClearType::All))?;
        execute!(&self.screen.screen, cursor::Hide)?;
        enable_raw_mode()?;
        Ok(())
    }

    pub fn leave_pysical_terminal(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        execute!(&self.screen.screen, LeaveAlternateScreen)?;
        disable_raw_mode()?; 
        execute!(&self.screen.screen, cursor::Show)?;
        Ok(())
    }

    pub fn render(&mut self) {
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

    pub fn flush_screen(&mut self) {
        let _ = self.screen.screen.flush();
    }

}

impl GameVars {
    pub fn move_hero_up(&mut self) -> usize {
        self.hero_pos_y -= 1;
        self.hero_room_y = self.hero_pos_y / BASE_ROOM_SIZE;
        return 1;
    }

    pub fn move_hero_down(&mut self) -> usize {
        self.hero_pos_y += 1;
        self.hero_room_y = self.hero_pos_y / BASE_ROOM_SIZE;
        return 1;
    }

    pub fn move_hero_right(&mut self) -> usize {
        self.hero_pos_x += 1;
        self.hero_room_x = self.hero_pos_x / BASE_ROOM_SIZE;
        return 1;
    }

    pub fn move_hero_left(&mut self) -> usize {
        self.hero_pos_x -= 1;
        self.hero_room_x = self.hero_pos_x / BASE_ROOM_SIZE;
        return 1;
    }
    
    pub fn visit_room_in_current_hero_pos(&mut self) -> usize {
        let res: usize = self.dungeon.set_room_as_visited(self.hero_room_y, self.hero_room_x);
        return res;
    }

    pub fn add_log(&mut self, log: String) {
        self.logs.push_back(log);

        if self.logs.len() > LOG_SIZE {
            self.logs.pop_front(); 
        }
    }

}
