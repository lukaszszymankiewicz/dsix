mod dir;
mod dice;
mod dungeon;
mod entity;
mod game;
mod gfx;
mod windows;
mod scenes;

use crate::gfx::RenderableScene;
use crate::game::Game;
use crate::scenes::{GreetingScene, MainDungeonScene};
use crate::dir::Dir;


fn main() {
    // GAME
    let initial_scene = Box::new(GreetingScene);
    let mut game: Game = Game::new(initial_scene);     
    game.run_game_loop();
}

    // game.change_scene(CurrentScene::MainDungeonScene);

    // let args: Vec<String> = env::args().collect();
    
    // run game in debug mode -- only dungeon in sketch form is shown
    // if args.len() == 2 && args[1] == "debug".to_string() {
    //     game.current_scene.windows.clear();
    //     game.current_scene.add_new_window_to_layout(DebugMapWindowContent, 20, 20, 24, 6, false, ' ');
    //     game.vars.visit_all_rooms();
    // }
