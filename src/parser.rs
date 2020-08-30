use nom::{bytes::complete::take_while, IResult};

fn is_whitespace(input: char) -> bool {
    input == ' ' || input == '\n' || input == '\t' || input == '\r'
}

/// Eat whitespace, returning it
pub fn eat_whitespace(input: &str) -> IResult<&str, &str> {
    let (input, whitespace) = take_while(is_whitespace)(input)?;
    Ok((input, whitespace))
}

pub fn trim(input: &str) -> String {
    input.chars().filter(|c| !c.is_whitespace()).collect()
}
