use axfive_matrix_dicebot::dice::parser::parse_element_expression;
use axfive_matrix_dicebot::roll::{Roll, Rolled};
use axfive_matrix_dicebot::commands::Command;
use std::error::Error;

fn main() -> Result<(), String> {
    let command = std::env::args().skip(1).collect::<Vec<String>>().join(" ");
    let command: Command = match Command::parse(&command) {
        Ok(command) => command.1,
        Err(e) => return Err(format!("{}", e)),
    };
    println!("{}", command.execute());
    Ok(())
}
