use std::collections::VecDeque;
use std::io::{Write};

use crossterm::{cursor, execute};
use crossterm::terminal::{ClearType, Clear, enable_raw_mode, disable_raw_mode};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};

use crate::gfx::{TerminalScreen, TerminalImage, RenderableContent, GFX_IDX};
use crate::dir::{Dir};
use crate::dungeon::Dungeon;

use crate::dungeon::BASE_ROOM_SIZE;


static BASE_ARMOR                  : usize = 0;
static BASE_SPEED                  : usize = 2;
static BASE_ATTACK                 : usize = 2;
static BASE_EXP                    : usize = 0;
static LOG_SIZE                    : usize = 3;

pub struct Game {
    pub screen: TerminalScreen,
    pub vars: GameVars,
}

pub struct GameVars {
    dungeon: Dungeon,
    hero_pos_x: usize,
    hero_pos_y: usize,
    attack: usize,
    armor: usize,
    speed: usize,
    exp: usize,
    logs: VecDeque<String>,
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
                
                let idx: usize = vars.dungeon.get_the_image_idx_of_a_room(row, col);

                // FOG OF WAR
                if !vars.dungeon.room_is_visited(row, col) {
                    imgs.push(TerminalImage::new(GFX_IDX::CORRIDOR_UNKNOWN, x, y));
                } else {
                    imgs.push(TerminalImage::new(idx, x, y));
                }

                // if there is no entity to render, just follow along
                if vars.dungeon.room_is_visited(row, col) {
                    continue
                }

                for idx in 0..vars.dungeon.get_n_entities_in_room(row, col) {

                    let ent = vars.dungeon.get_entity_in_room_by_idx(row, col, idx);

                    let entity_render_x = ent.entity_pos_x as isize - x_pad;
                    let entity_render_y = ent.entity_pos_y as isize - y_pad;
                    let img = ent.entity_type.img_idx;

                    imgs.push(TerminalImage::new(img, entity_render_x, entity_render_y));
                };
            }
        }

        // Hero
        let pos_x = win_w as isize / 2;
        let pos_y = win_h as isize / 2;

        imgs.push(TerminalImage::new(GFX_IDX::ENTITY_HERO, pos_x, pos_y));
        
        return imgs;
    }
}

pub struct DebugMapWindowContent;
impl RenderableContent for DebugMapWindowContent {
    fn render(&self, vars: &mut GameVars, _rows: usize, _cols: usize) -> Vec<TerminalImage> {
        
        let mut imgs = Vec::new();

        for row in 0..vars.dungeon.rows {
            for col in 0..vars.dungeon.cols {

                let idx: usize = vars.dungeon.get_the_image_idx_of_a_room(row, col);

                imgs.push(TerminalImage::with_debug_text(idx, col.try_into().unwrap(), row.try_into().unwrap()));
            }
        }

        // Hero
        imgs.push(TerminalImage::new(
            GFX_IDX::ENTITY_HERO,
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
        imgs.push(TerminalImage::with_text("LEVEL".to_string(), 1, 1));
        imgs.push(TerminalImage::with_text("ATTACK".to_string(), 1, 3));
        imgs.push(TerminalImage::with_text("ARMOR".to_string(), 1, 4));
        imgs.push(TerminalImage::with_text("SPEED".to_string(), 1, 5));
        imgs.push(TerminalImage::with_text("EXP".to_string(), 1, 6));
        imgs.push(TerminalImage::with_text("ROW".to_string(), 1, 7));
        imgs.push(TerminalImage::with_text("COL".to_string(), 1, 8));

        // Values
        imgs.push(TerminalImage::with_text(game.dungeon.level_number.to_string(), 10, 1));
        imgs.push(TerminalImage::with_text(game.attack.to_string(), 10, 3));
        imgs.push(TerminalImage::with_text(game.armor.to_string(), 10, 4));
        imgs.push(TerminalImage::with_text(game.speed.to_string(), 10, 5));
        imgs.push(TerminalImage::with_text(game.exp.to_string(), 10, 6));
        imgs.push(TerminalImage::with_text(game.hero_pos_x.to_string(), 10, 7));
        imgs.push(TerminalImage::with_text(game.hero_pos_y.to_string(), 10, 8));

        imgs 
    }
}

pub struct SkullWindowContent;
impl RenderableContent for SkullWindowContent {
    fn render(&self, _game: &mut GameVars, _rows: usize, _cols: usize) -> Vec<TerminalImage> {
        let mut imgs = Vec::new();
        imgs.push(TerminalImage::new(GFX_IDX::DECORATION_SKULL, 2, 0));
        imgs 
    }
}

pub struct BannerWindowContent;
impl RenderableContent for BannerWindowContent {
    fn render(&self, _game: &mut GameVars, _rows: usize, _cols: usize) -> Vec<TerminalImage> {
        let mut imgs = Vec::new();
        imgs.push(TerminalImage::with_text("EXPLORATION".to_string(), 32, 0));
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
                    
                    // this is just a debug info, it should not be used anywhere else
                    if vars.dungeon.there_is_some_entities_in_the_room(row, col) {
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
                attack: BASE_ATTACK,
                armor: BASE_ARMOR,
                speed: BASE_SPEED,
                exp: BASE_EXP,
                logs: VecDeque::new(),
            }
        };
        (hero_init_pos_x, hero_init_pos_y) = game.vars.dungeon.init();

        game.vars.hero_pos_x = hero_init_pos_x;
        game.vars.hero_pos_y = hero_init_pos_y;

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

    pub fn move_hero(&mut self, dir: Dir) {
        let mut new_x: usize = self.hero_pos_x;
        let mut new_y: usize = self.hero_pos_y;
        let log: String;

        match dir {
            Dir::Left => {new_x -=1; log = "You have moved left".to_string() },
            Dir::Right => {new_x +=1; log = "You have moved right".to_string() },
            Dir::Up => {new_y -=1; log = "You have moved up".to_string() },
            Dir::Down => {new_y +=1; log = "You have moved down".to_string() },
        }

        match self.dungeon.position_is_an_obstacle(new_x, new_y) {
            false =>  {
                self.hero_pos_y = new_y;
                self.hero_pos_x = new_x;
                self.add_log(log);
            },
            true => {
                self.add_log("You went into wall...".to_string());
            }
        }
    }

    pub fn set_the_room_as_visited_if_needed(&mut self) {
        let res: usize  = self.dungeon.visit_the_room_using_entity_pos(self.hero_pos_x, self.hero_pos_y);

        if res == 1 {
            self.add_log("You have entered new room!".to_string());
        }
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
