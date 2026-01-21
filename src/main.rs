use std::io::{stdout};
use crossterm::{ExecutableCommand, cursor};
use crossterm::terminal::{ClearType, Clear, enable_raw_mode, disable_raw_mode};
use crossterm::event::{read, Event, KeyEvent, KeyCode};
use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};

fn refresh_screen() {
    stdout().execute(Clear(ClearType::All)).unwrap();
}


fn main() -> Result<(), Box<dyn std::error::Error>>{
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen, Clear(ClearType::All))?;
    let x = 5;
    let y = 5;

    stdout().execute(cursor::MoveTo(x, y)).unwrap();

    loop {
        match read() {
            Ok(k) => match k {
                Event::Key(KeyEvent{code: KeyCode::Up, ..}) => stdout().execute(cursor::MoveUp(1)),
                Event::Key(KeyEvent{code: KeyCode::Down, ..}) => stdout().execute(cursor::MoveDown(1)),
                Event::Key(KeyEvent{code: KeyCode::Right, ..}) => stdout().execute(cursor::MoveRight(1)),
                Event::Key(KeyEvent{code: KeyCode::Left, ..}) => stdout().execute(cursor::MoveLeft(1)),
                Event::Key(KeyEvent{code: KeyCode::Char('k'), ..}) => break,
                _ => todo!()
            },
            Err(_) => todo!(),
        };
        refresh_screen();
    }

    execute!(stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?; 
    println!("raw mode disabled!");
    Ok(())
}
