use std::process::Command;

pub fn run(args: &mut dyn Iterator<Item = &str>) {
    if let Some(app) = args.next() {
        if cfg!(target_os = "macos") {
            let _ = Command::new("killall")
                .arg("-9")
                .arg(app)
                .status();
        } else if cfg!(target_os = "linux") {
            let _ = Command::new("pkill")
                .arg("-9")
                .arg(app)
                .status();
        } else {
            eprintln!("forcequit is not supported on this OS");
        }
    } else {
        eprintln!("Usage: forcequit <AppName>");
    }
}
