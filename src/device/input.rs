use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyModifiers};
use std::io::stdin;

pub fn read_str() -> Vec<u8> {
    let mut chars = String::new();
    stdin().read_line(&mut chars).unwrap();
    chars.trim().as_bytes().to_vec()
}

pub fn read_char() -> Result<u8> {
    let mut char = [0_u8; 1];
    crossterm::terminal::enable_raw_mode()?;
    let mut event = crossterm::event::read()?;
    loop {
        if let Event::Key(key) = event {
            if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('c') {
                crossterm::terminal::disable_raw_mode()?;
                std::process::exit(1);
            }
            match key.code {
                KeyCode::Enter => {
                    char[0] = 10;
                    break;
                }
                KeyCode::Backspace => {
                    char[0] = 8;
                    break;
                }
                KeyCode::Tab => {
                    char[0] = 9;
                    break;
                }
                KeyCode::Char(chr) => {
                    char[0] = chr as u8;
                    break;
                }
                KeyCode::Esc => {
                    char[0] = 27;
                    break;
                }
                KeyCode::Delete => {
                    char[0] = 127;
                    break;
                }
                _ => {}
            }
        }
        event = crossterm::event::read()?;
    }
    crossterm::terminal::disable_raw_mode()?;
    Ok(char[0])
}
