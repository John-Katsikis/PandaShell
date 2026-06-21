const RESET: &str = "\x1b[0m";
const DIM: &str = "\x1b[2m";
const GREEN: &str = "\x1b[38;5;82m";
const CYAN: &str = "\x1b[38;5;51m";
const BLUE: &str = "\x1b[38;5;75m";
const MAGENTA: &str = "\x1b[38;5;213m";
const YELLOW: &str = "\x1b[38;5;220m";

pub fn run() {
    print_header();

    section(
        "Shell",
        GREEN,
        &[
            ("cd <dir>", "Change directory"),
            ("clear_history", "Clear the in-session command history"),
            ("app <name>", "Open a macOS app by name"),
            ("app --list [search]", "List or search installed macOS apps"),
            ("restart <app>", "Quit and reopen a macOS app"),
            ("logo", "Clear the screen and redraw the Panda banner"),
            ("exit | quit | q", "Leave Panda"),
            ("TAB", "Open the history picker"),
            (
                "cmd | cmd",
                "Pipe external commands; quotes protect literal | characters",
            ),
        ],
    );

    section(
        "Math & Visuals",
        CYAN,
        &[
            (
                "calc <expr>",
                "Evaluate numeric expressions with constants and functions",
            ),
            (
                "formula <expr> <min> <max>",
                "Plot y=f(x) as an ASCII graph",
            ),
            (
                "pi <digits>",
                "Render colorized pi digits and digit statistics",
            ),
            (
                "pi <digits> --plain",
                "Print raw pi digits for copying or piping",
            ),
            ("qr [options] <text>", "Print a scannable terminal QR code"),
            (
                "spark [seed text]",
                "Generate a deterministic colorful terminal sigil",
            ),
            (
                "spark --file <path>",
                "Spark a visual fingerprint from file contents",
            ),
            (
                "spark --compare A B",
                "Compare text or @file visual signatures",
            ),
        ],
    );

    section(
        "Utilities",
        MAGENTA,
        &[
            ("hash <text>", "SHA-256 hash text"),
            ("hash --file <path>", "SHA-256 hash a file"),
            ("json pretty <json>", "Validate and format JSON"),
            ("uuid [short] [count]", "Generate UUIDs or short IDs"),
            ("url encode|decode <text>", "Encode or decode URL text"),
            (
                "findup <name>",
                "Find a file or folder in parent directories",
            ),
            ("size [path]", "Summarize file or directory size"),
            ("doctor [mode]", "Diagnose disk, project, and tool health"),
            ("weather <city>", "Current weather from Open-Meteo"),
            ("weather <city> --hourly", "Hourly forecast"),
            ("weather <city> --days N", "Multi-day forecast"),
            ("tree [path]", "Finder-like visible directory view"),
            (
                "tree [path] --all",
                "Detailed recursive tree, including hidden files",
            ),
            ("info", "Colorful system dashboard"),
            ("envx [key|search term]", "Inspect environment variables"),
            ("todo add|list|done", "Tiny local todo list"),
        ],
    );

    section(
        "Network & System",
        YELLOW,
        &[
            ("gitinfo", "Quick git repository dashboard"),
            ("serve [port]", "Serve the current directory over HTTP"),
            ("watch [options]", "Refresh a live system/project dashboard"),
            ("timer <duration>", "Countdown timer or stopwatch"),
            ("sniff --list", "List packet-capture interfaces"),
            ("sniff --interface IFACE --count N", "Capture IPv4 packets"),
            ("sniff --tcp --port 443", "Capture HTTPS-like TCP traffic"),
            ("sniff --icmp --count 5", "Capture ICMP packets"),
            ("forcequit <app>", "Force quit an app/process by name"),
        ],
    );

    examples();
    footer();
}

fn print_header() {
    println!();
    println!("{GREEN}╭──────────────────── PANDA HELP ────────────────────╮{RESET}");
    println!(
        "{GREEN}│{RESET} {MAGENTA}Panda command reference{RESET} {DIM}v0.2.0{RESET}                       {GREEN}│{RESET}"
    );
    println!("{GREEN}╰─────────────────────────────────────────────────────╯{RESET}");
}

fn section(title: &str, color: &str, rows: &[(&str, &str)]) {
    println!();
    println!("{color}┌─ {title}{RESET}");

    for (command, description) in rows {
        println!(
            "{color}│{RESET} {BLUE}{:<30}{RESET} {DIM}{}{RESET}",
            command, description
        );
    }

    println!("{color}└─────────────────────────────────────────────────────{RESET}");
}

fn examples() {
    println!();
    println!("{CYAN}Examples{RESET}");
    println!("{DIM}  app \"Visual Studio Code\"{RESET}");
    println!("{DIM}  restart --force Safari{RESET}");
    println!("{DIM}  doctor --project{RESET}");
    println!("{DIM}  watch --count 3 --interval 1{RESET}");
    println!("{DIM}  calc 2sin(pi/2) + sqrt(9){RESET}");
    println!("{DIM}  formula exp(-x^2) -3 3{RESET}");
    println!("{DIM}  pi 250 --group 10{RESET}");
    println!("{DIM}  qr --compact \"https://example.com\"{RESET}");
    println!("{DIM}  spark hash Yianni{RESET}");
    println!("{DIM}  spark --compare @Cargo.toml @Cargo.lock{RESET}");
    println!("{DIM}  json pretty '{{\"panda\":true}}'{RESET}");
    println!("{DIM}  gitinfo{RESET}");
    println!("{DIM}  todo add \"polish Panda\"{RESET}");
    println!("{DIM}  tree{RESET}");
    println!("{DIM}  tree . --all{RESET}");
    println!("{DIM}  weather Athens --alerts{RESET}");
    println!("{DIM}  sniff --interface en0 --tcp --port 443 --count 20{RESET}");
}

fn footer() {
    println!();
    println!(
        "{YELLOW}Tip:{RESET} {DIM}Panda parses commands as AST nodes, so `spark hash Yianni` uses the literal text as the seed.{RESET}"
    );
    println!();
}
