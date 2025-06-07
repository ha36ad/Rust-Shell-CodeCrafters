// src/main.rs
mod completer;
mod builtins;
mod execution;
mod redirection;
mod utils;

use rustyline::error::ReadlineError;
use rustyline::{Editor};
use rustyline::history::{DefaultHistory,History};
use completer::MyCompleter;
use execution::{parse_pipeline, execute_pipeline};
use redirection::parse_redirection;
use builtins::{cmd_type, cmd_echo, cmd_cd, cmd_history};
use utils::print_or_write;
use std::env;
use std::fs::{File, OpenOptions};
use std::path::Path;
use std::io::Write;

fn main() {
    let completer = MyCompleter::new();
    let mut rl = Editor::<MyCompleter, DefaultHistory>::new().unwrap();
    rl.set_helper(Some(completer));


    // Track the last written history position for -a command
    let mut last_written_pos = 0;
    
    // Load history from HISTFILE on startup
    let histfile = env::var("HISTFILE").ok();
    if let Some(ref file_path) = histfile {
        if let Ok(contents) = std::fs::read_to_string(file_path) {
            for line in contents.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    let _ = rl.add_history_entry(trimmed);
                }
            }
            // Track where the loaded history ends and new session begins
            last_written_pos = rl.history().len();
        }
    }

    loop {
        let readline = rl.readline("$ ");
        match readline {
            Ok(line) => {
                let input = line.trim();
                if input.is_empty() {
                    continue;
                }
                
                // Add to history
                let _ = rl.add_history_entry(input);
                let (command_str, stdout_file, stderr_file) = parse_redirection(input);

                // Check for pipeline first
                if let Some(commands) = parse_pipeline(&command_str) {
                    execute_pipeline(&commands);
                    continue;
                }

                // Ensure redirection files are created even if empty
                if let Some((file_path, append)) = &stdout_file {
                    if let Some(parent) = Path::new(file_path).parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    let _ = if *append {
                        OpenOptions::new().create(true).append(true).open(file_path)
                    } else {
                        File::create(file_path)
                    };
                }
                if let Some((file_path, append)) = &stderr_file {
                    if let Some(parent) = Path::new(file_path).parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    let _ = if *append {
                        OpenOptions::new().create(true).append(true).open(file_path)
                    } else {
                        File::create(file_path)
                    };
                }

                // Shlex split for quotations and other symbols
                let split = shlex::split(&command_str).unwrap();
                let args: Vec<&str> = split.iter().map(|s| s.as_str()).collect();

                match args.as_slice() {
                    // type
                    ["type", args @ ..] => cmd_type(args, stdout_file.as_ref()),
                    // echo
                    ["echo", args @ ..] => {
                        let result = cmd_echo(args);
                        print_or_write(stdout_file.as_ref(), &result);
                    }
                    // exit
                    ["exit", "0"] => {
                        if let Ok(histfile_path) = env::var("HISTFILE") {
                            let history: Vec<String> = rl.history().iter().map(|s| s.to_string()).collect();
                            
                            // Check if file already exists and has content
                            let existing_content = std::fs::read_to_string(&histfile_path).unwrap_or_default();
                            let existing_lines: Vec<&str> = existing_content.lines().collect();
                            
                            if existing_lines.is_empty(){
                                // No existing history or we didn't load any - write all history
                                let _ = std::fs::write(&histfile_path, history.join("\n") + "\n");
                            } else {
                                // Append only new commands from current session
                                if last_written_pos < history.len() {
                                    let new_commands: Vec<String> = history[last_written_pos..].to_vec();
                                    if !new_commands.is_empty() {
                                        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&histfile_path) {
                                            for command in &new_commands {
                                                let _ = writeln!(file, "{}", command);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                         std::process::exit(0);
                    }
                    // pwd
                    ["pwd"] => {
                        let path = env::current_dir().unwrap();
                        print_or_write(stdout_file.as_ref(), &path.display().to_string());
                    }
                    // cd 
                    ["cd", args @ ..] => {
                        cmd_cd(args);
                    }
                    // history
                    ["history", args @ ..] =>  {
                        if args.len() >= 2 && args[0] == "-r" {
                            // Read history from file
                            let file_path = args[1];
                            match std::fs::read_to_string(file_path) {
                                Ok(contents) => {
                                    for line in contents.lines() {
                                        let trimmed = line.trim();
                                        if !trimmed.is_empty() {
                                            let _ = rl.add_history_entry(trimmed);
                                        }
                                    }
                                    last_written_pos = rl.history().len();
                                }
                                Err(e) => {
                                    eprintln!("history: {}: {}", file_path, e);
                                }
                            }
                        } else if args.len() >= 2 && args[0] == "-w" {
                            // Write all history to file
                            let file_path = args[1];
                            let history: Vec<String> = rl.history().iter().map(|s| s.to_string()).collect();
                            match std::fs::write(file_path, history.join("\n") + "\n") {
                                Ok(_) => {
                                    last_written_pos = history.len();
                                }
                                Err(e) => {
                                    eprintln!("history: {}: {}", file_path, e);
                                }
                            }
                        } else if args.len() >= 2 && args[0] == "-a" {
                            // Append new history to file
                            let file_path = args[1];
                            let history: Vec<String> = rl.history().iter().map(|s| s.to_string()).collect();
                            
                            // Get commands that haven't been written yet (only from current session)
                            let start_pos = last_written_pos;
                            let new_commands: Vec<String> = if start_pos < history.len() {
                                history[start_pos..].to_vec()
                            } else {
                                Vec::new()
                            };
                            
                            if !new_commands.is_empty() {
                                match OpenOptions::new().create(true).append(true).open(file_path) {
                                    Ok(mut file) => {
                                        for command in &new_commands {
                                            if let Err(e) = writeln!(file, "{}", command) {
                                                eprintln!("history: {}: {}", file_path, e);
                                                break;
                                            }
                                        }
                                        last_written_pos = history.len();
                                    }
                                    Err(e) => {
                                        eprintln!("history: {}: {}", file_path, e);
                                    }
                                }
                            }
                        } else {
                            // Regular history command
                            let history: Vec<String> = rl.history().iter().map(|s| s.to_string()).collect(); 
                            cmd_history(&history, args, stdout_file.as_ref());
                        }
                    }
                    // Logic for spawning process
                    _ => {
                        let cmd_name = args[0]; // command name
                        let cmd_args = &args[1..]; // arguments
                        execution::run_external_command(cmd_name, cmd_args, stdout_file.as_ref(), stderr_file.as_ref());
                    }
                }
            }
            Err(ReadlineError::Interrupted | ReadlineError::Eof) => {
                break;
            }
            Err(e) => {
                println!("readline error: {:?}", e);
                break;
            }
        }
    }
}