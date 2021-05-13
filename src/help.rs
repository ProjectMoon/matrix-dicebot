use indoc::indoc;

pub fn parse_help_topic(input: &str) -> Option<HelpTopic> {
    match input {
        "cofd" => Some(HelpTopic::ChroniclesOfDarkness),
        "dicepool" => Some(HelpTopic::DicePool),
        "dice" => Some(HelpTopic::RollingDice),
        "cthulhu" => Some(HelpTopic::Cthulhu),
        "" => Some(HelpTopic::General),
        _ => None,
    }
}

pub enum HelpTopic {
    ChroniclesOfDarkness,
    DicePool,
    Cthulhu,
    RollingDice,
    General,
}

const COFD_HELP: &'static str = indoc! {"
Chronicles of Darkness

Commands available:
 !pool, !rp: roll a dice pool
 !chance: roll a chance die

See also:
 !help dicepool
"};

const DICE_HELP: &'static str = indoc! {"
Rolling basic dice

Command: !roll, !r

Syntax !roll <dice-expression>

Dice expression can be a basic die (e.g. 1d4), with a bonus (1d4+3),
or a more complex series of dice rolls or arbitrary numbers.
Parentheses are not supported.

Examples:
 !roll 1d4
 !roll 1d4+5
 !roll 2d6+8
 !roll 2d8 + 4d6 - 3
"};

const DICEPOOL_HELP: &'static str = indoc! {"
Rolling dice pools

Command: !pool, !rp

Syntax: !pool <modifiers>:<expression>

Short syntax: !pool <expression>

Expression Syntax: <num|variable> [+/- <expression> ...]

Modifiers:
 n = nine-again
 e = eight-again
 r = rote quality
 x = do not re-roll 10s
 s<num> = number of successes for exceptional

Examples:
 !pool 8 (roll a regular pool of 8 dice)
 !pool n:5 (roll dice pool of 5, nine-again)
 !pool rs3:6 (roll dice pool of 6, rote quality, 3 successes for exceptional)
 !pool 10 + 3 (roll dice pool of 10 + 3, which is 13)
 !pool myskill - 4 (roll pool of the value of myskill - 4).
 !pool n:myskill - 5 (roll pool of myskill - 5, with nine-again)
"};

const CTHULHU_HELP: &'static str = indoc! {"
Rolling Call of Cthlhu dice

Commands: !cthroll (regular rolls), !cthadv (advancement rolls)

Regular roll syntax: !cthroll <modifiers>:<num|variable>

Advancement roll syntax: !cthadv <num|variable>

Modifiers:
 b = one bonus die
 bb = two bonus dice
 p = one penalty die
 pp = two penalty dice

Examples:
  !cthroll 60 (make a roll against a skill of 60)
  !cthroll spothidden (make a roll against variable spothidden)
  !cthroll bb:30 (make a roll against skill of 30 with two bonus dice)
  !cthadv 50 (make an advancement roll against a skill of 50)
  !cthadv spothidden (make an advancement roll against the number in spothidden)

Note: If !cthadv is given a variable, and the roll is successful, it will
update the variable with the new skill.
"};

const GENERAL_HELP: &'static str = indoc! {"
General Help

Try these help commands:
  !help cofd
  !help dice
  !help cthulhu
"};

impl HelpTopic {
    pub fn message(&self) -> &str {
        match self {
            HelpTopic::ChroniclesOfDarkness => COFD_HELP,
            HelpTopic::DicePool => DICEPOOL_HELP,
            HelpTopic::Cthulhu => CTHULHU_HELP,
            HelpTopic::RollingDice => DICE_HELP,
            HelpTopic::General => GENERAL_HELP,
        }
    }
}
