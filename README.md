# matrix-dicebot

This is a fork of the
[axfive-matrix-dicebot](https://gitlab.com/Taywee/axfive-matrix-dicebot)
with basic support for the Chronicles of Darkness 2E Storytelling
System, with future plans to extend the codebase further to support
variables and perhaps character sheet management.

## Features

`matrix-dicebot` is a basic dice rolling bot. It currently has the
following features:

* Rolling arbitrary dice expressions (e.g. 1d4, 1d20+5, 1d8+1d6, etc).
* Rolling dice pools for the Chronicles of Darkness 2E Storytelling
System.
* Works in encrypted or unencrypted Matrix rooms.

## Building and Installation

### Running with Docker

The dice bot can run in a minimal Docker image based on [Void
Linux](https://voidlinux.org/). You can either pull it from [Docker
Hub](https://hub.docker.com/r/projectmoon/chronicle-dicebot) or build
it yourself. The easiest way is to use the prebuilt image. It is
updated on Docker Hub via a CI/CD pipeline. The `latest` tag always
points to the most recent successful master commit and is considered
unstable, while individual tags are considered stable:

* Unstable: `docker pull ghcr.io/projectmoon/chronicle-dicebot:latest`
* Stable: `docker pull ghcr.io/projectmoon/chronicle-dicebot:X.Y.Z`

To create the Docker image, run `docker build -t chronicle-dicebot .`
in the root of the repository.

After pulling or building the image, see [instructions on how to use
the Docker image](#docker-image).

### Build from Source

Precompiled executables are not yet available. Clone this repository
and run `OLM_LINK_VARIANT=dylib cargo install`.

Building the project requires:

* Basic build environment (`build-essential` on Ubuntu, `base-devel`
  on Void and Arch, etc).
* Rust 1.45.0 or higher.
* OpenSSL/LibreSSL development headers installed.
* `olm-sys` crate dependencies: cmake, libstdc++, libolm and its
  development headers.
* glibc.

Note: The `olm-sys` crate must be built in dynamic linking mode until
a [bug][1] in its build process is fixed.

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

The commands `!roll` and `!r` can handle arbitrary dice roll
expressions.

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

You can run the bot by creating a Matrix account for it, building the
application, and creating a config file that looks like this:

```ini
[matrix]
home_server = https://example.com'
username = 'thisismyusername'
password = 'thisismypassword'
```

Make sure to replace the information with your own. Then you can run
the "dicebot" binary. It takes the path to the configuration file as
its single argument.

You can also run it on the command line with the `dicebot-cmd`
command, which expects you to feed it one of the command expressions
as shown above, and will give you the plaintext response.

### Docker Image

A typical docker run command using the official Docker image should
look something like this:

```
VERSION="latest"
docker run --rm -d --name dicebot \
-v /path/to/dicebot-config.toml:/config/dicebot-config.toml:ro \
-v /path/to/cache/:/cache \
ghcr.io/projectmoon/chronicle-dicebot:$VERSION
```

The Docker image requires two volume mounts: the location of the
config file, which should be mounted at `/config/dicebot-config.toml`,
and a cache directory to store client state after initial sync. That
should be mounted at `/cache/`in the container.

## Future plans

The most basic plans are:

* To add support for simple per-user variable management, e.g. setting
  a name to a value (`gnosis = 3`) and then using those in dice rolls.
* Perhaps some sort of character sheet integration. But for that, we
  would need a sheet service.
* Automation of Docker builds and precompiled binaries.
* Use environment variables instead of config file in Docker image.

[1]: https://gitlab.gnome.org/BrainBlasted/olm-sys/-/issues/6
