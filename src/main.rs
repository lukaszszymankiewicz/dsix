mod gfx;
mod game;

use crossterm::event::{KeyCode, KeyEvent, Event};
use crossterm::event::read;
use game::{BannerWindowContent, LogWindowContent, StatWindowContent, MapWindowContent, SkullWindowContent};
use game::{Game, GameVars};

fn main() -> Result<(), Box<dyn std::error::Error>>{
    // GAME
    let mut game: Game = Game::new();     
    let _ = game.prepare_pysical_terminal();

    // UI layout
    game.screen.add_new_window_to_layout(SkullWindowContent, 13, 20, 2, 6, true, ' ');
    game.screen.add_new_window_to_layout(MapWindowContent, 13, 30, 24, 6, true, '.');
    game.screen.add_new_window_to_layout(StatWindowContent, 13, 15, 24+30+2, 6, true, ' ');
    game.screen.add_new_window_to_layout(BannerWindowContent, 1, 69, 2, 3, true, ' ');
    game.screen.add_new_window_to_layout(LogWindowContent, 3, 69, 2, 21, true, ' ');
    
    // game loop
    loop {

        // update
        game.render();
        game.flush_screen();

        // controls
        match read() {
            Ok(k) => match k {
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
