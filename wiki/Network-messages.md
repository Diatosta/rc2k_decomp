# Network messages

*Category: [Data formats](Data-formats)*

A network game of Rally Championship 2000 was built on Microsoft's DirectPlay. This project
replaces DirectPlay with a small protocol of its own, so a game here is played between copies
of this program and not with the 1999 one. The game's own messages, the bytes the cars send
each other, are the original's and travel inside it unchanged.

This page describes both layers: the **session** layer (finding a game, joining, players and
groups, delivering messages) and the **game** layer (what the lobby and the race say).

Numbers are little-endian. "Step" is one step of the simulation, a hundredth of a second (see
the [Glossary](Glossary)).

## Overview

- One machine **hosts** a session. Every other machine holds one TCP connection to the host.
  Machines never talk to each other directly: the host passes messages on.
- A session is found with a UDP probe sent to the same port number.
- The default port is `0xBA08` (47624), DirectPlay's old port. The options can name another.
- The host owns the session's state: its description, the list of players, the list of
  groups. Other machines ask the host for changes, and the host tells every machine each
  change it makes.
- If the host's connection ends, the session is lost for everyone.

## Building blocks

These appear in several messages.

| Name | Layout |
| --- | --- |
| bytes | A u32 length, then that many bytes. |
| text | A "bytes" holding UTF-8 text. |
| GUID | 16 bytes. |
| description | The session's description: u32 flags; GUID of this session; GUID of the application (the game's fixed one); u32 most players; u32 players now; text name; four u32 of the game's own (a fixed mark: `0x17`, 1, 0, 0; bit 0 of the second says the session is this game's, and a session without it is not listed). |
| player | text name; bytes data; u32 machine number. |
| group | text name; bytes data; u32 machine number; u32 count; that many u32 player ids. |

Flags of the description that the session layer looks at: bit 0 (`0x01`) "no new players",
bit 5 (`0x20`) "joining off". The host sets both to close the session to newcomers and clears
them to open it again.

Machines are numbered by the host: the host is machine 1, each machine that joins gets the
next number. Players and groups have ids the host hands out.

Limits, there so that another machine cannot make this one hold unbounded data:

| What | Most |
| --- | --- |
| A frame | 1 MB |
| A game message | `0x100` (256) bytes |
| Players, groups, connections | `0x40` (64) each |
| A name | `0x100` bytes |
| A player's or group's data | `0x80` bytes |

The game itself allows eight players in a session.

## Finding a session (UDP)

The searching machine sends one datagram of 24 bytes to the port, either to the address the
player typed or, with no address, to the network's broadcast address and to its own machine:

| Offset | Size | Content |
| --- | --- | --- |
| 0 | 8 | The text `RC2KNET4`. |
| 8 | 16 | The application's GUID. |

A host with a session of that application answers the sender with:

| Size | Content |
| --- | --- |
| 8 | `RC2KNET4`. |
| 4 | u32: the TCP port to connect to. |
| varies | The session's description. |

The searcher keeps each answer for 4 seconds and probes again each time the game asks for
the list, so the sessions screen fills in over a moment and a session that has gone drops
out.

## The connection (TCP)

Everything on the TCP connection is a **frame**: a u32 length, then that many bytes. The
first byte of a frame says what it is.

| First byte | Name | Direction | Rest of the frame |
| --- | --- | --- | --- |
| `01` | join | machine → host | `RC2KNET4`. Must be the first frame. The last character of the text is the protocol's version (`4`); a join that starts `RC2KNET` with another version is answered with a refuse (reason `0x887700E0`, not a DirectPlay number) and a host that ends the connection on the join is taken for an older version. |
| `02` | welcome | host → machine | u32 your machine number; description; u32 count and that many (u32 id, player); u32 count and that many (u32 id, group). |
| `03` | refuse | host → machine | u32 reason (a DirectPlay error number). Sent in place of the welcome when joining is off. |
| `04` | request | machine → host | u32 request number; one operation (below). |
| `05` | reply | host → machine | u32 request number; u32 result (0 = done, else a DirectPlay error number); u32 the new id, for a request that made a player or group. |
| `06` | change | host → every machine | One change (below). |
| `07` | send | machine → host | u32 from (the sender's player id); u32 to; bytes: a game message. |
| `08` | deliver | host → machine | u32 from; u32 to (the player it is for); bytes: the game message. |
| `09` | bye | host → every machine | Nothing. The host is closing the session. |
| `0B` | challenge | host → machine | Only in a locked session (its description has the flag `0x400`), in answer to the join: 16 bytes of the host's salt, 16 bytes of a fresh challenge. |
| `0C` | proof | machine → host | 32 bytes: the answer to the challenge (below). |
| `0A` | kicked | host → machine | Nothing. The host has put this machine out; the host ends its connection after it, and the machine's players go as when a machine leaves. |

A **locked session** is joined in two more steps. After the join the host sends a challenge; the
joiner works out `key` from the salt and its password (SHA-256 of the salt and the password,
then 4095 more rounds of SHA-256 of the result and the salt) and answers with `proof` =
SHA-256(challenge, key). The host keeps only the salt and the key, never the password, checks
the proof in a time that does not depend on where it differs, and a new challenge each join
makes a recorded proof useless. A wrong or missing password is answered, after a second, with
a refuse (reason `0x887700E1`, not a DirectPlay number). The password is never sent.

"To" in a send frame is a player's id, a group's id (every member gets it), or 0 for everyone.
The sender never gets its own message back. The host checks that "from" is a player of the
machine that sent the frame, and drops messages longer than `0x100` bytes.

### Operations a machine asks for

| Code | Operation | Fields |
| --- | --- | --- |
| `00` | Make a player | text name; bytes data. |
| `01` | Make a group | text name; bytes data. |
| `02` | Add a player to a group | u32 group; u32 player. |
| `03` | Take a player out of a group | u32 group; u32 player. |
| `04` | Remove a player | u32 player. |
| `05` | Set a player's data | u32 player; bytes data. |
| `06` | Set the session's description | description. Only the host's own machine may. |

A machine may only remove or change its own players.

### Changes the host announces

| Code | Change | Fields |
| --- | --- | --- |
| `00` | Player added | u32 id; player. |
| `01` | Player removed | u32 id. |
| `02` | Group added | u32 id; group. |
| `04` | Player joined a group | u32 group; u32 player. |
| `05` | Player left a group | u32 group; u32 player. |
| `06` | Player's data set | u32 player; bytes data. |
| `07` | Description changed | description. |

(There is no code `03`.) When a machine's connection ends, the host removes that machine's
players and announces it.

Each machine applies every change to its own copy of the session. For a change that another
machine caused, the game is also handed a "system message" laid out as DirectPlay's were
(player made or gone, added to or removed from a group, data set), because the game's lobby
code still reads those.

## The game's messages

These are the bytes carried in send and deliver frames, and the data attached to players.

### A player's data: 16 bytes

Each machine keeps its player's state in the session as the player's data, and sets it again
whenever it changes. Everyone therefore sees everyone's choice of car, readiness and results
without separate messages.

| Offset | Size | Meaning |
| --- | --- | --- |
| 0 | 1 | State bits: `0x01` ready, `0x02` in the race and waiting for the others, `0x04` race done, `0x10` this player is the host, `0x80` not counted when the game asks "is everyone ...". |
| 1 | 1 | Car. |
| 2 | 1 | The car's colours. |
| 3 | 1 | Nationality. |
| 4 | 1 | Stage. |
| 5 | 1 | The stage's direction. |
| 6 | 1 | Race number. |
| 7 | 1 | A random byte. |
| 8 | 4 | u32: this player's time in the race. |
| `0x0C` | 4 | u32: this player's running total. |

### Messages by first byte

| First byte | Message | Rest |
| --- | --- | --- |
| `0x22` (`"`) | A chat line, joiner to host only. The host takes it, adds it to its chat and orders it (see `0x23`); other machines drop it. | The text, with no ending zero. |
| `0x23` | A chat line in the host's order, host to every other machine (the line's author too). The receiver finds the author by id and shows the line when it comes; so every machine shows the lines in the order the host took them, and a joiner's own line shows when it comes back. | The host's number for the line (4 bytes), the author's player id (4 bytes), the text with no ending zero. |
| `0x40` | The host asks: is everyone ready to race? | One byte: the round number. |
| `0x60` | A machine's stage load's progress. | One byte: the percentage (0 to 100). |
| `0x41` | Answer to `0x40`. | One byte: the same round number. |
| `0x50` | The host asks: is everyone done with the race? | One byte: the round number. |
| `0x51` | Answer to `0x50`. | One byte: the round number. |
| `0x80` .. `0xFF` | A race message. | Below. |

The round number lets late answers from an earlier race be told from this one's: calls and
answers with another round's number are ignored. Only machines that are not the host answer,
and only the host counts the answers.

### Race messages

During a race each machine sends what its own car is doing. A race message is:

| Offset | Size | Content |
| --- | --- | --- |
| 0 | 1 | A sequence byte: one more than this machine's last, always with the top bit set. After `0xFF` comes `0x80`. |
| 1 | 4 | A tag: four characters. |
| 5 | 4 × n | n values of 32 bits. |

The receiver keeps up to `0x40` (64) messages from each player in sequence order and hands
them to the simulation at its next step.

There are three kinds:

| Tag (as bytes) | Values | Sent when the car's step count ... | That is |
| --- | --- | --- | --- |
| `JGSM` | 4 | is a multiple of 4 | 25 times a second |
| `PGSM` | 40 | ends in the bits `00010` (step mod 32 = 2) | about 3 times a second |
| `XGSM` | 9 | step mod 256 = 1 | every 2.56 seconds |

**`JGSM`: the inputs.** Four values: throttle, keys and buttons, steering, brake, as the
car's input code holds them. Between position messages the receiving machine drives the
remote car with these through its own physics.

**`PGSM`: where the car is.** Forty values:

| Values | Content | Type |
| --- | --- | --- |
| 0 .. 2 | Position x, y, z, in 1/65536ths of a metre. | i32 |
| 3 .. 14 | Orientation: the first twelve numbers of the car's matrix. | f32 |
| 15 .. 17 | Velocity. | f32 |
| 18 .. 20 | Velocity in the car's own axes. | f32 |
| 21 .. 23 | Spin. | f32 |
| 24, 25 | The checkpoint the car is at, and the one a reset would return it to. | u32 |
| 26 | Gear: −1 reverse, 0 neutral, 1 .. 7. | i32 |
| 27 | Engine speed. | f32 |
| 28, 29 | Speed of the driven wheels. | f32 |
| 30 .. 33 | How far each wheel's spring is pressed. | f32 |
| 34 .. 38 | Throttle, brake, handbrake, steering, and the steering before smoothing. | i32 |
| 39 | A word of the car's own state. | u32 |

**`XGSM`: the rest.** Nine values: eight impact figures, one for each of the car's eight
corner regions (f32, 0 to about 1), and whether the lights are on.

### What a receiver does with them

A race message is only used if every value in it is something a car could hold: the gear in
range, the floats finite and not huge, the pedals within bounds. A message that fails is
dropped whole. Unknown tags and messages that are too short are dropped too.

When a position arrives the receiver compares it with where its own copy of that car is:

- less than 10 metres away, or in the car's first 1000 steps: the car is moved there
  smoothly, a 32nd of the way each step for 32 steps;
- further: the car is put there at once.

The rest of a `PGSM` (orientation, speeds, gear and so on) is taken over directly.

## A made-up example

A machine joins a session called "Friday" that has one player, the host's.

Machine → host, the join frame (length 9):

```
09 00 00 00  01  52 43 32 4B 4E 45 54 31
```

Host → machine, the welcome (shown in parts):

```
02                          welcome
02 00 00 00                 you are machine 2
00 00 00 00                 flags
<16 bytes> <16 bytes>       session GUID, application GUID
08 00 00 00  01 00 00 00    at most 8 players, 1 now
06 00 00 00  46 72 69 64 61 79          name "Friday"
17 00 00 00  01 00 00 00  00 00 00 00  00 00 00 00    the game's four values
01 00 00 00                 one player:
  01 00 00 00                 id 1
  04 00 00 00  41 6E 6E 61    name "Anna"
  10 00 00 00  <16 bytes>     its data: the 16-byte record
  01 00 00 00                 on machine 1
00 00 00 00                 no groups
```

Later, in a race, the joined machine's player (id 2) sends its inputs to everyone:

```
22 00 00 00                 frame length 34
07                          send
02 00 00 00                 from player 2
00 00 00 00                 to everyone
15 00 00 00                 21 bytes of message:
  93                          sequence
  4A 47 53 4D                 "JGSM"
  00 01 00 00  00 00 00 00  E0 FF FF FF  00 00 00 00      four input values
```

## Things to know

- Nothing is encrypted or signed, and a session's password only decides who may join: what is
  sent afterwards is not hidden. A session without a password lets anyone who can reach the
  host's port join. The protocol is meant for a home network or a trusted link.
- The host does not pass to another machine when it leaves.
- The host's port must be reachable over both TCP and UDP for others to find and join it.

## Code

`src/net/dp_session/` (the session layer: `protocol.rs` frames and changes, `host.rs`,
`peer.rs`, `discovery.rs`, `tests.rs`), `src/net/dp_objects.rs` (the DirectPlay-shaped
interface the game's code calls), `rc2k_core/src/net/message.rs` and `messages.rs` (the
game's messages by first byte), `rc2k_core/src/net/race.rs` (race message kinds, timing,
arrival), `src/net/send_inputs.rs` (which of the car's values each kind carries),
`rc2k_core/src/net/record.rs` (the player's 16 bytes), `rc2k_core/src/net/inbox.rs`,
`src/net/frozen_formats.rs` (tests that pin these bytes).
