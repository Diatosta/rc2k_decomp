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

## Some cars' marks lose their depth bias

*Original*: `marks::init_marks` (0x4D12F7). *Fixed*: 2026-10-23.

- **What**: a stage's marks get a depth bias from a table indexed by the car's kind, but
  the table has 0x17 entries and the kinds go to 0x21; for the cars past it the original
  reads the variables after the table (the bias itself, a flag, the number 9), a bias of
  about 0.
- **In game**: with those cars, the marks' depth is off (marks drawn as if without their
  bias).
- **Seen**: the standalone refused this read at a stage's loading (2026-10-23); the effect
  in the original was not looked at.
- **Fix**: kinds past the table get 18240.0, the value of most kinds (`marks::init_marks`).

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

## Split-screen divider left in a replay

*Original*: `hud::overlay_pass` (0x45A893), `hud::draw_view_strips` (0x45B7B1),
`hud::draw_view_strip` (0x45B9AA tail).
*Fixed*: 2026-10-23.

- **What**: the race draws its views by `dword_63DB98` (1 or 2 views), the HUD by its
  split layout `dword_6238A4` (0 one player, 1 / 2 the two splits). A replay sets the
  views back to one (`drive::choose_shown_cars`), but outside a race it leaves the layout
  as the split-screen race had it, so the HUD still draws the two-player layout over the
  one view.
- **In game**: after a split-screen race, the replay shows the black divider across the
  middle (with the two players' HUD) until the HUD is hidden.
- **Seen**: yes, by the owner in the converted game (2026-10-23); the Rust code was the
  original's, read from the disassembly.
- **Fix**: the HUD takes its layout as 0 while one view is drawn
  (`hud::overlay_pass::shown_layout`), for the HUD, the view strips and their tail.

## Escape on the "leave?" question leaves the screen anyway

*Original*: the screen loops' Escape clearing (`cmp g_clearsEscapeFlag, 1` before
`mov dword_755A3C, 0` in each loop, e.g. `menu::leave_question_loop` 0x47FAC4),
`g_clearsEscapeFlag` (0x7558D8).
*Fixed*: 2026-10-24.

- **What**: Escape sets the Escape flag (`dword_755A3C`). A screen clears it when it
  leaves, but only if `g_clearsEscapeFlag` is 1. Only four screens set that switch (Start
  Rally, Options, Player 1 details, Information), and nothing ever clears it. Until one of
  them has been seen, the flag outlives the screen it was pressed on.
- **In game**: load a saved championship straight after starting the game, finish a
  stage, press Escape on the review menu, and Escape again on the "leave?" question to
  say no. The review menu takes the leftover flag as a new Escape, and the championship
  ends back at the main menu.
- **Seen**: yes, by the owner in the converted game (2026-10-24). The behaviour is
  read from the original's code: the converted screens do what it does.
- **Fix**: the Escape flags are cleared whenever a screen leaves, as the original does
  once any of the four screens has been seen (`menu::screen_loop::clear_escape`).

## The intro plays over and over after a race left by the quit question

*Original*: `menu::play_films` (0x4161CD) calls the credits' race `race::credits_race`
(0x463666) without `race::clear_player_absences` (0x449D44), which its other caller
(`sub_443104`) calls first; `race::players_all_out` (0x445506) reads the first car
block's +0xB0.
*Fixed*: 2026-10-24.

- **What**: leaving a race through the quit question writes the answer into the cars'
  "out" field (+0xB0), and it stays until the next race's set-up. The film loop's
  demonstration (the credits' race between two showings of the intro) runs with one
  player, and the race loop ends a race as soon as every player is out: with the old
  answer still there, the demonstration ends at its first frame and the intro starts
  again.
- **In game**: quit a race (Escape, then yes), then choose View Intro on the main menu,
  or leave the main menu idle until the films start: the intro plays again and again,
  never the credits' race, until a key.
- **Seen**: yes, by the owner in the converted game (2026-10-24, twice); the cause read
  from the original's code (the converted functions do what it does).
- **Fix**: the film loop clears the players' "out" fields before the demonstration, as
  the original's other way into it does.

---

## No music in races with the Dolby mode, and the Dolby mode's quieter, harsher effects

*Original*: `audio::mix` (0x40BDC5), `audio::write_mix` (0x40C03A), `audio::mix_channels`
(0x40BECC).
*Fixed*: 2026-10-26.

- **What**: in the Dolby mode (sound mode bit 0x40000) the game mixes the effects itself
  and writes the result straight into DirectSound's primary buffer. A program that writes
  the primary buffer shuts out the secondary buffers, and the music is one. The game's
  mixer also leaves 12 dB of headroom: at full volume a sample is scaled by 0x1000 / 2^14,
  a quarter, while the other modes play the effects at full level. And it resamples
  each sound by taking the frame its position is in, with no blending between frames, so
  the 22050 Hz effects played at 44100 Hz, often pitch-shifted, gain false high tones.
- **In game**: with Dolby chosen in the advanced audio options, races have no music, and
  the effects are clearly quieter than in the other modes, with a harsh edge.
- **Seen**: yes, by the owner without the DLL (2026-10-13: no music); the level difference
  heard in the converted game (2026-10-26) and read from the code.
- **Fix**: the standalone's sound output (`platform_sound`) plays the primary buffer and
  the secondary buffers together, so the music is heard in the Dolby mode; the secondary
  buffers play at a quarter (`SECONDARY_GAIN`, -12 dB), the level of the game's own mixer,
  so every mode sounds alike and the music keeps the same balance with the effects. The
  game's mixer blends each frame with the next by the position's fraction, as DirectSound
  resamples (`rc2k_core::audio::mixer`). The harsh edge heard by the owner in the
  original game too (2026-10-26), and gone with the blending.

---

## A PNG picture's first row read against stray memory

*Original*: `gfx::decode_png` (0x4D3040).
*Fixed*: 2026-10-27.

- **What**: PNG rows are stored filtered: each row may be coded as its difference from
  the row above it. For the first row the standard says the row above is all zeros. The
  game instead reads the memory just before its picture buffer there: the bookkeeping of
  its memory allocator, and for wide pictures part of the page before.
- **In game**: a PNG whose first row is coded against the row above (filters Up,
  Average or Paeth) would show wrong colours in its first pixels, top left.
- **Seen**: no; read from the code. The game's PNG files may not use those filters on
  their first row.
- **Fix**: the first row is unfiltered against zeros, as the standard has it. The
  picture buffer is Rust's own memory since the conversion's memory phase, so the bytes
  before it are no longer there to read.
