# Module: net

*Category: [Module reference](Modules)*

Network play. It has two halves. The game's half is as in the original: the lobby screens
(choosing a connection, hosting or joining a session, chat, settings), the messages a race
sends each step, and how a remote player's car is moved. Underneath, where the original used
DirectPlay, is this project's own session layer over the network (`dp_session/`), so a network
game is played between copies of this program only.

## State

The owner is `Net` (`src/net/owner.rs`): the link's state, the players, each player's inbox of
received messages, the lobby screens' variables. Parts are in `src/net/store.rs`,
`src/net/tables.rs` and `src/net/types.rs`.

## Main entry points

| Item | File | What it does |
| --- | --- | --- |
| `set_up` | `src/net/set_up.rs` | Network set-up at the program's start. |
| `lobby_loop` | `src/net/lobby_loop.rs` | The lobby screen. |
| `password` | `src/net/password.rs` | The host's password (typed on the settings screen, kept in `rc2k.toml`) and the joiner's answer (this join only), with `password_prompt`, the question a joiner is asked; `rc2k_core/src/net/password.rs` makes the salted key and the proofs. |
| `progress` | `src/net/progress.rs` | Each racing player's progress through the stage in percent: sent from the race's step, shown in the lobby's table. |
| `loading` | `src/net/loading.rs` | Each player's stage-load percentage: sent from the loading bar, listed on the loading screen. |
| `host_session`, `join_session` | files of the same names | Starting or joining a game. |
| `receive_thread` | `src/net/receive_thread.rs` | The thread that waits for messages and files them. |
| `send_message` | `src/net/send_message.rs` | Sends one message to the other players. |
| `race_step` | `src/net/race_step.rs` | Each simulation step of the own car: sends inputs, place and motion at their intervals. |
| `copy_received`, `apply_received` | files of the same names | Takes the received messages under the lock and applies them to the remote cars. |
| `follow_remote` | `src/net/follow_remote.rs` | Glides a remote car towards the place last received. |
| `shut_down` | `src/net/shut_down.rs` | Ends the network at the game's end. |
| `Session` | `src/net/dp_session/mod.rs` | A session: its host, players and groups. |
| `find_sessions` | `src/net/dp_session/discovery.rs` | Finds games on the local network. |
| `protocol.rs` | `src/net/dp_session/protocol.rs` | How this project's messages are written and read. |

## How it is laid out

Flat, one function per file, with `dp_session/` (`host.rs`, `peer.rs`, `discovery.rs`,
`protocol.rs`) for the session layer. Groups: `*_frame.rs`, `*_loop.rs` and `draw_*` (the
lobby's screens), `send_*` (messages), `*_found.rs` (what a search reports),
`dplay*.rs` and `dp_objects.rs` (the seam where the session layer stands in for DirectPlay).

## Calls and callers

It draws with [`gfx`](Module-gfx) and [`text`](Module-text) and uses [`race`](Module-race)'s
car state. [`drive`](Module-drive) calls it each step of a network race; start-up and
`threading` start it.

## Related pages

[Network internals](Network-internals), [Network play](Network-play),
[Network messages](Network-messages).
