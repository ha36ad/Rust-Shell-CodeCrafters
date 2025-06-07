use std::env;
use std::cell::RefCell;
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::{Context, Helper};
use rustyline::highlight::Highlighter;
use rustyline::validate::Validator;
use crate::utils::is_executable;
use std::io::Write;

pub struct MyCompleter {
    last_line: RefCell<String>,
    last_pos: RefCell<usize>,
    tab_count: RefCell<usize>,
}

impl MyCompleter {
    pub fn new() -> Self {
        MyCompleter {
            last_line: RefCell::new(String::new()),
            last_pos: RefCell::new(0),
            tab_count: RefCell::new(0),
        }
    }
}

impl Completer for MyCompleter {
    type Candidate = Pair;

    fn complete(&self, line: &str, pos: usize, _ctx: &Context<'_>)
        -> Result<(usize, Vec<Pair>), ReadlineError>
    {
        let mut candidates = Vec::new();

        // Check if this is the same completion context as last time
        let same_context = *self.last_line.borrow() == line && *self.last_pos.borrow() == pos;
        
        if same_context {
            *self.tab_count.borrow_mut() += 1;
        } else {
            *self.tab_count.borrow_mut() = 1;
            *self.last_line.borrow_mut() = line.to_string();
            *self.last_pos.borrow_mut() = pos;
        }

        // Get current word fragment
        let (before_cursor, _) = line.split_at(pos);
        let tokens: Vec<&str> = before_cursor.split_whitespace().collect();
        let last_token = tokens.last().unwrap_or(&"");

        // Complete built-in commands only if we're at the beginning
        if tokens.len() <= 1 {
            let builtins = vec!["echo", "exit", "type", "pwd", "cd"];
            for cmd in builtins {
                if cmd.starts_with(last_token) {
                    candidates.push(Pair {
                        display: format!("{} ", cmd),
                        replacement: format!("{} ", cmd),
                    });
                }
            }
        }

        // Path execution - find all matching executables
        let mut executable_matches = Vec::new();
        if let Ok(path_env) = env::var("PATH") {
            for dir in path_env.split(':') {
                if let Ok(entries) = std::fs::read_dir(dir) {
                    for entry in entries.flatten() {
                        if let Ok(file_name) = entry.file_name().into_string() {
                            if file_name.starts_with(last_token) && entry.path().is_file() && is_executable(&entry.path()) {
                                if !executable_matches.contains(&file_name) {
                                    executable_matches.push(file_name.clone());
                                }
                            }
                        }
                    }
                }
            }
        }

        // Sort the matches for consistent output
        executable_matches.sort();

        // Handle completion logic
        if executable_matches.len() > 1 && !last_token.is_empty() {
            // Find the longest common prefix
            let common_prefix = find_longest_common_prefix(&executable_matches);
            
            // If the common prefix is longer than what we currently have, complete to it
            if common_prefix.len() > last_token.len() {
                candidates.push(Pair {
                    display: common_prefix.clone(),
                    replacement: common_prefix,
                });
                return Ok((pos - last_token.len(), candidates));
            }
            
            // If no additional completion possible, handle multiple matches with tab behavior
            let tab_count = *self.tab_count.borrow();
            
            if tab_count == 1 {
                // First tab: ring bell
                print!("\x07"); // Bell character
                std::io::stdout().flush().unwrap();
                return Ok((pos, vec![])); // Return empty to prevent rustyline from doing anything else
            } else if tab_count >= 2 {
                // Second tab: display matches
                let matches_str = executable_matches.join("  ");
                println!("\n{}", matches_str);
                print!("$ {}", line);
                std::io::stdout().flush().unwrap();
                return Ok((pos, vec![])); // Return empty to prevent rustyline from doing anything else
            }
        }

        // Add executable matches to candidates for normal completion
        for file_name in executable_matches {
            candidates.push(Pair {
                display: format!("{} ", file_name),
                replacement: format!("{} ", file_name),
            });
        }

        Ok((pos - last_token.len(), candidates))
    }
}

// Implement all required traits for rustyline::Helper
impl Helper for MyCompleter {}
impl rustyline::hint::Hinter for MyCompleter {
    type Hint = String;
    fn hint(&self, _line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<String> {
        None
    }
}
impl Highlighter for MyCompleter {}
impl Validator for MyCompleter {}

fn find_longest_common_prefix(strings: &[String]) -> String {
    if strings.is_empty() {
        return String::new();
    }
    
    if strings.len() == 1 {
        return strings[0].clone();
    }
    
    let first = &strings[0];
    let mut prefix_len = first.len();
    
    for s in strings.iter().skip(1) {
        let common_len = first.chars()
            .zip(s.chars())
            .take_while(|(a, b)| a == b)
            .count();
        prefix_len = prefix_len.min(common_len);
    }
    
    first.chars().take(prefix_len).collect()
}
