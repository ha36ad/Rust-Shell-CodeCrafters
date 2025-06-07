use std::process::{Command, Stdio, Child};
use std::fs::{File, OpenOptions};
use std::path::Path;
use std::io::Write;
use std::env;
use crate::builtins::execute_builtin;

pub fn parse_pipeline(command_str: &str) -> Option<Vec<Vec<String>>> {
    if command_str.contains(" | ") {
        let parts: Vec<&str> = command_str.split(" | ").collect();
        let mut commands = Vec::new();
        
        for part in parts {
            if let Some(args) = shlex::split(part.trim()) {
                if !args.is_empty() {
                    commands.push(args);
                }
            } else {
                return None; // Parse error
            }
        }
        
        if commands.len() >= 2 {
            return Some(commands);
        }
    }
    None
}

pub fn execute_pipeline(commands: &[Vec<String>]) {
    if commands.is_empty() {
        return;
    }
    
    if commands.len() == 1 {
        // Single command, execute normally
        let cmd_args: Vec<&str> = commands[0].iter().map(|s| s.as_str()).collect();
        execute_single_command(&cmd_args);
        return;
    }
    
    let builtins = vec!["echo", "exit", "type", "pwd", "cd"];
    let mut processes: Vec<Child> = Vec::new();
    let mut previous_output: Option<String> = None;
    
    for (i, command) in commands.iter().enumerate() {
        let cmd_args: Vec<&str> = command.iter().map(|s| s.as_str()).collect();
        let is_builtin = builtins.contains(&cmd_args[0]);
        let is_first = i == 0;
        let is_last = i == commands.len() - 1;
        
        if is_builtin {
            // Handle built-in command
            let output = execute_builtin(&cmd_args);
            
            if is_last {
                // Last command, print to stdout
                if !output.is_empty() {
                    println!("{}", output);
                }
            } else {
                // Store output for next command
                previous_output = Some(output);
            }
        } else {
            // Handle external command
            let mut cmd = Command::new(&cmd_args[0]);
            cmd.args(&cmd_args[1..]);
            
            // Set up stdin
            if is_first {
                cmd.stdin(Stdio::inherit());
            } else if let Some(ref prev_output) = previous_output {
                // Previous command was a built-in, pipe its output
                cmd.stdin(Stdio::piped());
            } else if !processes.is_empty() {
                // Previous command was external, use its stdout
                if let Some(stdout) = processes.last_mut().unwrap().stdout.take() {
                    cmd.stdin(Stdio::from(stdout));
                }
            }
            
            // Set up stdout
            if is_last {
                cmd.stdout(Stdio::inherit());
            } else {
                cmd.stdout(Stdio::piped());
            }
            
            let mut process = cmd.spawn().expect("Failed to start command");
            
            // If previous command was built-in, write its output to this process's stdin
            if let Some(ref prev_output) = previous_output {
                if let Some(mut stdin) = process.stdin.take() {
                    let _ = stdin.write_all(prev_output.as_bytes());
                    if !prev_output.ends_with('\n') {
                        let _ = stdin.write_all(b"\n");
                    }
                    drop(stdin); // Close stdin
                }
                previous_output = None; // Clear it
            }
            
            processes.push(process);
        }
    }
    
    // Wait for all processes to complete
    for mut process in processes {
        let _ = process.wait();
    }
}

pub fn execute_single_command(cmd_args: &[&str]) {
    let builtins = vec!["echo", "exit", "type", "pwd", "cd"];
    
    if builtins.contains(&cmd_args[0]) {
        let output = execute_builtin(cmd_args);
        if !output.is_empty() {
            println!("{}", output);
        }
    } else {
        run_external_command(cmd_args[0], &cmd_args[1..], None, None);
    }
}

pub fn run_external_command(
    cmd_name: &str,
    args: &[&str],
    stdout_file: Option<&(String, bool)>,
    stderr_file: Option<&(String, bool)>
) {
    
    if let Ok(path_var) = env::var("PATH") {
        for dir in path_var.split(':') {
            let full_path = Path::new(dir).join(cmd_name);
            if let Ok(metadata) = std::fs::metadata(&full_path) {
                if metadata.is_file() {
                    let mut command = Command::new(cmd_name);
                    command.args(args);

                    // Redirect stdout to file (append or overwrite)
                    if let Some((file_path, append)) = stdout_file {
                        if let Some(parent) = Path::new(file_path).parent() {
                            let _ = std::fs::create_dir_all(parent);
                        }
                        let file = if *append {
                            OpenOptions::new().create(true).append(true).open(file_path)
                        } else {
                            File::create(file_path)
                        };
                        if let Ok(f) = file {
                            command.stdout(Stdio::from(f));
                        } else {
                            println!("Error opening stdout file: {}", file_path);
                            return;
                        }
                    }

                    // Redirect stderr to file (append or overwrite)
                    if let Some((file_path, append)) = stderr_file {
                        if let Some(parent) = Path::new(file_path).parent() {
                            let _ = std::fs::create_dir_all(parent);
                        }
                        let file = if *append {
                            OpenOptions::new().create(true).append(true).open(file_path)
                        } else {
                            File::create(file_path)
                        };
                        match file {
                            Ok(file) => {
                                command.stderr(Stdio::from(file));
                            }
                            Err(e) => {
                                println!("Error opening stderr file {}: {}", file_path, e);
                                return;
                            }
                        }
                    }

                    let mut process = command.spawn().unwrap();
                    let _status = process.wait().unwrap();
                    return;
                }
            }
        }
    }
    println!("{}: command not found", cmd_name);
}
