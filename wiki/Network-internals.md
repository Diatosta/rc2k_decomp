# Network internals

*Category: [Game flow](Game-flow)*

This page explains how a network game works inside: how machines find and join each other,
what the lobby does, what a car sends while racing, and how the other players' cars are moved
on your screen. For setting a game up as a player, see [Network play](Network-play). For the
bytes on the wire, see [Network messages](Network-messages).

The code is `src/net/`, with rules in `rc2k_core/src/net/`. The state is the `Net` owner
(`src/net/owner.rs`).

## Two layers

The 1999 game used DirectPlay, a Windows networking library, and its game code is written
against DirectPlay's ideas: a **session** that one machine hosts, **players** in it, a
**group**, messages sent to a player or to everyone, and **system messages** that tell each
machine when a player joins or leaves.

This project keeps the game code and replaces what is underneath:

| Layer | Where | What it is |
| --- | --- | --- |
| The game's network code | `src/net/*.rs` | The original's logic: lobby, settings, race messages. It asks for DirectPlay-style operations. |
| DirectPlay's operations | `src/net/dp_objects.rs`, `src/net/dplay.rs` | The same operations (list sessions, host, join, create a player, send, receive) as plain functions. |
| Sessions | `src/net/dp_session/` | This project's own implementation of those operations over ordinary sockets. It knows nothing about the game and is tested on its own over the loopback address. |

Because the bottom layer is new, a network game is only possible between copies of this
program, not with the 1999 game.

## Sessions

`src/net/dp_session/mod.rs` describes the design:

- **The host keeps the session.** It holds the session's description (name, most players,
  the game's mark), the list of players and the group. Every change is made by the host: another
  machine asks, the host does it and tells every machine. So all machines see changes in the
  same order.
- **Ping**: the host sends each machine a `PING` (its clock in ms) every second, the machine
  answers `PONG` with the same stamp, and the host folds the round trip into a moving average
  (three parts old, one new) and sends every machine the figures (`PINGS`); the lobby shows
  them after the names (`Session::ping_of`). A frame of a kind a machine does not know is
  passed over, not an error; the protocol's mark is `RC2KNET5` since these frames came (and `RC2KNET6` since the bots' messages, below).
- **Finding a session** is a small UDP question sent to a given address, or broadcast on the
  local network. A host answers with its description (`discovery.rs`). With no address the
  probe also goes to the peers `tailscale status --json` lists (`rc2k_core::net::tailscale`,
  asked on a thread of its own at most every 30 s) and to the last host joined.
- **Joining** is a TCP connection to the host (`peer.rs` on the joiner, `host.rs` on the
  host). The default port is DirectPlay's old one, 47624.
- **The connection is a seam** (`wire.rs`): the session talks to each machine through a `Wire`
  (read, write, clone, shut down, time limits). TCP is one; Steam's relay (`net::steam`, Valve's library loaded at run
  time through its flat C interface) is the other: a host listens through a Steam lobby and a joiner connects to
  the lobby owner's Steam ID, each write being one reliable Steam message. The protocol above is
  the same over both. The tests run a whole session over an in-memory `Wire`.
- **Messages** from one machine to others go to the host, which passes them on to the players
  they are addressed to. They are never sent back to the sender.
- **If the host goes**, the session is lost. It does not move to another machine.

`protocol.rs` defines the frames: join, welcome or refuse, a request and its reply, a change,
a message to send and one delivered, and goodbye.

## Before the race

### Start-up

`net::set_up` runs once at start (`src/net/set_up.rs`). It makes the event that the receive
thread waits on. No connection is made until the player opens the network menu.

### The menus

| Step | Code | What happens |
| --- | --- | --- |
| Network menu | `src/menu/network_rally_loop.rs` | The entry point from the main menu. |
| Settings | `src/net/settings_loop.rs` | The network settings; from here the connection is chosen. |
| Connection | `src/net/choose_connection.rs` | The list of connections to choose from. |
| Sessions | `src/net/find_sessions.rs`, `choose_session.rs` | Lists the sessions that answered; the player picks one, or hosts (`host_session.rs`, `join_session.rs`). |
| Lobby | `src/net/lobby_loop.rs` | Everyone gathers here. |

Operations that can take a while (opening a session, listing sessions) can be **posted** to
the receive thread (`net::post_call`), so the menu keeps drawing while they run.

### The lobby

In the lobby each machine has a **player record**: name, chosen car and colours, ready or not,
state (in the lobby, waiting, racing, raced), and the last race's time. A machine publishes its
record as its player data, and once a second each machine lists the players and reads their
records (`src/net/list_players.rs`). That is how names, cars and "ready" marks appear on every
screen.

Pressing ready again clears the ready bit (`State::made_not_ready`) and publishes the record at
once; the host's sync round starts afresh (a new counter) when its all-ready test fails, and a
machine that is not ready does not answer a ready call.

The lobby's items are: ready, car (`choose_car.rs`), stage (the host chooses, `choose_stage.rs`)
and chat (`chat_key.rs`, `send_chat.rs`). Chat lines are ordinary messages with a marker byte; a joiner's goes to the host, which numbers it and sends it to everyone (`relay_chat.rs`).

When two or more players are all ready, the host starts a **sync round**: it sends each player
a start message with a counter and waits until every one has answered. Then every machine
leaves the lobby loop and starts the race.

### Computer drivers (bots)

The host's lobby button sets how many computer drivers the race has (off, 1 to 7). The number
rides in bits 1 to 3 of the host's record's direction byte, so every machine reads it with the
stage; each machine works out the same number of bots, the players' count and the setting held
to the eight cars (`rc2k_core::net::bots`). The bots take the cars after the players': with
three players the bots are cars 3, 4 and so on. Their kinds are drawn the same way on every
machine, from the cars within five places of the host's car (as the arcade chooses its
opponents), and their pace differs a little from bot to bot.

Every machine drives a bot's car itself with the game's own computer driver
(`drive::read_car_controls`; the cars have the driver kind the arcade's opponents have). The host
then sends each bot's position and extra values as the `0x62` message
(`src/net/send_bot_turn.rs`); the others file it in the bot's inbox
(`src/net/take_messages.rs`) and apply it as they do a player's, so the bot's copy glides to the
host's. The host's copy is the one every machine follows. The race ends for a machine when its own
car has finished, as before. The lobby does not list the bots: a label shows their number to every player.

## Starting the race

`net::lets_rally` (`src/net/lets_rally.rs`) runs one network race:

1. `net::set_up_race` builds the race from the players' records: each player's car, colours
   and number, the host's stage. It loads the stage as any race does
   (see [A race, step by step](A-race-step-by-step)).
2. Each machine then says it is waiting (`net::race_wait`). When all are, the host runs a
   second sync round and the race begins on all machines together.
3. `net::lets_race` runs the race loop.
4. Afterwards the machine's time is recorded and published (`net::race_over`), and everyone
   returns to the lobby. The host opens the session to newcomers again.

Every machine runs the **whole** race itself: all the cars, with full physics. There is no
central simulation. Each machine is the authority on its own player's car and follows what the
others say about theirs.

## During the race

### The receive thread

`net::receive_thread` (`src/net/receive_thread.rs`) waits for a signal that something has
arrived, then takes messages until there are no more (`src/net/take_messages.rs`). By kind:

| Message | What the thread does |
| --- | --- |
| A system message (a player joined or left, data changed) | Updates the player lists (`system_message.rs`). |
| A chat line | Adds it to the chat (`add_chat_line.rs`). |
| A sync message | Answers it, or notes the answer if this machine is the host. |
| A race message | Files it in the sender's **inbox** with the time it arrived (`store_received.rs`). |
| Anything else, or from an unknown sender | Dropped. |

An inbox holds a player's recent race messages in order of their **sequence number**, a
counter the sender puts on every race message. A message older than what has already been
used is dropped.

The thread only files messages. It never touches a car. A message too long for the game to
take is dropped by the session layer, and a race message holding values no car could have is
ignored when it comes to be applied.

### What a car sends

In each simulation step, after a car's physics, `net::race_step` runs for it
(`src/net/race_step.rs`). For this machine's own car, `net::send_turn` sends, by the car's
step count:

| Message | When | Holds | Code |
| --- | --- | --- | --- |
| Inputs | Every 4th step (25 a second) | The car's four input values: steering, throttle, brake, buttons. | `send_inputs.rs` |
| Position | Once every 32 steps (about 3 a second) | The car's place, its orientation, and the main motion values: speeds, gear, engine. | `send_position.rs` |
| Extra | Once every 256 steps | A few slow-changing values of the car. | `send_extra.rs` |

`net::send_message` (`src/net/send_message.rs`) adds the sequence number and sends the message
to all players.

Then it applies what the others sent: `net::copy_received` copies the inboxes out under their
lock, and `net::apply_received` puts each message into its sender's car
(`src/net/apply_message.rs`).

### How the other cars move

A remote car is not simply placed where its owner says. That would make it jump three times a
second. Instead:

- **Its inputs are applied to its own physics.** Between position messages, the remote car is
  driven on this machine by the last inputs received, through the same physics as every other
  car. With the same inputs and the same physics it goes nearly where the real one goes.
- **Position messages correct the drift.** When a position arrives, the difference between
  where the car is here and where its owner says it is gets spread over the next 32 steps: each
  step, `net::follow_remote` moves the car a 32nd of the way (`src/net/follow_remote.rs`). The
  car glides to the right place instead of jumping.
- **A large error is a jump.** If the car is ten metres or more from where it should be (after
  a crash that went differently on the two machines, say), it is put there at once.

In a network game a car's input values in `drive::car_input_words` come from its block, where
`apply_message` put them, instead of from a device.

The original's version of this had a wrong distance limit that made remote cars jerk and jump
back; that and several other network faults are described in [Fixed bugs](Fixed-bugs).

### Which car is "mine"

In a network race every machine has the same list of cars in the same order, so this
machine's car is usually not the first. `net::set_up_race` notes which one is local, and the
view, the cockpit, the sound listener and the HUD follow that car
(`drive::choose_shown_cars`).

## Leaving

- Escape in a race asks, as usual; a yes leaves the race and the machine's result is "did not
  finish".
- Leaving the lobby closes the session for this machine (`src/net/close_session.rs`); if it
  was the host, the session ends for everyone.
- At the program's end `net::shut_down` closes whatever is open (`src/net/shut_down.rs`).

## Limits

| Limit | Value |
| --- | --- |
| Players in a race | 8 |
| A race message | 255 bytes |
| Messages kept per player's inbox | 64 |
| Session survives the host leaving | No |
| Plays against the 1999 program | No |

## Testing

- `src/net/dp_session/tests.rs` tests sessions alone: hosting, joining, requests, routing and
  losing the host, all over loopback.
- `src/net/frozen_formats.rs` builds each game message from fixed inputs and compares the
  exact bytes, so the message formats cannot change by accident.
- The project's check rounds run a real two-copy network race on one machine and compare the
  cars' paths; see [Testing](Testing).
- `tools/fuzz.sh` fuzzes the readers of incoming messages.

## Where to look

| To change or understand | Start in |
| --- | --- |
| The overall picture | the comment at the top of `src/net/mod.rs` |
| Sockets, hosting, joining | `src/net/dp_session/` |
| The operations the game asks for | `src/net/dp_objects.rs` |
| The lobby | `src/net/lobby_loop.rs`, `lobby.rs` |
| Starting a race together | `src/net/lets_rally.rs`, `race_wait.rs`, `set_up_race.rs` |
| What is sent | `src/net/send_turn.rs` and the three `send_*.rs` files |
| What is done with a received message | `src/net/take_messages.rs`, `apply_message.rs` |
| Smoothing remote cars | `src/net/follow_remote.rs` |
| Player records | `src/net/types.rs`, `list_players.rs` |

Back to [Game flow](Game-flow). See also [Module-net](Module-net),
[Network messages](Network-messages), [Network play](Network-play),
[Threads and shared state](Threads-and-shared-state).
