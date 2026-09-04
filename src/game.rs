use std::collections::VecDeque;
use std::io::{Write};

use crossterm::{cursor, execute};
use crossterm::terminal::{ClearType, Clear, enable_raw_mode, disable_raw_mode};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};

use crate::gfx::{TerminalScreen};
use crate::dir::{Dir};
use crate::dungeon::Dungeon;


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
    pub dungeon: Dungeon,
    pub hero_pos_x: usize,
    pub hero_pos_y: usize,
    pub attack: usize,
    pub armor: usize,
    pub speed: usize,
    pub exp: usize,
    pub logs: VecDeque<String>,
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
