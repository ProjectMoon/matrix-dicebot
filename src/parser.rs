use nom::{
    bytes::complete::take_while,
    IResult,
};

fn is_whitespace(input: char) -> bool {
    input == ' ' || input == '\n' || input == '\t' || input == '\r'
}

pub fn eat_whitespace(input: &str) -> IResult<&str, ()> {
    let (input, _) = take_while(is_whitespace)(input)?;
    Ok((input, ()))
}
