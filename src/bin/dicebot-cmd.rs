use axfive_matrix_dicebot::commands::parse_command;

fn main() -> Result<(), String> {
    let command = std::env::args().skip(1).collect::<Vec<String>>().join(" ");
    let command = match parse_command(&command) {
        Ok(Some(command)) => command,
        Ok(None) => return Err("Command not recognized".into()),
        Err(e) => return Err(format!("Error parsing command: {}", e)),
    };
    println!("{}", command.execute().plain());
    Ok(())
}
