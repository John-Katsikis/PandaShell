use std::env;
use std::process::Command;

const RESET: &str = "\x1b[0m";
const DIM: &str = "\x1b[2m";
const GREEN: &str = "\x1b[38;5;82m";
const CYAN: &str = "\x1b[38;5;51m";
const BLUE: &str = "\x1b[38;5;75m";
const MAGENTA: &str = "\x1b[38;5;213m";
const YELLOW: &str = "\x1b[38;5;220m";
const RED: &str = "\x1b[38;5;203m";

pub fn run() {
    let user = env_value(&["USER", "USERNAME"]).unwrap_or_else(|| "Unknown".into());
    let home = env_value(&["HOME", "USERPROFILE"]).unwrap_or_else(|| "Unknown".into());
    let cwd = env::current_dir()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|_| "Unknown".into());

    let mut system = Vec::new();
    system.push(("Host", command_output("hostname", &[])));
    system.push(("User", Some(user)));
    system.push(("Home", Some(home)));
    system.push(("Current Dir", Some(cwd)));
    system.push(("OS", os_name()));
    system.push(("Kernel", command_output("uname", &["-r"])));
    system.push(("Architecture", Some(env::consts::ARCH.into())));
    system.push(("Panda", Some(format!("v{}", env!("CARGO_PKG_VERSION")))));

    let mut hardware = Vec::new();
    hardware.push(("CPU", cpu_name()));
    hardware.push(("CPU Cores", cpu_cores()));
    hardware.push(("Memory", memory_total()));
    hardware.push(("Disk /", disk_usage()));
    hardware.push(("Uptime", uptime()));

    let mut network = Vec::new();
    network.push(("Local IP", local_ip()));
    network.push(("Shell", env_value(&["SHELL", "COMSPEC"])));
    network.push(("Terminal", env_value(&["TERM_PROGRAM", "TERM"])));

    println!();
    print_title();
    print_pulse();
    print_section("SYSTEM", CYAN, &system);
    print_section("HARDWARE", MAGENTA, &hardware);
    print_section("SESSION", YELLOW, &network);
    print_footer();
    println!();
}

fn print_title() {
    println!(
        "{green}╭──────────────────────────────────────────────────────╮{reset}",
        green = GREEN,
        reset = RESET
    );
    println!(
        "{green}│{reset} {magenta}◆ PANDA SYSTEM ORACLE ◆{reset} {dim}telemetry from the bamboo core{reset} {green}│{reset}",
        green = GREEN,
        magenta = MAGENTA,
        dim = DIM,
        reset = RESET
    );
    println!(
        "{green}╰──────────────────────────────────────────────────────╯{reset}",
        green = GREEN,
        reset = RESET
    );
}

fn print_pulse() {
    println!(
        "{dim}     {green}▁▂▃▄▅▆▇{yellow}▆▅▄▃▂▁{blue}  ◇  {magenta}▁▂▃▄▅▆▇{red}▆▅▄▃▂▁{reset}",
        dim = DIM,
        green = GREEN,
        yellow = YELLOW,
        blue = BLUE,
        magenta = MAGENTA,
        red = RED,
        reset = RESET
    );
}

fn print_section(title: &str, color: &str, rows: &[(&str, Option<String>)]) {
    println!();
    println!(
        "{color}┌─ {title} ─────────────────────────────────────────{reset}",
        reset = RESET
    );

    for (label, value) in rows {
        let value = value
            .as_deref()
            .filter(|v| !v.is_empty())
            .unwrap_or("Unknown");
        println!(
            "{color}│{reset} {label:<12} {dim}→{reset} {value}",
            color = color,
            reset = RESET,
            label = label,
            dim = DIM,
            value = value
        );
    }

    println!(
        "{color}└────────────────────────────────────────────────────{reset}",
        reset = RESET
    );
}

fn print_footer() {
    println!();
    println!(
        "{green}        ◢◤{reset} {dim}Panda says: systems nominal, snacks advisable.{reset} {green}◥◣{reset}",
        green = GREEN,
        dim = DIM,
        reset = RESET
    );
}

fn env_value(keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| env::var(key).ok())
}

fn command_output(command: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(command).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!text.is_empty()).then_some(text)
}

fn os_name() -> Option<String> {
    match env::consts::OS {
        "macos" => {
            let product = command_output("sw_vers", &["-productName"])?;
            let version = command_output("sw_vers", &["-productVersion"])?;
            Some(format!("{} {}", product, version))
        }
        "linux" => linux_pretty_name().or_else(|| Some("Linux".into())),
        "windows" => Some("Windows".into()),
        other => Some(other.into()),
    }
}

fn linux_pretty_name() -> Option<String> {
    let release = std::fs::read_to_string("/etc/os-release").ok()?;
    release.lines().find_map(|line| {
        line.strip_prefix("PRETTY_NAME=")
            .map(|value| value.trim_matches('"').to_string())
    })
}

fn cpu_name() -> Option<String> {
    match env::consts::OS {
        "macos" => command_output("sysctl", &["-n", "machdep.cpu.brand_string"]),
        "linux" => std::fs::read_to_string("/proc/cpuinfo")
            .ok()?
            .lines()
            .find_map(|line| line.strip_prefix("model name"))
            .and_then(|line| {
                line.split_once(':')
                    .map(|(_, value)| value.trim().to_string())
            }),
        "windows" => env::var("PROCESSOR_IDENTIFIER").ok(),
        _ => None,
    }
}

fn cpu_cores() -> Option<String> {
    std::thread::available_parallelism()
        .ok()
        .map(|count| count.get().to_string())
}

fn memory_total() -> Option<String> {
    match env::consts::OS {
        "macos" => {
            let bytes = command_output("sysctl", &["-n", "hw.memsize"])?
                .parse::<u64>()
                .ok()?;
            Some(format_bytes(bytes))
        }
        "linux" => {
            let meminfo = std::fs::read_to_string("/proc/meminfo").ok()?;
            let kb = meminfo
                .lines()
                .find_map(|line| line.strip_prefix("MemTotal:"))?
                .split_whitespace()
                .next()?
                .parse::<u64>()
                .ok()?;
            Some(format_bytes(kb * 1024))
        }
        _ => None,
    }
}

fn disk_usage() -> Option<String> {
    let output = command_output("df", &["-h", "/"])?;
    let line = output.lines().nth(1)?;
    let cols: Vec<&str> = line.split_whitespace().collect();

    if cols.len() >= 5 {
        Some(format!("{} used of {} ({})", cols[2], cols[1], cols[4]))
    } else {
        None
    }
}

fn uptime() -> Option<String> {
    match env::consts::OS {
        "macos" | "linux" => command_output("uptime", &[]).map(clean_uptime),
        _ => None,
    }
}

fn clean_uptime(raw: String) -> String {
    raw.split(" up ")
        .nth(1)
        .and_then(|rest| rest.split(',').next())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or(raw)
}

fn local_ip() -> Option<String> {
    match env::consts::OS {
        "macos" => command_output("ipconfig", &["getifaddr", "en0"])
            .or_else(|| command_output("ipconfig", &["getifaddr", "en1"])),
        "linux" => command_output("hostname", &["-I"])
            .and_then(|ips| ips.split_whitespace().next().map(|ip| ip.to_string())),
        _ => None,
    }
}

fn format_bytes(bytes: u64) -> String {
    let gb = bytes as f64 / 1024.0 / 1024.0 / 1024.0;
    format!("{:.1} GB", gb)
}
