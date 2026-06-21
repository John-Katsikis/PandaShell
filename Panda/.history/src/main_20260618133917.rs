use std::env;
use std::io::{stdin, stdout, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};

mod commands;


fn main(){

    commands::logo::run();

    loop {
        print!("\x1b[38;5;82m[PANDEMONIUM] > \x1b[0m");
        if let Err(e) = stdout().flush() {
        eprintln!("flush failed: {}", e);
}
        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();

        // must be peekable so we know when we are on the last command
        let mut commands = input.trim().split(" | ").peekable();
        let mut previous_command = None;

        while let Some(command) = commands.next()  {

            let mut parts = command.trim().split_whitespace();
            
            let Some(command) = parts.next() else {
                continue;
            };

            let args = parts;

            match command {
                "cd" => {
                    let new_dir = args.peekable().peek()
                        .map_or("/", |x| *x);
                    let root = Path::new(new_dir);
                    if let Err(e) = env::set_current_dir(&root) {
                        eprintln!("{}", e);
                    }

                    previous_command = None;
                },

                "calc" => {
                    commands::calc::run(&input);
                },
                   
                "logo" => commands::logo::run(),
                
                "info" => {
                    commands::info::run();
                },
  
                "pi" => {
                    commands::pi::run(&input);
                },

                "weather" => {
                    commands::weather::run(&input);
                },
                
                "help" => {
                    commands::help::run();
                },
                
                "exit" | "quit" | "q" => return,
                command => {
                    let stdin = previous_command
                        .map_or(
                            Stdio::inherit(),
                            |output: Child| Stdio::from(output.stdout.unwrap())
                        );

                    let stdout = if commands.peek().is_some() {
                        // there is another command piped behind this one
                        // prepare to send output to the next command
                        Stdio::piped()
                    } else {
                        // there are no more commands piped behind this one
                        // send output to shell stdout
                        Stdio::inherit()
                    };

                    let output = Command::new(command)
                        .args(args)
                        .stdin(stdin)
                        .stdout(stdout)
                        .spawn();

                    match output {
                        Ok(output) => { previous_command = Some(output); },
                        Err(e) => {
                            previous_command = None;
                            eprintln!("{}", e);
                        },
                    };
                }
            }
        }

        if let Some(mut final_command) = previous_command {
            // block until the final command has finished
            if let Err(e) = final_command.wait() {
            eprintln!("wait failed: {}", e);
}
        }

    }
}

