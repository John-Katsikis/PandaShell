use std::env;
use std::io::{stdout, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};

mod commands;

fn main() -> std::io::Result<()> {
    commands::logo::run();

    enable_raw_mode()?;

    let mut history: Vec<String> = Vec::new();
    let mut buffer = String::new();

    loop {
        print!(
            "\r\x1b[2K\x1b[38;5;82m[PANDEMONIUM] > \x1b[0m{}",
            buffer
        );
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

                        if !input.trim().is_empty() {
                            history.push(input.clone());
                        }

                        buffer.clear();

                        let mut pipeline_commands = input.trim().split(" | ").peekable();
                        let mut previous_command = None;

                        while let Some(command_text) = pipeline_commands.next() {
                            let mut parts = command_text.trim().split_whitespace();

                            let Some(command) = parts.next() else {
                                continue;
                            };

                            let args = parts;

                            match command {
                                "cd" => {
                                    let new_dir = args.peekable().peek().map_or("/", |x| *x);
                                    let root = Path::new(new_dir);

                                    if let Err(e) = env::set_current_dir(root) {
                                        eprintln!("{}", e);
                                    }

                                    previous_command = None;
                                }

                                "calc" => commands::calc::run(&input),
                                "logo" => commands::logo::run(),
                                "info" => commands::info::run(),
                                "pi" => commands::pi::run(&input),
                                "weather" => commands::weather::run(&input),
                                "help" => commands::help::run(),

                                "exit" | "quit" | "q" => {
                                    return Ok(());
                                }

                                command => {
                                    let stdin = previous_command.map_or(
                                        Stdio::inherit(),
                                        |output: Child| {
                                            Stdio::from(output.stdout.unwrap())
                                        },
                                    );

                                    let stdout = if pipeline_commands.peek().is_some() {
                                        Stdio::piped()
                                    } else {
                                        Stdio::inherit()
                                    };

                                    let output = Command::new(command)
                                        .args(args)
                                        .stdin(stdin)
                                        .stdout(stdout)
                                        .spawn();

                                    match output {
                                        Ok(output) => previous_command = Some(output),
                                        Err(e) => {
                                            previous_command = None;
                                            eprintln!("{}", e);
                                        }
                                    }
                                }
                            }
                        }

                        if let Some(mut final_command) = previous_command {
                            if let Err(e) = final_command.wait() {
                                eprintln!("wait failed: {}", e);
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

                        if let Event::Key(num_key) = event::read()? {
                            if let KeyCode::Char(digit) = num_key.code {
                                if let Some(idx) = digit.to_digit(10) {
                                    if let Some(cmd) = history.get(idx as usize) {
                                        buffer = cmd.clone();
                                    }
                                }
                            }
                        }

                        print!("\r\n");
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