# Network play

*Category: [Getting started](Getting-started)*

Several machines can race each other over a local network or the internet. The game's
network screens are the 1999 game's; what carries the messages underneath is new.

## What differs from the 1999 game

- The original used DirectPlay. This program has **its own protocol** in its place, so it
  plays against other copies of this program only, not against the original `RAL.EXE`.
- Only one kind of connection is offered: TCP/IP. The modem, serial and IPX connections of
  1999 are not there.
- There are no lobby services; one machine hosts and the others join it directly.
- The protocol is the same on every system the program runs on, so builds for different
  systems are meant to play each other. Use the same version of the program on all machines: a machine of another protocol version
  is refused (an older host does not show up in the session list, and ending the connection
  at a join is read as an older host; both end up in `rc2k.log`).
- Several of the original's network bugs are fixed, among them remote cars that jerked and
  jumped back along the road ([Fixed bugs](Fixed-bugs)).

## Before you start

- Every machine needs the program and the game's files, ideally the same build.
- The machines must reach the host on **port 47624** (`0xB9E8`), both TCP and UDP. On a home
  network that usually just works; a firewall on the host may need the port opened. To host
  over the internet, forward that port (TCP and UDP) to the hosting machine.
- A different port can be set on the network settings page (the "port" row below the host's
  address: Enter, type the number, Enter), for hosting and for joining; it is kept in
  `var/rc2k.toml` (`port` in the `network` table, which can also be edited by hand); `0` means
  the default. All machines must use the same one.

## Setting up a game

In the main menu choose the network game, then host or join.

**The network settings screen** comes first:

| Item | What it is |
| --- | --- |
| Connection | The kind of connection; there is one, TCP/IP. |
| Player's name | Your name in the lobby and the results. |
| Most players | The host's limit on how many may join. |
| Host's address | For joining: the host's address. Leave it empty to look for games on the local network. |

**Hosting.** The host makes the session and waits in the lobby. Its machine must stay in the
game: when the host leaves, the session ends for everybody (it is not handed to another
machine).

**Joining.** The program lists the sessions it finds. With no address given it asks the whole
local network (and this machine itself); with an address it asks that machine only. Choose a
session to join it. Full or closed sessions are not listed.

**The lobby.** Everyone chooses a car; the host chooses the stage, and its name is shown at
the top left of everybody's lobby. There is a chat line. When every player is ready the host
starts the race.

**Putting a player out.** The host types `/kick <name>` in the chat line (the first other player
of that name, case ignored). The line is not sent to the others; the host sees an answer line
in its chat. The player is told and leaves the lobby as if the host had gone, and may join again.

**The race.** Each machine simulates its own car and sends its inputs and position; the others
show it. The stage bar along the top shows every car as a marker, each in its own colour, the
own car's in white. Afterwards the results come up and the lobby returns for the next stage.

## From the command line

`rc2k --host` and `rc2k --join <address>` go straight to the lobby, with `--name` and `--port`
for this run: see [Running and options](Running-and-options).

## Two copies on one machine

For trying it out, two copies can run on the same computer: host in one, and in the other join
with the address left empty (this machine is always asked) or set to `127.0.0.1`. Give each
copy its own game folder, or they will share `var` and `savedata`.

## If it does not work

- No session listed: check the firewall and the port on the host; over the internet, give the
  host's address rather than leaving it empty (a broadcast does not leave the local network).
- The session disappears: the host left or lost its connection.
- Anything else: both machines' `rc2k.log` say what the network layer saw
  ([Running and options](Running-and-options)).

The messages themselves are described on [Network messages](Network-messages), and the code
on [Network internals](Network-internals).
