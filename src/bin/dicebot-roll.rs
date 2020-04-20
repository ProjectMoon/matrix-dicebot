use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let _roll_string = std::env::args().skip(1).collect::<Vec<String>>().join(" ");
    // first regex needs to be different because the sign is mandatory for the rest
    //let expression = parse_expression(&roll_string);
    //println!("{:?}", expression);
    Ok(())
}
