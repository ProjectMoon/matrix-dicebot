# matrix-dicebot

[![Build Status](https://drone.agnos.is/api/badges/projectmoon/matrix-dicebot/status.svg)](https://drone.agnos.is/projectmoon/matrix-dicebot)

_This repository is hosted on [Agnos.is Git][main-repo] and mirrored
to [GitHub][github-repo]._

This is a dice rolling bot for facilitating roleplaying games on the
Matrix messaging platform. It currently has basic support for the
Chronicles of Darkness 2E Storytelling System and Call of Cthulhu,
with plans to extend the codebase further to support other systems and
character sheet management.

## Features

`matrix-dicebot` is a basic dice rolling bot. It currently has the
following features:

* Rolling arbitrary dice expressions (e.g. 1d4, 1d20+5, 1d8+1d6, etc).
* Rolling dice pools for the Chronicles of Darkness 2E Storytelling
System.
* Rolling dice for the Call of Cthulhu system.
* Works in encrypted or unencrypted Matrix rooms.
* Storing variables created by the user.

## Building and Installation

### Docker Image

The easiest way to run the dice bot is to use the [official Docker
image][docker-image]. It is distributed on GitHub Container Registry
by a CI pipeline.

The `latest` tag always points to the most recent successfully built
master commit and is considered unstable, while individual tags are
considered stable.

* Unstable: `docker pull ghcr.io/projectmoon/chronicle-dicebot:latest`
* Stable: `docker pull ghcr.io/projectmoon/chronicle-dicebot:X.Y.Z`

This image is based on [Void Linux](https://voidlinux.org/). To build
the image yourself, run `docker build -t chronicle-dicebot .` in the
root of the repository.

After pulling or building the image, see [instructions on how to use
the Docker image](#running-the-bot).

### Build from Source

Precompiled executables are not yet available. Clone this repository
and run `cargo install`.

Building the project requires:

* Basic build environment (`build-essential` on Ubuntu, `base-devel`
  on Void and Arch, etc).
* Rust 1.45.0 or higher.
* OpenSSL/LibreSSL development headers installed.
* `olm-sys` crate dependencies: cmake, libstdc++.
* glibc.

#### Why doesn't it build on musl libc?

As far as I can tell, the project doesn't build on musl libc. It
certainly doesn't build a static binary out of the box using the
rust-musl-builder. This appears to be due to a transitive dependency
of the Rust Matrix SDK.

Any PRs to get the project or Matrix SDK to properly be built into a
static binary using musl would be very useful.

## Usage

To use it, you can invite the bot to any room you want, and it will
automatically jump in. Then you can simply give a dice expressions for
either the Storytelling System or more traditional RPG dice rolls.

The bot supports a `!help` command for basic help information about
its capabilities.

### Basic Dice Rolling
The commands `!roll` and `!r` can handle arbitrary dice roll
expressions.

```
!roll 4d6
!r 4d7 + 3
!r 3d12 - 5d2 + 3 - 7d3 + 20d20
```

This system does not yet have the capability to handle things like D&D
5e advantage or disadvantage.

### Storytelling System

The commands `!pool` (or `!rp`) and `!chance` are for the Storytelling
System, and they use a specific syntax to support the dice system. The
simplest version of the command is `!pool <num>` to roll a pool of the
given size using the most common type of roll.

The type of roll can be controlled by adding `n`, `e`, or `r` before
the number, for 9-again, 8-again, and rote quality rolls. The number
of successes required for an exceptional success can be controlled by
`s<num>`, e.g. `s3` to only need 3 successes for an exceptional
success. All modifiers should come before the number, with a `:`
colon.

Examples:

```
!pool 8      //regular pool of 8 dice
!pool n:8    //roll 8 dice, 9-again
!pool ns3:8  //roll 8 dice, 9-again with only 3 successes for exceptional
!pool rs2:5  //5 dice, rote quality, 2 successes for exceptional
```

### Call of Cthulhu System

The commands `!cthRoll`, `!cthroll`, `!cthARoll` and `!cthadv` are for
the Call of Cthulhu system. `!cthRoll` and `!cthroll` are for rolling
percentile dice against a target number. A `b:` or `bb:` can be
prepended to get one or two bonus dice.

`!cthARoll` and `!cthadv` are for skill advancement.

Examples:

```
!cthRoll 50     //roll against a target of 50
!cthRoll bb:60  //roll against a target of 60 with 2 bonus dice
!cthARoll 30    //advancement roll against a target of 30
```

### User Variables

Users can store variables for use with the Storytelling dice pool
system. Variables are stored on a per-room, per-user basis in the
database (currently located in the cache directory if using the Docker
image).

Examples:

```
!set myvar 5 //stores 5 for this room under the name "myvar"
!get myvar //will print 5
```

Variables can be referenced in dice pool and Call of Cthulhu rolling
expressions, for example `!pool myvar` or `!pool myvar+3` or `!cthroll
myvar`. The Call of Cthulhu advancement roll also accepts variables,
and if a variable is used, and the roll is successful, it will update
the variable with the new skill.

## Running the Bot

The easiest way to run the bot is to use the [official Docker
image][docker-image], although you can also [run the binary
directly](#running-binary-directly).

A typical docker run command using the official Docker image should
look something like this:

```
# Run unstable version of the bot
VERSION="latest"
docker run --rm -d --name dicebot \
-v /path/to/dicebot-config.toml:/config/dicebot-config.toml:ro \
-v /path/to/cache/:/cache \
ghcr.io/projectmoon/chronicle-dicebot:$VERSION
```

The Docker image requires two volume mounts: the location of the
[config file][config-file], which should be mounted at
`/config/dicebot-config.toml`, and a cache directory to store the
database and client state after initial sync. That should be mounted
at `/cache/`in the container.

### Configuration File

The configuration file is a TOML file with three sections.

```toml
[matrix]
home_server = 'https://example.com'
username = 'thisismyusername'
password = 'thisismypassword'

[database]
path = '/path/to/database/directory/'

[bot]
oldest_message_age = 300
```

The `[matrix]` section contains the information for logging in to the
bot's matrix account.

 - `home_server`: The URL for the Matrix homeserver the bot should log
   in to. This should be the proper hostname of the homeserver that
   you would enter into the login box, which might be different than
   the server name that is displayed to other users.
 - `username`: Bot account username.
 - `password`: Bot account password.

The `[database]` section contains information for connecting to the
embedded database. Note: **you do not need this** if you are using the
Docker image.
 - `path`: Path on the filesystem to use as the database storage
   directory.

The `[bot]` section has settings for controlling how the bot operates.
This section is optional and the settings will fall back to their
default values if the section or setting is not present.

 - `oldest_message_age`: the oldest time (in seconds) in the past that
   a message can be before being ignored. This prevents the bot from
   processing out-of-context old commands received while offline. The
   default value is 900 seconds (15 minutes).

### Running Binary Directly

If you have [built the application from source](#build-from-source),
you can invoke the dice bot directly instead of using Docker by
running `dicebot /path/to/config.toml`. By default, the user account
cache is stored in a [platform-dependent location][dirs]. If you want
to change the cache location on Linux, for example, you can run
`export XDG_CACHE_HOME=/path/to/cache` before invoking the bot.

Installing the application directly also installs `dicebot-cmd`, which
allows you to run arbitrary bot commands on the command line. This
does not connect to a running instance of the bot; it just processes
commands locally.

## Future plans

The most basic plans are:

* Resource counting: creation of custom counters that can go up and
  down.
* Perhaps some sort of character sheet integration. But for that, we
  would need a sheet service.
* Use environment variables instead of config file in Docker image.

## Credits

This was orignally a fork of the [axfive-matrix-dicebot][axfive], with
support added for Chronicles of Darkness and Call of Cthulhu.

[axfive]: https://gitlab.com/Taywee/axfive-matrix-dicebot
[config-file]: #Configuration-File
[docker-image]: https://github.com/users/ProjectMoon/packages/container/package/chronicle-dicebot
[dirs]: https://docs.rs/dirs/2.0.2/dirs/
[main-repo]: https://git.agnos.is/projectmoon/matrix-dicebot
[github-repo]: https://github.com/ProjectMoon/matrix-dicebot
[roadmap]: https://git.agnos.is/projectmoon/matrix-dicebot/wiki/Roadmap
