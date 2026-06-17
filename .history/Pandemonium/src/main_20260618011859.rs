use std::env;
use std::io::{stdin, stdout, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};

fn print_logo(){
    println!("\x1b[38;5;82m
██████╗  █████╗ ███╗   ██╗██████╗ ███████╗███╗   ███╗ ██████╗ ███╗   ██╗██╗██╗   ██╗███╗   ███╗
██╔══██╗██╔══██╗████╗  ██║██╔══██╗██╔════╝████╗ ████║██╔═══██╗████╗  ██║██║██║   ██║████╗ ████║
██████╔╝███████║██╔██╗ ██║██║  ██║█████╗  ██╔████╔██║██║   ██║██╔██╗ ██║██║██║   ██║██╔████╔██║
██╔═══╝ ██╔══██║██║╚██╗██║██║  ██║██╔══╝  ██║╚██╔╝██║██║   ██║██║╚██╗██║██║██║   ██║██║╚██╔╝██║
██║     ██║  ██║██║ ╚████║██████╔╝███████╗██║ ╚═╝ ██║╚██████╔╝██║ ╚████║██║╚██████╔╝██║ ╚═╝ ██║
╚═╝     ╚═╝  ╚═╝╚═╝  ╚═══╝╚═════╝ ╚══════╝╚═╝     ╚═╝ ╚═════╝ ╚═╝  ╚═══╝╚═╝ ╚═════╝ ╚═╝     ╚═╝

\x1b[38;5;82m                    Welcome to the Abyss. Mind your footing.\x1b[0m
");
}

fn main(){

    print_logo();

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
            let command = parts.next().unwrap();
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
                    let parts: Vec<&str> = input.split_whitespace().collect();

                    //if parts.len() != 4 {
                    //    println!("Usage: calc <number> <whitespace> <operator> <whitespace> <number>");
                    //    continue;
                    //}

                    let left: i32 = parts[1].parse().unwrap();
                    let op = parts[2];

                                        
                    let right: i32 = if op == "sqrt" || op == "square_root" {
                        0
                    } else {
                        parts[3].parse().unwrap()
                    };

                    match op {
                        "+" | "add" => println!("{}", left + right),
                        "-" | "sub" => println!("{}", left - right),
                        "*" | "mul" => println!("{}", left * right),
                        "/" | "div" => println!("{}", left / right),
                        "^" | "pow" => println!("{}", left.pow(right as u32)),
                        "%" | "mod" => println!("{}", left % right),
                        "sqrt" | "square_root" => println!("{}", (left as f64).sqrt()),

                        _ => println!("Unknown operator"),
}
                },

                "logo" => print_logo(),

                "info" => {
                    use std::process::Command;

                    println!();
                    println!("====================================");
                    println!("      PANDEMONIUM SYSTEM INFO");
                    println!("====================================");

                    // User
                    println!(
                        "User: {}",
                        std::env::var("USER")
                            .unwrap_or("Unknown".to_string())
                    );

                    // Home directory
                    println!(
                        "Home: {}",
                        std::env::var("HOME")
                            .unwrap_or("Unknown".to_string())
                    );

                    // Current directory
                    println!(
                        "Directory: {}",
                        std::env::current_dir()
                            .unwrap()
                            .display()
                    );

                    // Hostname
                    let hostname = Command::new("hostname")
                        .output()
                        .unwrap();

                    println!(
                        "Host: {}",
                        String::from_utf8_lossy(&hostname.stdout).trim()
                    );

                    // OS
                    println!("OS: {}", std::env::consts::OS);

                    // Architecture
                    println!("Architecture: {}", std::env::consts::ARCH);

                    // Kernel
                    let kernel = Command::new("uname")
                        .arg("-r")
                        .output()
                        .unwrap();

                    println!(
                        "Kernel: {}",
                        String::from_utf8_lossy(&kernel.stdout).trim()
                    );

                    // CPU Name
                    let cpu = Command::new("sysctl")
                        .args(["-n", "machdep.cpu.brand_string"])
                        .output();

                    match cpu {
                        Ok(output) => println!(
                            "CPU: {}",
                            String::from_utf8_lossy(&output.stdout).trim()
                        ),
                        Err(_) => println!("CPU: Apple Silicon"),
                    }

                    // Core Count
                    println!(
                        "CPU Cores: {}",
                        std::thread::available_parallelism()
                            .unwrap()
                            .get()
                    );

                    // RAM
                    let ram = Command::new("sysctl")
                        .args(["-n", "hw.memsize"])
                        .output()
                        .unwrap();

                    let ram_bytes: u64 = String::from_utf8_lossy(&ram.stdout)
                        .trim()
                        .parse()
                        .unwrap_or(0);

                    let ram_gb = ram_bytes / 1024 / 1024 / 1024;

                    println!("RAM: {} GB", ram_gb);

                    // Uptime
                    let uptime = Command::new("uptime")
                        .output()
                        .unwrap();

                    println!(
                        "Uptime: {}",
                        String::from_utf8_lossy(&uptime.stdout).trim()
                    );

                    println!("Shell: Pandemonium v0.1");
                    println!("Welcome to the Abyss.");
                    println!("====================================");
                    println!();
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