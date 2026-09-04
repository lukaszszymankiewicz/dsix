mod dir;
mod dice;
mod dungeon;
mod entity;
mod game;
mod gfx;

use crossterm::event::{KeyCode, KeyEvent, Event, read};
use crate::game::{
    DebugMapWindowContent,
    BannerWindowContent,
    LogWindowContent,
    StatWindowContent,
    MapWindowContent,
    SkullWindowContent,
    Game,
};
use std::env;

use crate::dir::Dir;

fn prepare_ui(game: &mut Game) -> usize {
    game.screen.add_new_window_to_layout(MapWindowContent, 13, 30, 24, 6, true, '.');
    game.screen.add_new_window_to_layout(SkullWindowContent, 13, 20, 2, 6, true, ' ');
    game.screen.add_new_window_to_layout(StatWindowContent, 13, 15, 24+30+2, 6, true, ' ');
    game.screen.add_new_window_to_layout(BannerWindowContent, 1, 69, 2, 3, true, ' ');
    game.screen.add_new_window_to_layout(LogWindowContent, 3, 69, 2, 21, true, ' ');

    return 0;
}

fn main() {
    // GAME
    let mut game: Game = Game::new();     
    let _ = game.prepare_pysical_terminal();
    let _ = prepare_ui(&mut game);

    let args: Vec<String> = env::args().collect();
    
    // run game in debug mode -- only dungeon in sketch form is shown
    if args.len() == 2 && args[1] == "debug".to_string() {
        game.screen.winds.clear();
        game.screen.add_new_window_to_layout(DebugMapWindowContent, 20, 20, 24, 6, false, ' ');
        game.vars.visit_all_rooms();
    }
    
    // game loop
    loop {

        // update
        game.render();
        game.flush_screen();

        // controls
        match read() {
            Ok(k) => match k {
                Event::Key(KeyEvent{code: KeyCode::Up, ..}) => game.vars.move_hero(Dir::Up),
                Event::Key(KeyEvent{code: KeyCode::Down, ..}) => game.vars.move_hero(Dir::Down),
                Event::Key(KeyEvent{code: KeyCode::Right, ..}) => game.vars.move_hero(Dir::Right),
                Event::Key(KeyEvent{code: KeyCode::Left, ..}) => game.vars.move_hero(Dir::Left),
                _ => break
            },
            Err(_) => todo!(),
        };
        
        // check the room
        game.vars.set_the_room_as_visited_if_needed();
    }

    let _ = game.leave_pysical_terminal();

}
