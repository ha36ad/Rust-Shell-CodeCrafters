pub fn parse_redirection(input: &str) -> (String, Option<(String, bool)>, Option<(String, bool)>) {
    let mut command_str = input.to_string();
    let mut stdout_file = None;
    let mut stderr_file = None;

    // Check for stderr redirection
    if let Some(redirect_pos) = command_str.find(" 2>> ") {
        let (cmd_part, file_part) = command_str.split_at(redirect_pos);
        let file_path = file_part.trim_start_matches(" 2>> ").trim().split_whitespace().next().unwrap_or("");
        stderr_file = Some((file_path.to_string(), true));
        command_str = cmd_part.to_string();
    } else if let Some(redirect_pos) = command_str.find(" 2> ") {
        let (cmd_part, file_part) = command_str.split_at(redirect_pos);
        let file_path = file_part.trim_start_matches(" 2> ").trim().split_whitespace().next().unwrap_or("");
        stderr_file = Some((file_path.to_string(), false));
        command_str = cmd_part.to_string();
    }

    // Check for stdout redirection
    if let Some(redirect_pos) = command_str.find(" 1>> ") {
        let (cmd_part, file_part) = command_str.split_at(redirect_pos);
        let file_path = file_part.trim_start_matches(" 1>> ").trim().split_whitespace().next().unwrap_or("");
        stdout_file = Some((file_path.to_string(), true));
        command_str = cmd_part.to_string();
    } else if let Some(redirect_pos) = command_str.find(" >> ") {
        let (cmd_part, file_part) = command_str.split_at(redirect_pos);
        let file_path = file_part.trim_start_matches(" >> ").trim().split_whitespace().next().unwrap_or("");
        stdout_file = Some((file_path.to_string(), true));
        command_str = cmd_part.to_string();
    } else if let Some(redirect_pos) = command_str.find(" 1> ") {
        let (cmd_part, file_part) = command_str.split_at(redirect_pos);
        let file_path = file_part.trim_start_matches(" 1> ").trim().split_whitespace().next().unwrap_or("");
        stdout_file = Some((file_path.to_string(), false));
        command_str = cmd_part.to_string();
    } else if let Some(redirect_pos) = command_str.find(" > ") {
        let (cmd_part, file_part) = command_str.split_at(redirect_pos);
        let file_path = file_part.trim_start_matches(" > ").trim().split_whitespace().next().unwrap_or("");
        stdout_file = Some((file_path.to_string(), false));
        command_str = cmd_part.to_string();
    }

    (command_str, stdout_file, stderr_file)
}