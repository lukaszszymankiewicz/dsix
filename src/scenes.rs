use crate::gfx::{Layout, RenderableScene};
use crossterm::event::{KeyCode, KeyEvent, Event, read};
use crate::dir::{Dir};

use crate::game::{GameVars, ControlSignal};

use crate::windows::{
    // DebugWindowMainMapContents,
    WindowTopBannerContent,
    WindowLogsContent,
    WindowHeroStatsContent,
    WindowMainMapContents,
    WindowSkullImageContent,
    GreetingWindowContent,
};


pub struct MainDungeonScene;

impl RenderableScene for MainDungeonScene {
    fn fill_the_layout(&self, layout: &mut Layout) {
        layout.add_new_window_to_layout(WindowMainMapContents, 13, 30, 24, 6, true, '.');
        layout.add_new_window_to_layout(WindowSkullImageContent, 13, 20, 2, 6, true, ' ');
        layout.add_new_window_to_layout(WindowHeroStatsContent, 13, 15, 24+30+2, 6, true, ' ');

        layout.add_new_window_to_layout(WindowTopBannerContent, 1, 69, 2, 3, true, ' ');
        layout.add_new_window_to_layout(WindowLogsContent, 3, 69, 2, 21, true, ' ');
    }

    fn correspond_to_controls(&self, vars: &mut GameVars) -> ControlSignal {
        
        let mut control_signal: ControlSignal = ControlSignal::DoNothingBitchSlap;

        match read() {
            Ok(k) => match k {
                Event::Key(KeyEvent{code: KeyCode::Up, ..}) => control_signal = vars.move_hero(Dir::Up),
                Event::Key(KeyEvent{code: KeyCode::Down, ..}) => control_signal = vars.move_hero(Dir::Down),
                Event::Key(KeyEvent{code: KeyCode::Right, ..}) => control_signal = vars.move_hero(Dir::Right),
                Event::Key(KeyEvent{code: KeyCode::Left, ..}) => control_signal = vars.move_hero(Dir::Left),
                _ => control_signal = ControlSignal::ExitGame,
            },
            Err(_) => control_signal = ControlSignal::ExitGame,
        };

        return control_signal;
    }

    fn dispatch_scene(&self, vars: &mut GameVars) -> Box<dyn RenderableScene> {
        return Box::new(MainDungeonScene);
    }
}



pub struct GreetingScene;
impl RenderableScene for GreetingScene {
    fn fill_the_layout(&self, layout: &mut Layout) {
        layout.add_new_window_to_layout(GreetingWindowContent, 13, 30, 24, 6, true, ' ');
    }

    fn correspond_to_controls(&self, vars: &mut GameVars) -> ControlSignal {

        let mut control_signal: ControlSignal = ControlSignal::DoNothingBitchSlap;

        match read() {
            Ok(k) => match k {
                Event::Key(KeyEvent{code: KeyCode::Left, ..}) => control_signal = ControlSignal::ExitGame,
                _ => control_signal = ControlSignal::ChangeScene,
            },
            Err(_) => todo!(),
        };
        
        return control_signal;
    }

    fn dispatch_scene(&self, vars: &mut GameVars) -> Box<dyn RenderableScene> {
        return Box::new(MainDungeonScene);
    }
}

// pub struct DebugMainDungeonScene;
// impl RenderableScene for DebugMainDungeonScene {
//     fn fill_the_layout(ts: &mut Scene) {
//         ts.add_new_window_to_layout(DebugWindowMainMapContents, 13, 30, 24, 6, true, '.');
//     }
// }
