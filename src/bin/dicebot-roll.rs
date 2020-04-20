use axfive_matrix_dicebot::dice::parser::parse_element_expression;
use axfive_matrix_dicebot::roll::{Roll, Rolled};
use std::error::Error;

fn main() -> Result<(), String> {
    let roll_string = std::env::args().skip(1).collect::<Vec<String>>().join(" ");
    let (_tail, expression) = match parse_element_expression(&roll_string) {
        Ok(response) => response,
        Err(e) => return Err(e.to_string()),
    };
    println!("{}", expression.roll());
    Ok(())
}
