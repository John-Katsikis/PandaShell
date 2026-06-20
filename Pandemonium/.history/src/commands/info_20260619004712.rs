use std::process::Command;

pub fn run() {
    let username = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or("Unknown".to_string());

    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or("Unknown".to_string());

    println!();
    println!("====================================");
    println!("          PANDA SYSTEM INFO");
    println!("====================================");

    println!("User: {}", username);
    println!("Home: {}", home);

    println!(
        "Current Directory: {}",
        std::env::current_dir().unwrap().display()
    );

    println!("OS: {}", std::env::consts::OS);
    println!("Architecture: {}", std::env::consts::ARCH);

    println!(
        "CPU Cores: {}",
        std::thread::available_parallelism().unwrap().get()
    );

    if let Ok(output) = Command::new("hostname").output() {
        println!("Host: {}", String::from_utf8_lossy(&output.stdout).trim());
    }

    match std::env::consts::OS {
        "macos" => {
            if let Ok(output) = Command::new("uname").arg("-r").output() {
                println!("Kernel: {}", String::from_utf8_lossy(&output.stdout).trim());
            }

            if let Ok(output) = Command::new("sysctl")
                .args(["-n", "machdep.cpu.brand_string"])
                .output()
            {
                println!("CPU: {}", String::from_utf8_lossy(&output.stdout).trim());
            }

            if let Ok(output) = Command::new("sysctl")
                .args(["-n", "hw.memsize"])
                .output()
            {
                let ram_bytes: u64 = String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .parse()
                    .unwrap_or(0);

                println!("RAM: {} GB", ram_bytes / 1024 / 1024 / 1024);
            }
        }

        "linux" => {
            if let Ok(output) = Command::new("uname").arg("-r").output() {
                println!("Kernel: {}", String::from_utf8_lossy(&output.stdout).trim());
            }

            if let Ok(output) = Command::new("sh")
                .args([
                    "-c",
                    "grep 'model name' /proc/cpuinfo | head -1 | cut -d ':' -f2",
                ])
                .output()
            {
                println!("CPU: {}", String::from_utf8_lossy(&output.stdout).trim());
            }

            if let Ok(output) = Command::new("sh")
                .args(["-c", "grep MemTotal /proc/meminfo | awk '{print $2}'"])
                .output()
            {
                let ram_kb: u64 = String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .parse()
                    .unwrap_or(0);

                println!("RAM: {} GB", ram_kb / 1024 / 1024);
            }
        }

        "windows" => {
            println!("Detailed hardware information coming soon.");
        }

        other => {
            println!("No platform-specific information available for {}", other);
        }
    }

    println!("Shell: Pandemonium v0.1");
    println!("Welcome to the Abyss.");
    println!("====================================");
    println!();
}
