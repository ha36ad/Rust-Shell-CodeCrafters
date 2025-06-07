use std::env;
use std::path::Path;
use crate::utils::print_or_write;

pub fn cmd_echo(input: &[&str]) -> String {
    input.join(" ")
}

pub fn cmd_cd(args: &[&str]) {
    if args.is_empty() {
        return;
    }
    let path = args[0];
    let target_path = path.replace("~", &env::var("HOME").unwrap_or_default());
    if !env::set_current_dir(target_path).is_ok() {
        println!("cd: {}: No such file or directory", path);
    }
}

pub fn cmd_type(args: &[&str], output_file: Option<&(String, bool)>) {
    if args.is_empty() {
        return;
    }
    let output = execute_type_builtin(args);
    print_or_write(output_file, &output);
}

pub fn execute_type_builtin(args: &[&str]) -> String {
    if args.is_empty() {
        return String::new();
    }
    
    let builtins = vec!["echo", "exit", "type", "pwd","history", "cd"];
    let command = args[0];

    if builtins.contains(&command) {
        format!("{} is a shell builtin", command)
    } else if let Ok(path_var) = env::var("PATH") {
        for dir in path_var.split(':') {
            let full_path = Path::new(dir).join(command);
            if full_path.exists() {
                return format!("{} is {}", command, full_path.display());
            }
        }
        format!("{}: not found", command)
    } else {
        format!("{}: not found", command)
    }
}

pub fn execute_builtin(cmd: &[&str]) -> String {
    match cmd[0] {
        "echo" => {
            if cmd.len() > 1 {
                cmd_echo(&cmd[1..])
            } else {
                String::new()
            }
        }
        "pwd" => {
            env::current_dir()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|_| String::new())
        }
        "type" => {
            if cmd.len() > 1 {
                execute_type_builtin(&cmd[1..])
            } else {
                String::new()
            }
        }
        _ => String::new(),
    }
}


pub fn cmd_history(history_entries: &[String], args: &[&str], output_file: Option<&(String, bool)>) {
    let entries_to_show = if args.is_empty() {
        // Show all history if no limit specified
        history_entries
    } else {
        // Parse the limit argument
        match args[0].parse::<usize>() {
            Ok(n) => {
                if n >= history_entries.len() {
                    history_entries
                } else { // Show last n entries
                    &history_entries[history_entries.len() - n..]
                }
            }
            Err(_) => {
                history_entries
            }
        }
    };

    // Calculate the starting index
    let start_index = if args.is_empty() {
        1
    } else {
        match args[0].parse::<usize>() {
            Ok(n) => {
                if n >= history_entries.len() {
                    1
                } else {
                    history_entries.len() - n + 1
                }
            }
            Err(_) => 1,
        }
    };

    for (i, entry) in entries_to_show.iter().enumerate() {
        let line = format!("    {}  {}", start_index + i, entry);
        print_or_write(output_file, &line);
    }
}