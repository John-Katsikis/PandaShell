use std::env;
use std::path::Path;
use std::process::{Child, Command, Stdio};

use crate::{commands, parser};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellFlow {
    Continue,
    Exit,
}

pub fn execute_line(input: &str, history: &mut Vec<String>) -> Result<ShellFlow, String> {
    if !input.trim().is_empty() {
        history.push(input.to_string());
    }

    let ast = parser::parse_line(input)?;
    let pipeline_len = ast.commands.len();
    let mut previous_command = None;

    for (index, command_ast) in ast.commands.iter().enumerate() {
        match command_ast.name.as_str() {
            "cd" => {
                let new_dir = command_ast.args.first().map_or("/", String::as_str);
                let root = Path::new(new_dir);

                if let Err(e) = env::set_current_dir(root) {
                    eprintln!("{}", e);
                }

                previous_command = None;
            }

            "app" => commands::app::run(command_ast.as_input()),
            "calc" => commands::calc::run(command_ast.as_input()),
            "doctor" => commands::doctor::run(command_ast.as_input()),
            "envx" => commands::envx::run(command_ast.as_input()),
            "findup" => commands::findup::run(command_ast.as_input()),
            "gitinfo" => commands::gitinfo::run(),
            "logo" => commands::logo::run(),
            "info" => commands::info::run(),
            "json" => commands::json::run(command_ast.as_input()),
            "pi" => commands::pi::run(command_ast.as_input()),
            "qr" => commands::qr::run(command_ast.as_input()),
            "restart" => commands::restart::run(command_ast.as_input()),
            "serve" => commands::serve::run(command_ast.as_input()),
            "size" => commands::size::run(command_ast.as_input()),
            "weather" => commands::weather::run(command_ast.as_input()),
            "hash" => commands::hash::run(command_ast.as_input()),
            "help" => commands::help::run(),
            "sniff" => commands::sniff::run(command_ast.as_input()),
            "spark" => commands::spark::run_seed(command_ast.arg_text()),
            "timer" => commands::timer::run(command_ast.as_input()),
            "todo" => commands::todo::run(command_ast.as_input()),
            "tree" => commands::tree::run(command_ast.as_input()),
            "url" => commands::url::run(command_ast.as_input()),
            "uuid" => commands::uuid::run(command_ast.as_input()),
            "watch" => commands::watch::run(command_ast.as_input()),
            "formula" => commands::formula::run(command_ast.as_input()),

            "forcequit" => {
                commands::forcequit::run(&mut command_ast.args.iter().map(String::as_str));
                previous_command = None;
            }

            "clear_history" => {
                history.clear();
                previous_command = None;
            }

            "exit" | "quit" | "q" => {
                return Ok(ShellFlow::Exit);
            }

            command => {
                let stdin = previous_command.map_or(Stdio::inherit(), |output: Child| {
                    Stdio::from(output.stdout.unwrap())
                });

                let stdout = if index + 1 < pipeline_len {
                    Stdio::piped()
                } else {
                    Stdio::inherit()
                };

                let output = Command::new(command)
                    .args(&command_ast.args)
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

    Ok(ShellFlow::Continue)
}
