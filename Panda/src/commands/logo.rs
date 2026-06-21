use std::io::{stdout, Write};

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const GREEN: &str = "\x1b[38;5;82m";
const CYAN: &str = "\x1b[38;5;51m";
const BLUE: &str = "\x1b[38;5;75m";
const MAGENTA: &str = "\x1b[38;5;213m";
const YELLOW: &str = "\x1b[38;5;220m";
const RED: &str = "\x1b[38;5;203m";
const WHITE: &str = "\x1b[38;5;255m";

pub fn run() {
    print!("\x1B[2J\x1B[H");
    stdout().flush().unwrap();

    print_frame_top();
    print_wordmark();
    print_face();
    print_footer();
}

fn print_frame_top() {
    println!();
    println!("{GREEN}╭────────────────────────────────────────────────────────────╮{RESET}");
    println!(
        "{GREEN}│{RESET} {DIM}◆◇◆{RESET} {CYAN}terminal utility shell{RESET} {DIM}with teeth and glitter{RESET} {DIM}◆◇◆{RESET}      {GREEN}│{RESET}"
    );
    println!("{GREEN}├────────────────────────────────────────────────────────────╯{RESET}");
    println!(
        "   {GREEN}✦{RESET} {CYAN}▁▂▃▄▅▆▇{RESET} {YELLOW}◇{RESET} {MAGENTA}▇▆▅▄▃▂▁{RESET} {BLUE}✧{RESET} {RED}▁▂▃▄▅▆▇{RESET} {GREEN}✦{RESET}"
    );
    println!();
}

fn print_wordmark() {
    let rows = [
        ("██████╗  █████╗ ███╗   ██╗██████╗  █████╗ ", GREEN),
        ("██╔══██╗██╔══██╗████╗  ██║██╔══██╗██╔══██╗", CYAN),
        ("██████╔╝███████║██╔██╗ ██║██║  ██║███████║", YELLOW),
        ("██╔═══╝ ██╔══██║██║╚██╗██║██║  ██║██╔══██║", MAGENTA),
        ("██║     ██║  ██║██║ ╚████║██████╔╝██║  ██║", BLUE),
        ("╚═╝     ╚═╝  ╚═╝╚═╝  ╚═══╝╚═════╝ ╚═╝  ╚═╝", RED),
    ];

    for (row, color) in rows {
        println!("{BOLD}{color}{row}{RESET}");
    }
}

fn print_face() {
    println!();
    println!("          {WHITE}╭─────────────╮{RESET}");
    println!(
        "       {WHITE}╭──╯{RESET} {GREEN}◖██◗{RESET}   {GREEN}◖██◗{RESET} {WHITE}╰──╮{RESET}"
    );
    println!("       {WHITE}│{RESET}       {MAGENTA}◆{RESET}       {WHITE}│{RESET}   {DIM}charged and ready{RESET}");
    println!("       {WHITE}╰──╮{RESET}  {YELLOW}╰───╯{RESET}  {WHITE}╭──╯{RESET}");
    println!("          {WHITE}╰─────────────╯{RESET}");
}

fn print_footer() {
    println!();
    println!(
        "{GREEN}╰─{RESET} {YELLOW}Panda v{} {RESET}{DIM}:: calc | weather | sniff | spark | formula | pi{RESET}",
        env!("CARGO_PKG_VERSION")
    );
    println!("{DIM}   type `help` to open the command reference{RESET}");
    println!();
}
