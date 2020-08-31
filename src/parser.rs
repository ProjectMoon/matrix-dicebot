use nom::{bytes::complete::take_while, IResult};

fn is_whitespace(input: char) -> bool {
    input == ' ' || input == '\n' || input == '\t' || input == '\r'
}

/// Eat whitespace, returning it
pub fn eat_whitespace(input: &str) -> IResult<&str, &str> {
    let (input, whitespace) = take_while(is_whitespace)(input)?;
    Ok((input, whitespace))
}

/// Remove the whitespace on the ends of the string.
pub fn trim(input: &str) -> String {
    //2 allocations, how fun
    String::from(input).trim().to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_trim_test() {
        assert_eq!(String::from("blah"), trim("   blah   "));
    }

    #[test]
    fn trim_only_removes_ends_test() {
        assert_eq!(String::from("b l a h"), trim("   b l a h   "));
    }
}
