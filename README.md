# axfive-matrix-dicebot

This is a fork of the
[axfive-matrix-dicebot](https://gitlab.com/Taywee/axfive-matrix-dicebot)
with basic support for the Chronicles of Darkness 2E Storytelling
System, with future plans to extend the codebase further to support
variables and perhaps character sheet management.

## Usage

To use it, you can invite the bot to any room you want, and it will
automatically jump in. Then you can simply give a dice expressions for
either the Storytelling System or more traditional RPG dice rolls.

The commands `!roll` and `!r` can handle arbitrary dice roll expressions.

```
!roll 4d6
!r 4d7 + 3
!r 3d12 - 5d2 + 3 - 7d3 + 20d20
```

The commands `!pool` (or `!rp`) and `!chance` are for the Storytelling
System, and they use a specific syntax to support the dice system. The
simplest version of the command is `!pool <num>` to roll a pool of the
given size using the most common type of roll.

The type of roll can be controlled by adding `n`, `e`, or `r` after
the number, for 9-again, 8-again, and rote quality rolls. The number
of successes required for an exceptional success can be controlled by
`s<num>`, e.g. `s3` to only need 3 successes for an exceptional
success.

Examples:

```
!pool 8     //regular pool of 8 dice
!pool 8n    //roll 8 dice, 9-again
!pool 8ns3  //roll 8 dice, 9-again with only 3 successes for exceptional
!pool 5rs2  //5 dice, rote quality, 2 successes for exceptional

```

## Running the Bot
You can run the bot by creating a bot account, building the dicebot
program (either from this repo, and creating a config file that looks
like this:

```ini
[matrix]
home_server = 'matrix.org'

[matrix.login]
password = 'thisismypassword'
type = 'm.login.password'
user = 'axfive-dicebot'
```

Of course replacing all the necessary fields. Then you can run the
"dicebot" binary pointing at that, and it will log in and hum along
and do its thing.

You can also just run it on the command line with the `dicebot-cmd`
command, which expects you to feed it one of the command expressions
as shown above, and will give you the plaintext response.

## Future plans

The most basic plans are:

* To add support for simple per-user variable management, e.g. setting
  a name to a value (`gnosis = 3`) and then using those in dice rolls.
* Perhaps some sort of character sheet integration. But for that, we
  would need a sheet service.
* Robustness fixes if necessary, which will be sent upstream.
