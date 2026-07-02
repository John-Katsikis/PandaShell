use std::io::{stdout, Write};

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};

use panda::{commands, shell};

fn main() -> std::io::Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.first().is_some_and(|arg| arg == "--run") {
        let input = args.get(1..).map_or(String::new(), |parts| parts.join(" "));
        let mut history = Vec::new();
        if let Err(e) = shell::execute_line(&input, &mut history) {
            eprintln!("\x1b[91m{}\x1b[0m", e);
        }
        return Ok(());
    }

    commands::logo::run();

    enable_raw_mode()?;

    let mut history: Vec<String> = Vec::new();
    let mut buffer = String::new();

    loop {
        print!("\r\x1b[2K\x1b[38;5;82m[PANDA] > \x1b[0m{}", buffer);
        stdout().flush().unwrap();

        if event::poll(std::time::Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char(c) => {
                        buffer.push(c);
                    }

                    KeyCode::Backspace => {
                        buffer.pop();
                    }

                    KeyCode::Enter => {
                        println!("\r");
                        disable_raw_mode()?;

                        let input = buffer.clone();
                        buffer.clear();

                        match shell::execute_line(&input, &mut history) {
                            Ok(shell::ShellFlow::Continue) => {}
                            Ok(shell::ShellFlow::Exit) => return Ok(()),
                            Err(e) => {
                                eprintln!("\x1b[91mParse error: {}\x1b[0m", e);
                                enable_raw_mode()?;
                                continue;
                            }
                        }

                        enable_raw_mode()?;
                    }

                    KeyCode::Tab => {
                        print!("\r\n--- History ---\r\n");

                        for (i, cmd) in history.iter().enumerate() {
                            print!("{}: {}\r\n", i, cmd);
                        }

                        print!("Select number: ");
                        stdout().flush().unwrap();

                        disable_raw_mode()?;

                        let mut selection = String::new();
                        std::io::stdin().read_line(&mut selection)?;

                        enable_raw_mode()?;

                        if let Ok(idx) = selection.trim().parse::<usize>() {
                            if let Some(cmd) = history.get(idx) {
                                buffer = cmd.clone();
                            } else {
                                print!("\r\nNo history item at index {}\r\n", idx);
                            }
                        } else {
                            print!("\r\nInvalid selection\r\n");
                        }
                    }

                    KeyCode::Esc => {
                        disable_raw_mode()?;
                        return Ok(());
                    }

                    _ => {}
                }
            }
        }
    }
}
