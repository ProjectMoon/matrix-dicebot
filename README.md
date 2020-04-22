# axfive-matrix-dicebot

Just a simple dicebot for matrix written in Rust.  It might have some sort of
stability issues.  I haven't seen it die in the wild, but it's mostly
hacked-together code that has been pretty lightly tested and reviewed, and I'm
certain that some things are still left undone, like interacting properly with
rate-limiting, and dealing with any sort of sporadic error (like being suddenly
deauthenticated and having to reauthenticate).

This was mostly built as a fun way of

* Experimenting with writing an async program in Rust that does something real
* Experimenting with writing a bot for Matrix
* Experimenting with writing a simple parser using [nom](https://github.com/Geal/nom)

All of which was new territory for me.

## Usage

To use it, you can invite the bot (@axfive-dicebot:matrix.org) to any room you
want, and it will automatically jump in.  Then you can simply give a dice
expression as such:

```
!roll 4d6
!r 4d7 + 3
!r 3d12 - 5d2 + 3 - 7d3 + 20d20
```

And the dicebot should reply with the result in short order.

You can also run it yourself by creating a bot account, building the dicebot
program (either from this repo or by running `cargo install
axfive-matrix-dicebot`, and creating a config file that looks like this:

```ini
[matrix]
home_server = 'matrix.org'

[matrix.login]
password = 'thisismypassword'
type = 'm.login.password'
user = 'axfive-dicebot'
```

Of course replacing all the necessary fields.  Then you can run the "dicebot"
binary pointing at that, and it will log in and hum along and do its thing.

You can also just run it on the command line with the `dicebot-cmd` command,
which expects you to feed it one of the command expressions as shown above, and
will give you the plaintext response.

## Future plans

None, really.  This is not a very serious project and I'm not planning on doing
much heavy maintenance or anything of the sort.  This was mostly for fun.  If I
get some motivation to work it up, I might at some point do some things like:

* Actually have it handle rate-limiting and other errors properly.
* Add more syntax to the dice expressions, maybe making it possibly have more
  features that [Avrae](https://avrae.io/commands#roll) offers.
* Potentially add more commands.

I would happily accept any sort of pull requests for extra functionality or more
robustness if anybody wants to actually use it.  The code is built to hopefully
be relatively easy to extend (with things like new commands).
