use std::collections::VecDeque;
use std::io::{Write};
use std::io;
use crate::scenes::MainDungeonScene;
use crossterm::{cursor, execute};
use crossterm::terminal::{ClearType, Clear, enable_raw_mode, disable_raw_mode};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use crate::RenderableScene;
use crate::gfx::Layout;
use crate::dir::{Dir};
use crate::dungeon::Dungeon;

static BASE_ARMOR                  : usize = 0;
static BASE_SPEED                  : usize = 2;
static BASE_ATTACK                 : usize = 2;
static BASE_EXP                    : usize = 0;
static LOG_SIZE                    : usize = 3;

pub enum ControlSignal {
    DoNothingBitchSlap,
    ExitGame,
    ChangeScene,
}

pub struct Game {
    pub layout: Layout,
    pub scene: Box<dyn RenderableScene>,
    pub vars: GameVars,
    pub screen_output: io::Stdout,
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
    pub fn new(initial_scene: Box<dyn RenderableScene>) -> Game {
        
        let hero_init_pos_x: usize;
        let hero_init_pos_y: usize;
        
        let mut game: Game =  Game {
            layout: Layout::new(),
            screen_output: io::stdout(),
            scene: initial_scene,
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

        game.update_the_layout();

        return game;
    }
    
    pub fn prepare_pysical_terminal(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        execute!(&self.screen_output, EnterAlternateScreen)?;
        execute!(&self.screen_output, Clear(ClearType::All))?;
        execute!(&self.screen_output, cursor::Hide)?;
        enable_raw_mode()?;
        Ok(())
    }

    pub fn leave_pysical_terminal(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        execute!(&self.screen_output, LeaveAlternateScreen)?;
        disable_raw_mode()?; 
        execute!(&self.screen_output, cursor::Show)?;
        Ok(())
    }

    pub fn clear_pysical_terminal(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        execute!(&self.screen_output, Clear(ClearType::All))?;
        Ok(())
    }

    pub fn render_current_scene(&mut self) {
        for w in 0..self.layout.windows.len() {
            {
                let wind = &mut self.layout.windows[w];
                wind.clear();
                let imgs = wind.content.render(&mut self.vars, wind.rows, wind.cols);
                wind.push_images(imgs);
            }

            self.layout.render_single_window(w, &mut self.screen_output);
        }
    }

    pub fn flush_screen(&mut self) {
        let _ = self.screen_output.flush();
    }

    pub fn update_the_layout(&mut self) {
        self.layout.windows.clear();
        self.clear_pysical_terminal();
        self.scene.fill_the_layout(&mut self.layout);
    }

    pub fn run_game_loop(&mut self) {

        let _ = self.prepare_pysical_terminal();

        loop {

            // update
            self.render_current_scene();
            self.flush_screen();

            // control
            match self.scene.correspond_to_controls(&mut self.vars) {
                ControlSignal::DoNothingBitchSlap => (),
                ControlSignal::ExitGame => break,
                ControlSignal::ChangeScene => {
                    self.scene = self.scene.dispatch_scene(&mut self.vars);
                    self.update_the_layout();
                }
            }
        }

        let _ = self.leave_pysical_terminal();
    }
}

impl GameVars {

    pub fn move_hero(&mut self, dir: Dir) -> ControlSignal {
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
        
        // check if new room is visited
        let new_room_is_visited: usize = self.dungeon.visit_the_room_using_entity_pos(self.hero_pos_x, self.hero_pos_y);

        if new_room_is_visited == 1 {
            self.add_log("You have entered new room!".to_string());
        }

        return ControlSignal::DoNothingBitchSlap;
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
