# Fixed bugs

Bugs of the original `RAL.EXE` that the Rust version fixes. Up to the parity milestone the
Rust code kept every one of the original's slips, so that it could be compared with the
original function by function; since then each bug is fixed as its code is converted.
Addresses are those of the original executable (as IDA shows them).

Each entry says:
- **What**: the mistake in the original's code.
- **In game**: what a player could see or hear (or that nothing shows).
- **Seen**: whether it was seen in play or found by reading the code.
- **Fix**: what the Rust version does instead, and where.

---

## Sounds stuck on after a stage ends (lost sound commands)

*Original*: `sound::send_command` (0x409C45) and `audio::take_commands` (0x40A8A4).
*Fixed*: 2026-10-21.

- **What**: the game thread queues sound commands for the audio thread and counts them up;
  the audio thread takes them and counts down. Both change the one counter without an
  atomic instruction, so on a processor with more than one core an update can be lost.
  A lost count-up leaves a command uncounted: every later command is taken one late, and
  the last one before a quiet moment not at all. A lost count-down replays an old command.
- **In game**: at a stage's end the "stop every sound" command can go missing, so the
  car's looping engine sounds (or a stage's ambient loop) keep playing over the results
  and the menus, even while paused, until the next stop; now and then an old sound is
  played again. Machines of the game's day had one core and never showed it.
- **Seen**: yes, in play (fifteen stop commands sent, nine taken in one session).
- **Fix**: both sides change the counter atomically (`sound::queue`,
  `audio::take_commands`).

## The music can hang the sound

*Original*: `audio::music_command` (0x40C4DD). *Fixed*: 2026-10-20.

- **What**: two ways for the audio thread to loop for ever. When a tune starts but stops
  at its first block of data while a next tune is set, that next tune is played again and
  again (it is never cleared once used). And with the saved tune position past its last
  value (a damaged `ral.dat`) and no tune files 10 .. 15, the search for the next tune
  never ends.
- **In game**: the music stops and the sound effects stop answering (the audio thread
  no longer takes commands), until the game is closed.
- **Seen**: suspected in play (stuck sound and stopped music), not reproduced; found by
  reading the code.
- **Fix**: `rc2k_core::audio::streams`: after seven plays in a row that stop at once the
  music falls silent (logged); a tune position past the last starts again from the first.

## Changing the sound mode leaves the music playing

*Original*: `audio::set_mode` (0x40A426, sound command 5). *Fixed*: 2026-10-20.

- **What**: before closing and reopening the sound device for a new mode, the original
  asks to stop the music streams, but passes whatever a register happens to hold as the
  kind of stop; usually that is "feed the stream", not "stop it".
- **In game**: changing the sound options does not stop the music before the device it
  plays on is closed and reopened (a glitch or a cut at the change).
- **Seen**: no; found by reading the code.
- **Fix**: the Rust version always asks for a stop (`audio::set_mode`).

## Quitting leaves the music and the stream playing

*Original*: `audio::close_all` (0x409DD5, the audio thread's end). *Fixed*: 2026-10-23.

- **What**: when the audio thread closes the sound, it first asks the music and the
  sound stream to stop, but passes whatever a register happens to hold as the kind of
  stop; at that point it is always "feed the stream", so neither is stopped (their buffers
  and files are not released) before the sound device is closed under them.
- **In game**: at quitting, the music's and the stream's buffers and files are left to
  the device's release and the process's end.
- **Seen**: no; found by reading the code.
- **Fix**: the Rust version always asks for a stop (`audio::close_all`).

## Text lookup with a bad string id reads anywhere

*Original*: `strings::expand_entry` (0x401FE1). *Fixed*: 2026-10-20.

- **What**: the game's texts (menus, messages) live in one table of entries that may name
  other entries. The lookup checks neither that an id is inside the table's index, nor
  that an entry's data stays inside the table, nor how deep entries nest. A bad id reads
  whatever memory follows, and an entry naming itself recurses until the stack runs out.
- **In game**: nothing with the game's own text files, whose ids are all valid; a damaged
  or edited text file could show garbage text or crash the game.
- **Seen**: no; found by reading the code.
- **Fix**: `rc2k_core::strings::StringTable::expand` checks the id, the entry's bounds and
  the nesting depth; a bad entry is refused (nothing written) and logged.

## Logged floating-point numbers carry leftover digits

*Original*: `logfile::log_float` (0x4014D8). *Fixed*: 2026-10-22.

- **What**: to log a number with a given count of decimals, the original writes its text
  into a small scratch buffer and puts the end marker where that count of decimals would
  end. When the text is shorter (more decimals asked for than it has, or "infinity" /
  "nan"), the marker lands past it, so the logged line also takes whatever earlier numbers
  left in the buffer.
- **In game**: nothing on screen; some lines of the game's log (`var\logfile.txt`) show
  stray characters after a number.
- **Seen**: no; found by reading the code.
- **Fix**: the Rust logger (`logfile::numeric::log_number`) logs the text only up to what
  was written: the marker's place, but never past the text's end.

## Crash driving over mud (particle ring full)

*Original*: `marks::emit_bursts` (0x4CD21A) and `marks::emit_bursts_2` (0x4CD747).
*Fixed*: 2026-10-23.

- **What**: wheels on soft ground (mud, gravel) throw bursts of particles into a shared
  ring of 0x3FE entries. Each burst's loop keeps its counter and a pointer on the stack.
  When the ring fills in the middle of a burst, the code jumps to an exit that removes
  only the pointer, not the counter, before restoring the registers and returning. The
  registers come back shifted by one, and the return goes to whatever value one of them
  held: a jump into nowhere.
- **In game**: with heavy particle use (driving through mud), the game ends with a
  "Guru Meditation" access violation at a small address (seen: IP 0x0000D852).
- **Seen**: yes, with the original game (the reason the owner's copy had the particles
  cut out) and with the converted game on 2026-10-23.
- **Fix**: the Rust version ends the bursts when the ring is full, keeping the caller's
  registers, as the intended exit does (`src/marks/emit_bursts.rs`). (For the hooked
  game, removed on 2026-10-23, the original's two exits were mended at start-up:
  `lea esp, [esp+4]` became `lea esp, [esp+8]` at 0x4CD741 and 0x4CDC6E.)
