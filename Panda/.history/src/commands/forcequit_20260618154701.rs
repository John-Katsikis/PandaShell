pub fn run() {

if let Some(app) = args.peekable().peek() {
                                        if cfg!(target_os = "macos") {
                                            // macOS: killall -9 "AppName"
                                            let _ = Command::new("killall")
                                                .arg("-9")
                                                .arg(app)
                                                .status();
                                        } else if cfg!(target_os = "linux") {
                                            // Linux: pkill -9 appname
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

                                    previous_command = None;
                                }