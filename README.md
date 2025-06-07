# Rust Shell (rsh)

A command-line shell implementation written in Rust. This was for the CodeCrafters [Build your Own Shell Challenge](https://app.codecrafters.io/courses/shell/overview)

## Features

### Core Shell Functionality
- **Interactive Command Line**: Clean, responsive prompt with readline support
- **Command History**: Full history management with persistence
- **Built-in Commands**: Essential shell built-ins (echo, cd, pwd, type, exit)
- **External Command Execution**: Run any executable in your PATH
- **Pipeline Support**: Chain commands with pipes (`|`)
- **File Redirection**: Comprehensive I/O redirection support

### Advanced Tab Completion
- **Smart Autocompletion**: Intelligent completion for built-ins and executables
- **PATH-aware**: Automatically discovers and suggests executables from your PATH
- **Multi-match Handling**: 
  - First tab: Bell notification for multiple matches
  - Second tab: Display all available options
  - Automatic completion to longest common prefix
- **Context-aware**: Different completion behavior based on command position

### Built-in Commands

#### `echo [args...]`
Print arguments to stdout with space separation.
```bash
$ echo Hello World
Hello World
```

#### `cd [directory]`
Change current working directory. Supports `~` for home directory.
```bash
$ cd ~/Documents
$ cd /usr/local/bin
```

#### `pwd`
Print the current working directory.
```bash
$ pwd
/home/user/projects
```

#### `type <command>`
Display information about a command (built-in vs external executable).
```bash
$ type echo
echo is a shell builtin
$ type ls
ls is /usr/bin/ls
```

#### `exit 0`
Exit the shell gracefully.

#### `history [options] [file]`
Manage command history with various options:
- `history`: Display all history entries
- `history -r <file>`: Read history from file
- `history -w <file>`: Write all history to file  
- `history -a <file>`: Append new history entries to file

## Installation

### Prerequisites
- Rust 1.80+ (with Cargo)
- Unix-like operating system (Linux, macOS)

### Dependencies
The shell uses several Rust crates:
- `rustyline`: For readline functionality and command history
- `shlex`: For proper shell argument parsing
- Standard library modules for file I/O and process management

## Usage

### Starting the Shell
```bash
$ ./run.sh
$ 
```
## Limitations

- Currently Unix/Linux only (uses Unix-specific file permissions)
- No job control (background processes, job management)  
- No shell scripting support (variables, conditionals, loops)
- No glob expansion (*, ?, [])
- No command substitution
- Limited to basic POSIX-style redirection
