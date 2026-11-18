# Network play

*Category: [Getting started](Getting-started)*

Several machines can race each other over a local network or the internet. The game's
network screens are the 1999 game's; what carries the messages underneath is new.

## What differs from the 1999 game

- The original used DirectPlay. This program has **its own protocol** in its place, so it
  plays against other copies of this program only, not against the original `RAL.EXE`.
- Two kinds of connection are offered: TCP/IP and, when Valve's library is beside the program,
  Steam ([Playing through Steam](#playing-through-steam)). The modem, serial and IPX connections
  of 1999 are not there.
- One machine hosts and the others join it directly (over Steam, through a Steam lobby).
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

## Playing through Steam

Every desktop build (Windows, Linux, macOS) can use Steam, loading Valve's library, `libsteam_api.so`,
`steam_api64.dll` / `steam_api.dll` or `libsteam_api.dylib`, while it runs: from the program's folder, or from
the system's library path (a release made with `RC2K_STEAM=1 ./build.sh <target>` carries it). With the library
there, the program lists **Steam** as a second connection, after TCP/IP, when the Steam client is running and
you are signed in. It is not there on the handheld, and without the library or without Steam running
nothing changes: only TCP/IP is listed, and the program's log says why Steam was left out
(`net: steam: not available: ...`). No port needs opening: Steam's relay carries the
connection, between Steam accounts.

What a player needs:
1. The Steam client installed, running and signed in (any account: no purchase is needed).
2. Valve's Steam library in the game program's folder, beside `rc2k` / `rc2k.exe`: `libsteam_api.so` on
   Linux, `steam_api64.dll` on 64-bit Windows (`steam_api.dll` on 32-bit), `libsteam_api.dylib` on macOS. A
   release that offers Steam carries it; otherwise take it from the Steamworks SDK (free with a Steam account
   at partner.steamgames.com), folder `redistributable_bin`, and copy it there.
3. Start the game, open the network page and choose **Steam** as the connection. If it is not listed, the
   game's log (`rc2k.log`) says why (`net: steam: not available: ...`).

- Until the game has an application number of its own it runs as Valve's test application,
  **Spacewar** (480), which every Steam account has: friends see you playing "Spacewar".
- Choose the Steam connection, then host or join as with TCP/IP. **Hosting** makes a public
  Steam lobby that carries the session's name, its limit and whether it has a password.
  **Joining** lists the lobbies of the same program version that Steam finds (strangers'
  included, as Spacewar's list is shared with other test games), friends' lobbies and a lobby you
  were invited to; choosing one joins the lobby and connects to its host. The password question and
  the lobby are as over TCP/IP.
- **Invites.** "Join game" in Steam's friends list, or accepting an invitation, takes you straight
  into that lobby from any menu: the game goes to the network flow with the Steam connection and
  joins the lobby as picking it from the list would (your name as the network page has it; the
  password question when the lobby has one, with Escape going back to the list of sessions). In a
  race, a replay, a film or the installer the invitation waits and is followed when you are next
  in the menus. An invitation accepted while you are in a network game is ignored: leaving a
  lobby others are in is not something to do on a click in another window. A game started by the
  invitation takes the lobby from the launch argument `+connect_lobby <id>`, followed once the
  menus are reached. While in a lobby the status line shown to friends says you are in a network race.
- The Steam list and the TCP/IP list are separate: a Steam host is not found by a TCP/IP join.
- Two Steam accounts are needed to try a game: one machine cannot join its own lobby.

## Setting up a game

In the main menu choose the network game, then host or join.

**The network settings screen** comes first:

| Item | What it is |
| --- | --- |
| Connection | The kind of connection: TCP/IP, or Steam when the library is there and the Steam client is running. |
| Player's name | Your name in the lobby and the results. |
| Most players | The host's limit on how many may join. |
| Host's address | For joining: the host's address. Leave it empty to look for games on the local network. |
| Port | The port, as described above. |
| Host password | For hosting only: the session needs this password to join (leave it empty for none). Shown as stars; choosing the row clears it for typing a new one (Escape keeps the old one). Kept in `var/rc2k.toml` (`[network] password`, readable text) so that it is there at the next start. |

**Hosting.** The host makes the session and waits in the lobby. Its machine must stay in the
game: when the host leaves, the session ends for everybody (it is not handed to another
machine).

**Joining.** The program lists the sessions it finds. With no address given it asks the whole
local network (and this machine itself); with an address it asks that machine only. Choose a
session to join it. Full or closed sessions are not listed. A session with a password shows a lock mark before its name; choosing it asks for the password there and then (typed as stars, on the on-screen keyboard from a pad; the question and the stars stand high on the screen so the keyboard, below them, covers neither). A wrong one asks again; Escape goes back to the list of sessions. A session without a password is joined as before. Nothing about the right password is revealed to a joiner, and the joiner's answer is not kept.

**The lobby.** Everyone chooses a car; the host chooses the stage, and its name is shown at
the top left of everybody's lobby. While the stage loads, the loading screen lists every player's progress in percent in its top left corner. There is a chat line under the last six chat lines (older ones are kept but not shown, so that no line runs under the buttons; a long message is cut into lines as wide as the screen); the chat starts closed each time the lobby is entered, also on coming back from a race (a key still held from the race's last screen does not open it); the host puts the lines in order, so everyone sees the same order (a line you type shows when the host sends it back). Each joined player's **ping** (the round trip to the host in milliseconds, a moving average of the host's once-a-second measurement) is shown after the name; the host's own row has none. Pressing **ready** again takes the ready back (the button reads "not ready" while you are ready; left and right move between it and the chat). When every player is ready the host
starts the race; a player who takes the ready back before the start stops it, and a machine that is no longer ready does not answer the host's start call.

Above the bottom row of buttons is a second row. Its rightmost button is **bots** (the host's switch for computer drivers, below), then **settings** (always shown): it opens the options of the menu
after a stage (audio, display, advanced, and the players' details; the cars' specifications are left out, as the lobby has its own car button; what the race
fixes is not changed there) and returns to the lobby. Up and down change between the two rows,
left and right move along one.

**Bots (computer drivers).** The second row's last button reads "bots: off", "bots: 1" .. "bots: 7". The host presses it to change the number (Enter; the others see the number and cannot change it, and it cannot be changed while the host is ready). The race then has that many computer-driven cars after the players' cars, as many as there are free places among the race's eight cars (with five players, "bots: 7" gives three). They drive as the arcade's opponents do and are listed under the players in the lobby, with their times and totals as each machine saw them (the host's race is the one every machine follows, so a bot's time can differ a little between machines). The race ends for you as before, when your own car has finished. With bots off the race is as it was. All machines must run the same version of the program (the protocol's mark changed with this).

**Putting a player out.** The host types `/kick <name>` in the chat line (the first other player
of that name, case ignored). The line is not sent to the others; the host sees an answer line
in its chat. The player is told and leaves the lobby as if the host had gone, and may join again.

**The race.** Each machine simulates its own car and sends its inputs and position; the others
show it. The stage bar along the top shows every car as a marker, each in its own colour, the
own car's in white. Afterwards the results come up and the lobby returns for the next stage. A player who finishes goes back to the lobby while the others are still racing; there the table shows, in place of the stage time of each player still racing, how far through the stage that player is, as a percentage that rises while the lobby is shown (nothing is shown until the first figure arrives). A finished player's own time replaces it.

**The replay.** A network race is recorded as a single race is, every car included: the other
players' cars from the inputs their messages carry, and their whole state at the start of each
second, as this machine saw it. Each machine records its own view, so the replays of one race
differ a little between players (a remote car's position corrections are not recorded, only
its inputs and its state each second). Once the race is over the lobby shows a **save replay**
button in a second row of buttons above the bottom row (reached with the up key, then the left
key); it asks for a slot as the single-player results screen does, and the replay can then be
watched from the replays menu. Backing out of the slot screen leaves the lobby as it was. A
replay that cannot be saved (its recording cannot be read, or the slot's file cannot be written)
gives a message and the lobby stays; it never ends the program.
The file is a replay of the usual format, but it holds only the first two cars' inputs and the
first four cars' states: a race of more than two players plays back with the other cars
following their stored states only.

## From the command line

`rc2k --host` and `rc2k --join <address>` go straight to the lobby, with `--name` and `--port`
for this run: see [Running and options](Running-and-options).

## Two copies on one machine

For trying it out, two copies can run on the same computer: host in one, and in the other join
with the address left empty (this machine is always asked) or set to `127.0.0.1`. Give each
copy its own game folder, or they will share `var` and `savedata` (see "Two copies in one
folder" on [Running and options](Running-and-options)). Each copy keeps the last race's replay
in a file of its own.

## If it does not work

- No session listed: check the firewall and the port on the host; over the internet, give the
  host's address rather than leaving it empty (a broadcast does not leave the local network).
  With the address left empty the search also asks every peer of your Tailscale network (when the
  `tailscale` command works on this machine; it is asked once in 30 seconds, and when it is
  absent nothing changes) and the last host you joined (kept in `var/rc2k.toml`, `last_host`),
  so a host over Tailscale is found with no address typed.
- "The port N is in use by another program." when hosting: another program (often a second copy
  of this game) already holds that port, TCP or UDP. Close it, or set another port on the network
  settings page.
- "Could not start the session" when hosting for another reason: `rc2k.log` says "the session could not be hosted on port N" with the
  system's reason; every other way the network start ends in a failure message is logged too.
- The session disappears: the host left or lost its connection.
- Anything else: both machines' `rc2k.log` say what the network layer saw
  ([Running and options](Running-and-options)).

The messages themselves are described on [Network messages](Network-messages), and the code
on [Network internals](Network-internals).
