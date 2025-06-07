use std::path::Path;
use std::fs::{File, OpenOptions};
use std::io::Write;

pub fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.metadata()
        .map(|meta| meta.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

pub fn write_to_file(file_path: &str, content: &str, append: bool) {
    if let Some(parent) = Path::new(file_path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let result = if append {
        OpenOptions::new().create(true).append(true).open(file_path)
    } else {
        File::create(file_path)
    };
    match result {
        Ok(mut file) => {
            let _ = writeln!(file, "{}", content);
        }
        Err(e) => {
            println!("Error writing to file {}: {}", file_path, e);
        }
    }
}

pub fn print_or_write(output: Option<&(String, bool)>, message: &str) {
    if let Some((file_path, append)) = output {
        write_to_file(file_path, message, *append);
    } else {
        println!("{}", message);
    }
}