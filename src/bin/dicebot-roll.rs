use regex::Regex;
use std::error::Error;
use std::fmt;
use std::str::FromStr;

#[derive(Debug)]
struct ParseError {
    reason: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ParseError: {}", self.reason)
    }
}

impl Error for ParseError {}

#[derive(Debug)]
struct Dice {
    count: u32,
    sides: u32,
}

#[derive(Debug)]
enum Roll {
    Dice(Dice),
    Bonus(u32),
}

impl From<u32> for Roll {
    fn from(value: u32) -> Roll {
        Roll::Bonus(value)
    }
}

impl From<Dice> for Roll {
    fn from(value: Dice) -> Roll {
        Roll::Dice(value)
    }
}

impl FromStr for Roll {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let regex = Regex::new(r"(?i)^(\d+)\s*(d\s*(\d+))?$")?;
        let captures = match regex.captures(s) {
            Some(captures) => captures,
            None => {
                return Err(ParseError {
                    reason: format!("{:?} is not a legal Roll Part expression", s),
                }
                .into())
            }
        };

        match captures.get(2) {
            Some(_) => Ok(Dice {
                count: captures.get(1).unwrap().as_str().parse()?,
                sides: captures.get(3).unwrap().as_str().parse()?,
            }
            .into()),
            None => Ok(Roll::Bonus(captures.get(1).unwrap().as_str().parse()?)),
        }
    }
}

#[derive(Debug)]
enum Part {
    Plus(Roll),
    Minus(Roll),
}

impl FromStr for Part {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        match s.chars().next() {
            Some('+') => Ok(Part::Plus(s[1..].parse()?)),
            Some('-') => Ok(Part::Minus(s[1..].parse()?)),
            Some(_) => Ok(Part::Plus(s.parse()?)),
            None => Err(ParseError {
                reason: format!("{:?} is not a legal Roll Part expression", s),
            }
            .into()),
        }
    }
}

#[derive(Debug)]
struct Expression(Vec<Part>);

impl FromStr for Expression {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let validation_regex = Regex::new(
            r"(?xi)^
        ([-\+])?\s*(\d+)\s*(d\s*(\d+))?
        (\s*([-\+])\s*(\d+)\s*(d\s*(\d+))?)*
        $",
        )?;
        if !validation_regex.is_match(s) {
            return Err(ParseError {
                reason: format!("{:?} is not a legal dice expression", s),
            }
            .into());
        }

        let part = Regex::new(r"(?i)[-\+]?\s*\d+\s*(d\s*(\d+))?")?;

        let results: Result<_, _> = part.find_iter(s).map(|p| p.as_str().parse()).collect();
        Ok(Expression(results?))
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let roll_string = std::env::args().skip(1).collect::<Vec<String>>().join(" ");
    // first regex needs to be different because the sign is mandatory for the rest
    let expression: Expression = roll_string.parse()?;
    println!("{:?}", expression);
    Ok(())
}
