use indoc::indoc;

pub fn parse_help_topic(input: &str) -> Option<HelpTopic> {
    match input {
        "cofd" => Some(HelpTopic::ChroniclesOfDarkness),
        "dicepool" => Some(HelpTopic::DicePool),
        "dice" => Some(HelpTopic::RollingDice),
        "" => Some(HelpTopic::General),
        _ => None,
    }
}

pub enum HelpTopic {
    ChroniclesOfDarkness,
    DicePool,
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

Syntax: !pool <num><modifiers>

Modifiers:
 n = nine-again
 e = eight-again
 r = rote quality
 x = do not re-roll 10s
 s<num> = number of successes for exceptional

Examples:
 !pool 8 (roll a regular pool of 8 dice)
 !pool 5n (roll dice pool of 5, nine-again)
 !pool 6rs3 (roll dice pool of 6, rote quality, 3 successes for exceptional)
"};

const GENERAL_HELP: &'static str = indoc! {"
General Help

Try these help commands:
  !help cofd
  !help dice
"};

impl HelpTopic {
    pub fn message(&self) -> &str {
        match self {
            HelpTopic::ChroniclesOfDarkness => COFD_HELP,
            HelpTopic::DicePool => DICEPOOL_HELP,
            HelpTopic::RollingDice => DICE_HELP,
            HelpTopic::General => GENERAL_HELP,
        }
    }
}
