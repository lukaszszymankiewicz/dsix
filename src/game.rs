use std::collections::VecDeque;
use std::io::{Write};
use rand::seq::IndexedRandom;

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

pub struct EntityType(pub usize, pub usize);
pub const EntityStairsToLowerLevel: EntityType = EntityType(0, 28);

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

static MAX_LEN_OF_PATH_TO_THE_EXIT: usize = 6;

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

struct Entity {
    entity_type: EntityType,
    entity_pos_x: usize,
    entity_pos_y: usize
}

struct Room {
    visited: bool,
    exits: [bool; 4],
    img_idx: usize,
    entities: Vec<Entity>
}

fn throw_k6_dice() -> usize {
    return rand::rng().random_range(0..6);
}

impl Room {
    fn new() -> Room {
        let new_room: Room = Room{
            visited: false,
            exits: [true; 4],
            img_idx: 28,
            entities: Vec::new()
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

struct Path {
    steps: Vec<(usize, usize)>,
    max_len: usize,
    possible_next_steps: Vec<Dir>
}

impl Path {
    fn new(initial_x: usize, initial_y: usize) -> Path {
        let new_path = Path {
            steps: vec![(initial_x, initial_y)],
            max_len: MAX_LEN_OF_PATH_TO_THE_EXIT,
            possible_next_steps: Vec::new()
        };
        return new_path;
    }

    fn last_step (&mut self) -> Option<(usize, usize)> {
        match self.steps.len() {
            0 => None,
            n => Some(self.steps[n-1])
        }
    }

    fn add_step_to_fixed_direction(&mut self, direction: Dir) {
        let last_step: (usize, usize) = self.last_step().unwrap();

        match direction {
            Dir::Left => {
                self.steps.push((last_step.0, last_step.1-1));
            }
            Dir::Right => {
                self.steps.push((last_step.0, last_step.1+1));
            }
            Dir::Up => {
                self.steps.push((last_step.0-1, last_step.1));
            }
            Dir::Down => {
                self.steps.push((last_step.0+1, last_step.1));
            }
        }
        
        self.possible_next_steps.clear();
    }

    fn check_possible_next_steps(&mut self, exits: [bool; 4]) {
        self.possible_next_steps.clear();

        let last_step = self.last_step();
        let last_step_x: usize = last_step.unwrap().0;
        let last_step_y: usize = last_step.unwrap().1;
        
        //urdl 

        if exits[3] == true & !self.steps.contains(&(last_step_x, last_step_y-1)) {
            self.possible_next_steps.push(Dir::Left);
        }

        if exits[1] == true & !self.steps.contains(&(last_step_x, last_step_y+1)) {
            self.possible_next_steps.push(Dir::Right);
        }

        if exits[0] == true & !self.steps.contains(&(last_step_x-1, last_step_y)) {
            self.possible_next_steps.push(Dir::Up);
        }
        
        if exits[2] == true & !self.steps.contains(&(last_step_x+1, last_step_y)) {
            self.possible_next_steps.push(Dir::Down);
        }

    }

    fn choose_random_possible_step(&mut self) -> Dir  {
        return *self.possible_next_steps.choose(&mut rand::rng()).unwrap();
    }

}

impl Dungeon {
    fn new(level_number: usize) -> Dungeon {
        let new_dungeon_cols = BASE_LEVEL_W + level_number;
        let new_dungeon_rows = BASE_LEVEL_H + level_number;
        let rooms_total = new_dungeon_cols * new_dungeon_rows;
        let mut new_rooms: Vec<Room> = Vec::new();
        
        // Dungeon coords start from upper-left corner
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
        // block horizontal borders of whole level
        for col in 0..self.cols {
            self.block_all_exists_from_room(0, col);
            self.block_all_exists_from_room(self.rows-1, col);
        }

        // block vertical borders of whole level
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
        self.spawn_an_entity_stairs_to_lower_level(init_row, init_col);

        // return the starting point of the hero, based on init room
        return (
            init_row * BASE_ROOM_SIZE + BASE_ROOM_SIZE/2,
            init_col * BASE_ROOM_SIZE + BASE_ROOM_SIZE/2
        );
    }

    fn spawn_an_entity(&mut self, entity_type: EntityType, entity_pos_x: usize, entity_pos_y: usize) {
        let new_entity = Entity {
            entity_type: entity_type,
            entity_pos_x: entity_pos_x,
            entity_pos_y: entity_pos_y
        };
        
        // add it to the room
        let entity_room_row: usize = entity_pos_y / BASE_ROOM_SIZE;
        let entity_room_col: usize = entity_pos_x / BASE_ROOM_SIZE;

        self.get_room(entity_room_row, entity_room_col).entities.push(new_entity);
    }

    fn spawn_an_entity_stairs_to_lower_level(&mut self, initial_x: usize, initial_y: usize) {
        let mut new_path: Path = Path::new(initial_x, initial_y);

        // initally - all direction are possible, so any way can be chosen
        new_path.add_step_to_fixed_direction(Dir::Up);

        let mut last_step_x: usize;
        let mut last_step_y: usize;

        loop {

            let last_step = new_path.last_step();
            last_step_x = last_step.unwrap().0;
            last_step_y = last_step.unwrap().1;

            let possible_exits = self.get_room(last_step_x, last_step_y).exits;
            new_path.check_possible_next_steps(possible_exits);

            if new_path.possible_next_steps.len() == 0 || new_path.steps.len() >= new_path.max_len {
                break; 
            }

            let new_dir: Dir = new_path.choose_random_possible_step();
            new_path.add_step_to_fixed_direction(new_dir);
        }

        let exit_room = new_path.last_step();
        let exit_room_row = exit_room.unwrap().0;
        let exit_room_col = exit_room.unwrap().1;
        
        // place exit on the middle of a room
        let entity_pos_x = (exit_room_col * BASE_ROOM_SIZE) + (BASE_ROOM_SIZE / 2);
        let entity_pos_y = (exit_room_row * BASE_ROOM_SIZE) + (BASE_ROOM_SIZE / 2);

        self.spawn_an_entity(EntityStairsToLowerLevel, entity_pos_x, entity_pos_y);

    }

    fn get_room(&mut self, row: usize, col: usize) -> &mut Room {
        // rooms coords start from uppper-left corner of the Dungeon
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

        // self.block_all_exists_from_room(row, 0);
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

    fn unblock_all_exists_from_room(&mut self, row: usize, col: usize) {
        for dir in [Dir::Left, Dir::Right, Dir::Up, Dir::Down] {
            self.unblock_single_exit_from_room(row, col, dir);
        }
    }

    fn unblock_single_exit_from_room(&mut self, row: usize, col: usize, dir: Dir) {
        self.get_room(row, col).unblock_exit(dir);

        if let Some((neighboour_row, neighbour_col)) = self.get_neighbor_room_coords(row, col, dir) {
            self.get_room(neighboour_row, neighbour_col).unblock_exit(dir.opposite());
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
    
    // needed only for debug
    fn set_all_rooms_as_visited(&mut self) {
        for row in 0..self.rows {
            for col in 0..self.cols {
                let _ = self.get_room(row, col).visit();
            }
        }
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
        
        // get room images and their position on the window
        // x_pad and y_pad are calculated to know if window slices the image (from left and up).
        // sliced image is not fully rendered (obviously)
        for row in st_cell_up..end_cell_down+1 {
            for col in st_cell_left..end_cell_right+1 {
                
                let x_pad: isize = (vars.hero_pos_x as isize) - (win_w / 2) as isize;
                let y_pad: isize = (vars.hero_pos_y as isize) - (win_h / 2) as isize;

                let x = ((col * BASE_ROOM_SIZE) as isize) - x_pad;
                let y = ((row * BASE_ROOM_SIZE) as isize) - y_pad;

                let idx: usize = vars.dungeon.get_room(row, col).img_idx;
                imgs.push(TerminalImage::new(idx, x, y));

                // EXIT
                let n_entities_in_room = vars.dungeon.get_room(row, col).entities.len();

                for entity_idx in 0..n_entities_in_room {

                    let ent = &vars.dungeon.get_room(row, col).entities[entity_idx];

                    let entity_render_x = ent.entity_pos_x as isize - x_pad;
                    let entity_render_y = ent.entity_pos_y as isize - y_pad;

                    // let entity_render_x = (col * BASE_ROOM_SIZE) as isize - x_pad + 4;
                    // let entity_render_y = (row * BASE_ROOM_SIZE) as isize - y_pad + 4;

                    imgs.push(TerminalImage::new(25, entity_render_x, entity_render_y));
                };
            }
        }

        // Hero
        let pos_x = win_w as isize / 2;
        let pos_y = win_h as isize / 2;

        imgs.push(TerminalImage::new(16, pos_x, pos_y));
        
        return imgs;
    }
}

pub struct DebugMapWindowContent;
impl RenderableContent for DebugMapWindowContent {
    fn render(&self, vars: &mut GameVars, _rows: usize, _cols: usize) -> Vec<TerminalImage> {
        
        let mut imgs = Vec::new();

        for row in 0..vars.dungeon.rows {
            for col in 0..vars.dungeon.cols {

                let idx: usize = vars.dungeon.get_room(row, col).img_idx;

                imgs.push(TerminalImage::with_debug_text(idx, col.try_into().unwrap(), row.try_into().unwrap()));
            }
        }

        // Hero
        imgs.push(TerminalImage::new(
            16,
            (vars.dungeon.cols/2).try_into().unwrap(),
            (vars.dungeon.rows/2).try_into().unwrap())
        );

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
        imgs.push(TerminalImage::new(26, 1, 7)); // ROW:
        imgs.push(TerminalImage::new(27, 1, 8)); // COL:

        // Values
        imgs.push(TerminalImage::with_text(game.dungeon.level_number.to_string(), 10, 1));
        imgs.push(TerminalImage::with_text(game.attack.to_string(), 10, 3));
        imgs.push(TerminalImage::with_text(game.armor.to_string(), 10, 4));
        imgs.push(TerminalImage::with_text(game.speed.to_string(), 10, 5));
        imgs.push(TerminalImage::with_text(game.exp.to_string(), 10, 6));
        imgs.push(TerminalImage::with_text(game.hero_room_x.to_string(), 10, 7));
        imgs.push(TerminalImage::with_text(game.hero_room_y.to_string(), 10, 8));

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
        

        // inital logs
        if vars.logs.len() == 0 {

            for row in 0..vars.dungeon.rows {
                for col in 0..vars.dungeon.cols {
                        
                    let mut exit_coord_string: String = String::new();
                    exit_coord_string.push_str(&col.to_string());
                    exit_coord_string.push(',');
                    exit_coord_string.push_str(&row.to_string());
                    
                    // pos of exit here!
                    if vars.dungeon.get_room(row, col).entities.len() > 0 {
                        imgs.push(TerminalImage::with_text(exit_coord_string, 0, 0));
                    }
                }
            }
        }

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

    pub fn visit_all_rooms(&mut self) {
        self.dungeon.set_all_rooms_as_visited();
    }

    pub fn add_log(&mut self, log: String) {
        self.logs.push_back(log);

        if self.logs.len() > LOG_SIZE {
            self.logs.pop_front(); 
        }
    }
}


// TODO:
// add collision with walls
// add collision with entity
// delete exit coords from the start log window
