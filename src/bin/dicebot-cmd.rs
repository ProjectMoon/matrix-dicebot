use axfive_matrix_dicebot::dice::parser::parse_element_expression;
use axfive_matrix_dicebot::roll::{Roll, Rolled};
use axfive_matrix_dicebot::commands::parse_command;
use std::error::Error;
use std::string::ToString;

fn main() -> Result<(), String> {
    let command = std::env::args().skip(1).collect::<Vec<String>>().join(" ");
    let command = match parse_command(&command) {
        Some(Ok(command)) => command,
        Some(Err(e)) => return Err(format!("Error parsing command: {}", e)),
        None => return Err("Command not recognized".into()),
    };
    println!("{}", command.execute().plain());
    Ok(())
}
