use std::env;
use std::io::{stdout, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{enable_raw_mode, disable_raw_mode},
};

mod commands;

fn main() -> crossterm::Result<()> {

    commands::logo::run();

    enable_raw_mode()?; // <-- RAW MODE ENABLED

    let mut history: Vec<String> = Vec::new();
    let mut buffer = String::new();

    loop {
        // draw prompt + current buffer
        print!("\r\x1b[38;5;82m[PANDEMONIUM] > \x1b[0m{}", buffer);
        stdout().flush().unwrap();

        // wait for keypress
        if event::poll(std::time::Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                match key.code {

                    // typed character
                    KeyCode::Char(c) => {
                        buffer.push(c);
                    }

                    // backspace
                    KeyCode::Backspace => {
                        buffer.pop();
                    }

                    // ENTER → run your existing pipeline logic
                    KeyCode::Enter => {
                        println!();
                        let input = buffer.clone();
                        history.push(input.clone());
                        buffer.clear();

                        // --- YOUR ORIGINAL EXECUTION LOGIC ---
                        let mut commands = input.trim().split(" | ").peekable();
                        let mut previous_command = None;

                        while let Some(command) = commands.next() {
                            let mut parts = command.trim().split_whitespace();
                            let Some(command) = parts.next() else { continue };
                            let args = parts;

                            match command {
                                "cd" => {
                                    let new_dir = args.peekable().peek().map_or("/", |x| *x);
                                    let root = Path::new(new_dir);
                                    if let Err(e) = env::set_current_dir(&root) {
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
                                    disable_raw_mode()?;
                                    return Ok(());
                                }

                                command => {
                                    let stdin = previous_command
                                        .map_or(Stdio::inherit(), |output: Child| {
                                            Stdio::from(output.stdout.unwrap())
                                        });

                                    let stdout = if commands.peek().is_some() {
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
                        // --- END ORIGINAL LOGIC ---
                    }

                    // TAB → show history and allow selection
                    KeyCode::Tab => {
                        println!("\n--- History ---");
                        for (i, cmd) in history.iter().enumerate() {
                            println!("{}: {}", i, cmd);
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
                    }

                    // ESC → exit shell
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
