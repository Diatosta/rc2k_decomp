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
- **Fix**: the standalone's sound output (`platform::sound`) plays the primary buffer and
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

## Closing the window leaves the game running

*Original*: `system::window_procedure` (0x41057A), `system::pump_messages` (0x410535).
*Fixed*: 2026-10-28.

- **What**: closing the window shuts the screens down and asks Windows to quit, but the
  game's message pump swallows that request without telling anything to stop.
- **In game**: closed during a film or a menu, the game goes on running with no window
  and takes no input; it has to be ended from the task manager.
- **Seen**: yes, in the conversion before the fix (the intro); read in the original's code.
- **Fix**: closing the window sets the game's quit switch, which the menus and races
  already obey; films now stop on it too, and if anything is still running 5 seconds
  later the program ends. The installer is left to finish its copy, so an install is
  never left half done.

## The "leave?" question loops once the game is quitting

*Original*: `menu::main_menu_loop` (0x474B43), `menu::leave_question_loop` (0x47FB9A).
*Fixed*: 2026-10-28.

- **What**: screens that ask before leaving treat the game's quit switch as a request to
  leave and put the question; the question's screen closes at once when the switch is
  set, with no answer, so the screen asks again, round and round.
- **In game**: once quitting, the menus never got out; with the window closed (the
  previous fix) the program waited for its 5-second safety net.
- **Seen**: yes, in the conversion (closing the window in the main menu).
- **Fix**: while the game is quitting, the question is not put and the answer is yes.

## Options lost when the game is closed by its window

*Original*: `system::main_loop` (0x415383).
*Fixed*: 2026-10-29.

- **What**: the game saves its options and best times on the way out of the main menu,
  but skips the save when its quit switch is set; in the original only a fatal error set
  it there.
- **In game**: since closing the window quits the game (the fix above), closing it lost
  every option (controls, names, screen and sound settings) and best time changed since
  the last save.
- **Seen**: yes, in the conversion (the options were not kept after closing the window).
- **Fix**: the save is made on that way out too.

## The frame rate never measured below some 28 frames a second

*Original*: `race::measure_frame_rate` (0x42009E).
*Fixed*: 2026-10-30.

- **What**: every 16 frames the game subtracts the low halves of the processor's cycle
  counter and tests the difference as a signed number, so a wait of 2^31 cycles or more
  counts as no time at all and the measure is dropped.
- **In game**: at 3.8 GHz, 16 frames take that long below some 28 frames a second: the
  frame rate stays at its last value (0 from the start) and its "fps" text is not drawn.
  Harmless on the processors of 2000, where 2^31 cycles took seconds.
- **Seen**: yes, in the conversion (0.0 in the debug UI at 18 frames a second).
- **Fix**: the whole 64-bit counter is kept and subtracted.

## The section best times drew a second, wrong sprite

- **What**: `screens::draw_section_best_times` (0x46762B) drew each row's car sprite, then put the sprite's number through the same "car to sprite" step again and drew that too, at the same place.
- **In game**: never, in the original: nothing reaches this screen. Opened from the debug UI, rows showed stray pieces of other sprites over their cars: red bars, a loading bar, badges.
- **Seen**: yes, in the conversion's debug UI.
- **Fix**: the second sprite is gone.

## The championship's welcome speech cannot be skipped

- **What**: `menu::championship_welcome` (0x47F314) waits in `system::wait_or_key` (0x407F69) and then polls the stream's state and the pending key, without pumping the window's messages.
- **In game**: choosing a car for a championship freezes the window for the length of the speech (about 25 seconds): a key does not skip it and closing the window does nothing until it ends.
- **Seen**: yes, in the conversion, and reported in the original.
- **Fix**: both waits pump the window's events, and the polling loop sleeps 10 ms a turn instead of spinning. A key now skips the speech and closing the window ends the wait (`system::wait_or_key`, `menu::flow_game`).

## Closing the window loses the session's records

- **What**: on the window's closing the game leaves without `saves::save_game_data` (0x411499), which runs only when the front end is left by its menu.
- **In game**: best times and options changed since the start were gone if the game was closed with the window instead of "exit to Windows".
- **Seen**: yes, in the conversion: a new best time was missing from `var\hst.dat` after a close.
- **Fix**: `standalone::leave_from_race` saves first. Only a game stuck for 5 seconds after the close ends without saving.

## A saved game chooses where a name is written

Read in the code, and tried with damaged files; not seen in normal play. A saved car holds the
address of its player's name, and loading took that address from the file. The next load copied
the name to it, so a damaged or hostile save could overwrite other data with 32 bytes. A
loaded car now keeps the game's own address, which is the same one a sound save holds.

## A full-length chat message writes past its buffer

Read in the code, not seen. A chat message that fills the 256-byte receive buffer had its end
marker written one byte past it, over the first byte of the message being sent. The last
character of such a message is now given up to the marker instead.

## A long chat message overruns the chat line

Read in the code, not seen. The lobby builds "name> text" in a 128-byte buffer without looking at
its length, while a received message can hold 255 bytes. Another player could overwrite the
network mode and the data after it. The line is now cut to the buffer's room.

## A long folder name in the install record overruns the game's paths

Read in the code, not seen. The game copies the install folder and language folder from the install
record without checking their length, so an over-long value overwrote the data after them.
The game now stops with a message naming the path.

## A damaged replay file picks a camera from outside its table

Seen by damaging a saved replay on purpose, under a memory checker. A byte in each replay frame
chooses the camera from a table of 47 entries, and the game does not check it, so a damaged file
read memory well past the table. The game now stops with a message naming the table.

## More than 256 flares in a frame overwrite other data

Read in the code, not seen. The game's six flare tables hold 256 entries each, but it counts up to
1024 flares in a frame, so each table ran over the next and the last ran past its end. Flares past
the tables' size are now left out.

## A network message can end or stall another player's race

Seen by sending damaged race messages on purpose. The game copies each message from another player
straight into that player's car, with no checks, so a gear of 8 read past the gear table (now the
game stopped instead), and values that are not numbers kept the car searching for a clear place on
every step, dropping the frame rate to almost nothing. A race message holding values no car could
have is now ignored, with one line in the log.

## One long network message stops all network messages

Read in the code, then fixed with the rest of the network limits. The game can only take messages
of up to 256 bytes, and a longer one stayed first in the queue for good, so nothing behind it was
ever read. Our network layer now drops messages the game could not take.

## Some collision faces name an object that was never loaded

Seen in play: the game ended with "an object kind past the 0x4B0 material records" on stage 22. A
scan of every stage found the cause on stages 22, 34, 52, 61, 63, 64, 65, 66, 67 and 68: faces naming an
object slot that holds nothing. The original reads their material from the bytes before the table,
gets a contact kind of 0 or 255 and does nothing, so the faces have no effect in play. That read triggered
our check. The unloaded slot's value now reads the same bytes, and any other bad value still stops
the game.

## A network client's cockpit does not animate

Seen in play, on the machine that joined: the driver's arms did not move and the windscreen glare was
missing, while the host looked right. The same car in single player was also right. Many places in the
game use the first car's block when they mean the player's car, and on a joining machine the first car
is the host's. The cockpit effects, the mud and the sound listener now follow the local car. The original
has the same fixed addresses, but its network mode could not be run here to see it.

## Cars warp and jerk in network races

**What it is:** a position message snaps a remote car to the place received when it is too far from where the car is. The limit compared with is 10.0, but places are in 1/65536 of a metre, so the limit was 0.00015 m: after a car's first 1000 steps (ten seconds) every message, about three a second, snapped it instead of gliding it. A snap also left a glide under way running on a field that then held a place.

**In game:** the other player's car jerked, and now and then jumped tens of metres back along the road for a third of a second. Moved from outside the physics, the car's probe points crossed face edges; the contact rounds ran out and the car was reset to the line, on that machine only, until the next message.

**Seen or read:** seen with two copies racing (a logged jump of 42 m, none before step 1000) and read in the disassembly at 0x4647A8. The original's own network could not be run for comparison.

**Fix:** the limit is ten metres, and a snap ends any glide. Measured with two copies: largest gap from 13.5 m to 1.5 m, jumps over 3 m from 123 to 0.

## A network client's dashboard is blurred

**What it is:** the game loads car 0's second set of textures, the cockpit's, at full size and every other car's at half size. In a network race a client's own car is not car 0.

**In game:** a client's dashboard and other cockpit textures were visibly blurred compared with the host's.

**Seen or read:** seen in screenshots of two copies racing, and read in `scene::init_texture_levels` (0x424001).

**Fix:** the car this machine drives, and any car of the same model, get the full size; away from a network race that is car 0 and its like. Two cars of one model need both: the cockpit stayed blurred with only the own car at full size.

## "Remove all" leaves the game's folder behind

**What it is:** the uninstaller removes only the files and folders its install script lists, then the folder itself. Anything else in the folder (the program's own log, files the game or the player added, and on Windows the running program and its libraries) keeps the folder from being removed.

**In game:** after choosing to remove everything, the game's folder was still there with a few files in it.

**Seen or read:** seen with the standalone in a test installation (a file added to the folder stayed), and read in `install::uninstall` (0x4134DF).

**Fix:** with "remove all" chosen, whatever is left of the installation's folder is removed once the listed files are gone; on Windows, what the running program still holds goes right after it ends. Only a folder that holds the installation's record and is neither a drive's root nor the home folder is removed this way. "Keep saved data" is unchanged: the saved games and settings stay.

## The second player's car is not repaired between split-screen stages

**What it is:** before each stage of a two-player split-screen single rally, the game means to repair both cars: it puts 0 and then 1 in a register and calls the repair twice. The repair does not read that register; it repairs the current player's car, and the current player is 0 both times.

**In game:** in a split-screen single rally, player one's car started each stage as new, while player two's kept the damage of the stages before (worn tyres, a bent body, broken lights).

**Seen or read:** read in `menu::single_rally_split` (0x48260C .. 0x48261B) and `service::repair_car` (0x486F63, which reads `g_currentPlayer`); not seen in play.

**Fix:** the current player is set to each of the two in turn for its repair, then back to 0 for the race, as the original has it.

## Four tables read past their end

**What it is:** four lookups take an index without checking it against their table and read on into whatever the program has after it: a car's kind in the menus' car carousel (35 entries; the original allows numbers up to 0x400), a kind's own place in the carousel (73 entries; the original allows kinds up to 0x88), a stage's duration on the stage panel (37 entries, the index read from the stage's entry), and a driver's name in the standings and the default best times (180 entries).

**In game:** none seen. With the game's own data every index is in range; a damaged or edited data file would make the menus show a number taken from an unrelated variable.

**Seen or read:** read in `scene::set_up_carousel`, `scene::draw_carousel_entry` / `draw_highlighted_object`, `screens::stage_panel` and the drivers' table (`g_driverNames` 0x741E88); not seen.

**Fix:** an index past its table is refused (the program stops, naming the table) instead of reading the memory after it.

## The installer's folder walk reads past the path

**What it is:** the installer makes each folder it copies into from a path with no closing `\`. Its walk looks for the next `\` after each folder name, and for the last name it runs past the path's end into the buffers stored after it in memory. Its first cut there leaves the path whole, so the last folder is still made; every `\` it meets after that only goes into the same folder again, until one is followed by a NUL.

**In game:** none seen with the original: the bytes after the path happened to hold a `\` soon enough. In the conversion, where the path has a buffer of its own, the walk reached that buffer's end and the Windows installer stopped while copying.

**Seen or read:** read in `install::make_folders` (0x413A97) and seen in the standalone's trace (2026-10-20); the stop seen under Wine (2026-11-07).

**Fix:** the walk treats the path's end as its closing `\`: the last folder is gone into or made there, and the walk ends without reading past the path.

## A championship's settings stay after it

**What it is:** a championship, started or loaded, works on the players' own records: the car chosen, the gearing, the controller and its set-up, the names. Loading a saved championship puts the saved records over the players' whole, and nothing puts the players' own back when the championship is over or left.

**In game:** after playing a championship, a single rally or the arcade starts with the championship's car, gearing and controls; leaving the front end then writes them into `var\ral.dat`, so they stay for good. Settings changed on the player pages before loading a championship are lost the same way.

**Seen or read:** seen in the original (2026-10-24, settings changed in one run were back in the next after loading a save) and read in `saves::load_savegame` (0x46E6E1).

**Fix:** the four player records and names are set aside before a championship starts or is loaded and put back when it ends or is left, on every way out. Inside the championship its own settings are used and saved as before. (Which of the save's or the player's controls and gearing a loaded championship uses is still `usesavegamekeys` in `var\ral.cfg`.)

## Course faces drawn with the last face's depth setting

**What it is:** course faces of mode 4 without texture animation (see-through overlays) are meant to be drawn without writing to the depth buffer, with depth writing back on afterwards. Each side of a face does that with its own texture and mode. But the code that draws a whole course face wraps its sides in the same switch a second time, judged by the previous face's texture and mode, before the face's own are set. And the switch back on is the wrong way round: it turns depth writing on only when it was not turned off.

**In game:** after a mode-4 face, the next face can be drawn without writing depth, and depth writing can stay off for a face longer than meant, so a shape drawn later behind it can show through for a frame. Rare and at most a few pixels; not seen. In the conversion it showed up as the drawing on several threads (each starting part-way through a list, without the previous face's leftovers) differing from the drawing on one.

**Seen or read:** read in `race::draw_course_face` (0x45232A), `race::face_depth_test_off` (0x452358) and `race::face_depth_test_on` (0x45237A); found 2026-11-08 by the multi-threaded drawing's comparison with the single-threaded one (replay 01).

**Fix:** each face's sides are switched by their own texture and mode only (the outer, stale switch dropped), and depth writing goes back on after a side that turned it off.

## The course's sounds placed from the mirror's camera

**What it is:** after the views are drawn, the game finds the course's sound spots (running water, crowds and the like) around the camera and places each one against it. It does this once for the whole frame, after every view, with whatever camera was drawn last. When the rear-view mirror is on, that is the mirror's camera, which looks backwards; with two players it is the second player's view (or its mirror). The race's sounds are heard around the first car.

**In game:** with the mirror on, the course's sounds are placed as seen from behind the car: what is on the left is panned to the right, and the spots chosen are the ones round the mirror's view. Subtle; not reported.

**Seen or read:** read in `race::draw_race_scene` (0x45C605 called after the views) and measured with the conversion's frame census (2026-11-08: the spots read the view matrix and grid the mirror left).

**Fix:** the sound spots are found right after the first view is drawn, from its own camera and grid.

## The race's sounds steer the race's random numbers

**What it is:** the game has one random number generator. The sounds draw from it (which of two nearby sounds is heard, the stage's random noises and their delays, a rare engine sound) in the same stream as the race itself (the cars' handling on some surfaces, the mud, the replay cameras, the camera's shake). So how many numbers the sounds draw changes every number drawn after them.

**In game:** what the sounds do changes, in a way nobody could notice, what the race does next: the mud thrown, the replay's camera choices, and in a race the computer cars' handling on some surfaces. In the conversion it showed up when the sounds were changed on purpose (the fix above): replays drew different mud and cameras although the cars were the same.

**Seen or read:** read in `sound::thin_spot_voices`, `sound::stage_sounds`, `sound::repeat_spot_voices` and `drive::duel_sound_levels`; measured 2026-11-08 (the cars the same tick by tick, the random state different).

**Fix:** the sounds have a generator of their own, started from the same seed. Nothing the sounds do changes the race or the pictures any more. (The race's own random numbers therefore differ from the original's after the first sound draw: the same kind of numbers, but not the same sequence.)

## The pictures steer the race's random numbers

**What it is:** the same single random number generator also serves the drawing: the mud thrown onto the car's textures and the shimmer of some objects draw numbers from it every frame they are drawn. The race draws from it too, for the camera's shake and for the chance of a course face being marked when a car touches it. How many numbers the drawing takes depends on how many frames are drawn, so the race's numbers depend on the frame rate.

**In game:** on a faster or slower machine, or when a frame is slow, the camera shakes a little differently and different faces get marked. Nobody could tell, but it means the same drive is never the same twice.

**Seen or read:** read in `race::throw_mud_texel` (0x44A2A2), `race::splash_mud` (0x44A35F) and `scene::set_object_material` (0x45170E), against `physics::camera_frame` (0x44FF1B) and `physics::touch_face` (0x45E9E0); found 2026-11-08 by the conversion's frame census and a driven race replayed with a held clock.

**Fix:** the drawing has a generator of its own, started from the same seed. The race's numbers no longer depend on what is drawn or how fast.

## The race's sounds placed from the mirror's camera

**What it is:** the sounds the cars make (the engine, knocks and hits) are placed against the camera, turned by the view's matrix. The game uses whatever matrix the last view drawn left behind. With the rear-view mirror on, that is the mirror's, which looks backwards, while the listener faces forwards. With two players it is the second player's view (or its mirror).

**In game:** with the mirror on, the car's sounds are turned half round: what is to the left is placed to the right, and in front is placed behind. Subtle, since the player's own car is close to the listener; not reported.

**Seen or read:** read in `physics::place_car_sound` (0x45CC5C), `physics::car_sound_state` (0x45CACE) and the car's sound places (`physics::car_sounds`), which read `g_viewTurnMatrix` during the race's ticks; found 2026-11-08 by the conversion's frame census.

**Fix:** the first view's matrix is kept once that view is drawn, and the race's sounds are placed by it.

## An object with many faces writes over the object tables

**What it is:** before drawing an object the game lists the faces it will draw, 16 bytes each. The list has room for 1,024 faces, but the check before adding one allows 4,096. From the 1,025th face on, the entries are written over what follows the list in memory: the material's timing, the list of textures, and each object slot's first point and point count.

**In game:** only an object with more than 1,024 faces facing the camera at once would do it; none of the game's own objects came near it in the drives and replays measured. If one did, the objects drawn after it would take their points and textures from the wrong places until the stage was loaded again: broken or missing objects.

**Seen or read:** read in `scene::list_object_face` (0x4504CB), `scene::sort_listed_faces` (0x45098C) and `scene::draw_object_faces` (0x450A4B); found 2026-11-09 by the conversion's survey of the drawing's working state.

**Fix:** each object's drawing has its own list, with room for the 4,096 faces the check allows. Nothing past it is touched.

## Each view lit by what the view before it left

**What it is:** the game draws its views one after another (the main view, the rear-view mirror, the second player's). Three values the drawing of a view reads were left by the view drawn before it: the light added to the view's own car was that of the last car the previous view drew; the sky's extra light was that of the previous view's last object; and the cars' shine started from the previous view's last car's, not from the stage's.

**In game:** barely visible: only the snowflakes' tint, by at most one step in 255, in a few frames with the mirror on. Not reported.

**Seen or read:** read in `race::view_light` (0x455896), `race::draw_sky` (0x455540) and `race::draw_race_scene`, which sets the shine once a frame; found 2026-11-09 by the conversion's survey of what each view leaves to the next.

**Fix:** each view takes its own car's light colour, the sky's extra light is 0, and the shine starts from the stage's at each view. A view now depends only on its own inputs.

## The reverb can be chosen from a half-made sum

**What it is:** the game chooses its reverb (the EAX room) from how much of the course is close to the player's car: each nearby face adds a weight, summed again for every view drawn. The sum is made in place in a variable that the sound thread reads whenever it runs. If the sound thread runs while the sum is being made, it takes a partial value, as low as zero.

**In game:** at most a moment of the wrong reverb, then the right one again on the next turn. Not reported; probably not audible. The port's own reverb has since been taken out, so nothing reads the sum any more (see [Sound and music](Sound-and-music)).

**Seen or read:** read in `race::eax_environment` (0x45C7EE) and the sound thread's voice update; found 2026-11-09 by the conversion's work on what each view hands on.

**Fix:** the weight is summed apart and handed to the sound whole, at the end of each view. (Later the weight was not worked out at all any more: nothing heard used it.)

## The mirror's backdrop drawn with whatever the last polygon left

**What it is:** before the rear-view mirror's scene, the game draws a dark rectangle behind it. It sets the rectangle's corners, texture place and colour, but not their depth and two other values: those are whatever the polygon drawn before it left in the same memory.

**In game:** nothing visible: fog is off for that rectangle, the leftover depths were close enough to the front to pass, and its colour is the same at every corner, so the other values cannot change a pixel. It would only have mattered had the polygon before left a depth behind what the main view drew there.

**Seen or read:** read in `race::draw_view_backdrop`; found 2026-11-10 when the mirror was first drawn on a thread of its own, where the leftovers were different ones.

**Fix:** the three values are set (to 0) with the rest, so the rectangle no longer depends on what was drawn before it.

## The rain's streaks given an absurd depth value

**What it is:** each raindrop is drawn as a thin four-cornered streak. The game sets its corners' perspective value (1/w) from a constant meant to be 256, but reads the constant's bytes as a whole number instead of as the decimal number they hold, so every streak gets 1,132,462,080.

**In game:** nothing on the original's Direct3D, nor in our PC builds: all four corners share the value and the depth buffer is off for the rain, so it cannot change a pixel. On the handheld's OpenGL ES the value made the streaks vanish entirely: the rain could be heard but not seen.

**Seen or read:** seen on the handheld (2026-11-10); traced to `effects::run_rain` (0x4C16F9), whose `fild` loads the float constant at 0x8C5F91 as an integer, where the windscreen drops and flakes load a true integer 256.

**Fix:** the constant is used as the decimal number it holds (256), as for the drops and flakes. The rain shows on the handheld; nothing changes on the PC.

## A car's lights chosen by the other player's camera

**What it is:** before drawing a view, the game decides which of the shown car's lights to draw from that view's camera (from inside the car some are not drawn). But it reads the camera values before the view has set them, so it takes whatever the view drawn before left there.

**In game:** with one player, after a camera change the car's lights follow the old camera for one frame. In split screen, the first player's view takes its car's lights from the second player's camera, drawn last in the frame before. Not reported.

**Seen or read:** read in `race::car_light_modes` (0x455B70) and `race::camera_view`; found 2026-11-10 when each view was given its own camera values.

**Fix:** the lights follow the shown car's own camera settings, read from the car itself.

## The reverb chosen by the mirror or the other player

**What it is:** the race's reverb (the EAX "room": open road, forest, tunnel...) is worked out while each view is drawn, from the course around that view's camera. Every view drawn sets it, so the rear-view mirror's, or in split screen the second player's view, replaced the first player's; the sound took whichever was set when it ran.

**In game:** with an EAX sound card of the time, the reverb could follow the mirror's view or the other player's surroundings instead of the player's own. The port had a reverb of its own for a time, which followed each player's own view; it has been taken out (see [Sound and music](Sound-and-music)).

**Seen or read:** read in `race::eax_environment` (0x45C7EE) and the frame's views; found 2026-11-10 while drawing the views on several threads.

**Fix:** the reverb comes from the first player's main view only. (Later the weight was not worked out at all any more.)

## Tyre wear on snow, ice, water, dust and grass taken from the wrong surface

**What it is:** the tyres' life comes from a table of figures per tyre type and surface. Each surface has two figure columns, blended by how much of a "surface noise" the ground has there. The game picks the first column by the surface, but looks the second one up as if the first column's number were a surface. Tarmac, gravel, mud and rough ground come out right by chance; on snow, water, ice, thick snow and snow gravel the second figure is the one for dust and grass (half the tyre life), and on dust and grass it is the snow one (double).

**In game:** nothing. The blend comes from each stage's surface noise, and every stage of the game has its noise scale and floor at 0, so the second column always has no weight. It would only show on a stage made with surface noise.

**Seen or read:** read in `physics::damage_wear` (0x4AC7C9: `dword_75B8C4[column * 8]`); checked 2026-11-10 with a build that worked out both figures on six rallies' stages (the blend was 0 everywhere) and by reading every stage's LOCALDAT.

**Fix:** both columns are taken from the surface's own pair.

## The lamps a view starts from taken from whichever view was drawn last

**What it is:** while a view is drawn, the game gathers the lamps (car lights and lit objects) that light the scene. The view's light and the sky read the lamps left from before, which in the original were the previous view's, as the views were drawn one after another. With the views and mirrors drawn side by side in the port, the previous one was whichever had finished last, a matter of timing.

**In game:** nothing seen: the old lamps matter only for a frame's tint on stages with lit objects of one kind (0x3B), and the held races came out identical.

**Seen or read:** read, in `race::draw_view_scene` and `ViewScratch::begin`, 2026-11-12.

**Fix:** each view and mirror keeps its own lamps from frame to frame and starts from them, so the result no longer depends on the order the views finish.


## Divisions by zero in the effects that left NaNs ("not a number")

**What it is:** several pieces of the effects' code divide by a length, a height or a count without looking at whether it is 0, and on the x87 a 0 / 0 gives a NaN (an infinity times 0 too). The places found:
- `effects::view_motion` (0x4C4D40): the view's step since the last frame made length 1 even when the view has not moved (`scene::normalise_vector3` on a zero vector), its "facing" then a NaN; and that step divided by the frame's ticks when there are none (the stage's first frame).
- `effects::view_heading` (0x4C4CEC): the view's forward direction's level part made length 1 when there is none (its matrix not set yet on the stage's first frame), so `scene::angle_between_vectors` compared NaNs sixteen times and the heading came out of that.
- `effects::span_edges` (0x4BE315): an edge pair of no height (two corners of a mask's triangle at the same height) given slopes over a height of 0.
- `effects::scale_corners` (0x4C6420): the slope of a corner record with both sides 0 (the stage's unused records).
- `effects::set_kind_values_1` / `_2` (0x4C7322, 0x4C7FA4): the slope of the needle / shadow angle's line above its split for the cars whose line has no length (`high` equal to `low`).
- `effects::measure_lines` (0x4D0EB4): a sum's share of a total of 0, whose NaN `fistp` stores as the integer 0x80000000.

**In game:** nothing. The facing, the heading and the per-tick step are read by nothing in the game (only the port's debug overlay shows them); the spans' slopes are kept and never read, and their rows are none; the corner and needle slopes would only be read on paths none of the test races or replays reached. The 0x80000000 a share of nothing leaves as its sum stays. The NaNs mattered for the port: their exact bits depend on the processor (an x87, an x86-64 and an ARM processor each make a different NaN), so they made the results platform-dependent in principle and kept a costly NaN rule in the x87 emulation under discussion.

**Seen or read:** found 2026-11-13 by counting the NaNs the x87 emulation met in held races (Langton, rain, snow) and the four replays: 3,800 to 14,000 a race, all from these places, at a stage's load and in a replay's first seconds (the camera still).

**Fix:** each degenerate case is handled before the division, keeping what the game did: a view that has not moved faces no way (facing 0) and a frame of no ticks keeps its step whole; a view with no level direction keeps its heading; a half of no height has slopes 0; a corner record of no size has slope 0; a line of no length is flat (slope 0); a share of nothing is 0x80000000 without the division. The same races and replays now meet no NaN, and their results are the same as before to the byte.

## A reflective face's shine scaled by the previous polygon's fog

**What it is:** an object's face with a reflection (the cars' paint and glass, shiny scenery) is drawn twice, the second time with the reflection map, whose strength is the face's shine times the stage's shine times the face's fog. The game took that fog from the first slot of the polygon work table before the face's own points were projected and fogged, so it read the fog the previous polygon drawn had left there.

**In game:** the reflection of the first reflective face of an object (the player's car's first one each frame) was a little stronger or weaker depending on what was drawn just before it: on Langton, the car's own light flares or the scenery behind it, whose fog bytes were 201 to 208 of 255. That is about one step of the reflection's 256 levels of alpha on a few of the car's polygons. The faces after the first one read the face drawn just before them on the same car, close to their own fog, so they were nearly right.

**Seen or read:** found 2026-11-13 while drawing the player's car on a thread of its own: the car drawn alone differed from the car drawn after the scenery in one alpha step of one reflection fan, and the cause was narrowed to the work table's entry 0 fog byte (`scene::draw_object_face`, 0x450BDA, reading it before `scene::fog_polygon`). Read in the port; the disassembly was not checked in this step.

**Fix:** the face's shine is scaled by its own fog, the fog `scene::fog_polygon` gives its first vertex once the face is projected (the same table and depth). Nothing else changes: held races keep every tick and sound turn; only the reflection pass's alpha on reflective faces differs.

## A gear outside the gearbox read past the HUD's sprite table

**What it is:** the HUD draws the gear from its sprite table at entry 0x41 plus the gear. The game did not check the gear, so a gear outside -1 to 7 read a rectangle from the fields after the table.

**In game:** never in ordinary play: the physics keeps the gear between reverse and seventh, and every gear change refuses one outside. Only a replay file damaged or edited by hand can carry such a gear; the game then drew a sprite from whatever bytes followed the table. Before this fix the port ended with "a HUD sprite's rectangle past the rectangle table".

**Seen or read:** read in the port (2026-11-12) while bounding the HUD's tables; not seen in the original.

**Fix:** the gear is clamped to -1 ..= 7 where its rectangle is chosen (`hud::layout_common::gear_rect`), and the game log says once a race which gear was read and which one is shown ("hud: gear 40 outside -1 ..= 7 (a damaged replay?), shown as 7"). Gears in range draw exactly as before.

## A loading-bar frame that could not be drawn ended the game

**What it is:** while a stage loads, the game draws a loading bar frame over the loading picture. When a frame could not be drawn (the window not the active one, its surfaces lost, or the frame's lock refused), the original called its surface restore routine and went on from there. The Rust version had not ported that routine and ended the program ("not ported") instead.

**In game:** switching away from the window or losing the display surfaces during a stage load would have ended the game instead of only pausing the bar.

**Seen or read:** read, not seen. The standalone keeps its window flag active and never marks the surfaces lost, so a headless run could not make the case happen; found 2026-11-14 reading `race::loading_bar_frame`.

**Fix:** the frame is skipped (nothing is drawn or presented that frame), the load goes on, and the next frame tries again (`race::loading_bar_frame`). The platform restores the window and its surfaces by itself.

## A race frame that could not be shown ended the game

**What it is:** as with the loading bar, when a race frame could not be drawn or shown (its lock refused, the present failed), the original called its surface restore routine. The Rust version had not ported that routine and ended the program ("not ported") instead.

**In game:** a display hiccup during a race would have ended the game.

**Seen or read:** read, not seen; the same case as the loading bar's, found 2026-11-14. A test switch in remote builds (`RC2K_LOSE_FRAMES=n`) now loses every n-th frame's lock, so the path can be run.

**Fix:** the frame is skipped and the next one tries again (`drive::race_frame`, `race::show_frame`). The race's results never depend on what was drawn: held races with frames lost match normal ones tick for tick.

## The first intro film's pictures late against its sound on the handheld

**What it is:** the film player timed its pictures by the wall clock, counted from before the first picture was shown, and never caught up when late. On the handheld the GPU renderer's start with the first picture took about a second while the sound was already playing.

**In game:** the first intro film (`MFLOGO.MPG`) ran up to a second behind its sound on the handheld; the second film was in step.

**Seen or read:** seen (reported on the handheld, measured: +1,020 .. +573 ms late).

**Fix:** the film's sound starts once the first picture is on the screen, and from then on the pictures follow the sound's played position (`film::Clock`). Both films stay within about 20 ms of their sound, with no frames skipped.

## The drive page with 25 drives lists an item that is not a drive

**What it is:** the drive page's largest menu holds 26 items: two fixed ones and 24 drives. The page counted every drive from `a` to `y`, so with 25 drives (letter `a` among them) it counted 27 items, one more than the menu holds, and read the 27th item from the bytes that follow the menu.

**In game:** a garbage item at the end of the drive list, with text that is not a drive. Not seen; found by reading the code (a PC with 25 installable drives is very unlikely).

**Fix:** the page lists at most 24 drives, the first 24 in letter order, so with 25 drives `y` is left out (`drives::kept`, `install::list_drives`).

## A scene object whose kind slot was never filled ended the game

**What it is:** a course's scene objects name an object slot, and the slot's kind is set only when that slot's file is loaded; every other slot holds -1 after the objects are reset. The original indexed its table of 1200 object-material records with that -1 and read the record before the first, which is the last bytes of the texture-info table in front of it (the game's own tables are laid out one after the other). The Rust version refused a kind past the 1200 records and ended the program.

**In game:** the game ended in the middle of a long stage (Ulster Rally stage 2, the last rally's stage 2, arcade level 2: every object with an unloaded slot reached it). Found by the championship and arcade runs; a single stage started directly did not reach it.

**Seen or read:** seen (headless championship and arcade runs); the original reads the same bytes and carries on.

**Fix:** a kind of -1 reads the record before the first (`scene::polygon::material_run_byte`) in the object's matrix, its contact material, its contact sound and the lit faces' animation, as the original does, so nothing ends the game.

## A computer named "andrew" could not set the best tyre-change time

**What it is:** at start the original asked for the computer's name, and on a machine whose name begins with "andrew" (in any case) the tyre change's gauge ignored a press made in the top band of the gauge: it ran on until the band was left. A developer's machine was kept off the record that way.

**In game:** on such a computer the fastest tyre changes could not be made, so the tyre-change record could not be beaten with a near-perfect press.

**Seen or read:** read.

**Fix:** the name is no longer looked at; a press ends the gauge on every machine (`rc2k_core::race::tyre_change::turn`).

## The first logo film played only on 16-bit screens

**What it is:** at start-up the original played `allogo.avi` only when the screen had 16 bits a pixel; on a 32-bit screen it went straight to the second logo.

**In game:** with the menus at 32 bits (every saved set of options of today) the first logo was never seen.

**Seen or read:** read (and seen: the film never played in 32-bit runs).

**Fix:** the screen always has 32 bits a pixel now, and the first logo always plays before the second (`rc2k_core::menu::films::play_films`); `nofmv` still skips every film.

## A sound that could not be started would have slowed the audio thread to one turn a second

**What it is:** the original watched for sound buffers that Windows had taken away for another program: a failed call set a flag, and while the flag was set the audio thread waited a second each turn before it tried to restore every buffer. Buffers are never taken away here, but the flag was also set by any failure to start the primary buffer, and then nothing cleared it unless the restore ran with the window active.

**In game:** after such a failure every audio turn would take a second instead of 20 ms: sounds late by up to a second, and every wait for the audio thread (a stage's start, a race's end) a second or more.

**Seen or read:** read, not seen (the platform's play call fails only for a buffer that is not there).

**Fix:** the lost-buffer flag and its restore are gone (`audio::restore` removed). A play that fails is written to `rc2k.log` and that sound stays silent.

## The menus took the last race's mipmapping

**What it is:** the number of texture levels the menus' scene was made with came from a race setting that only a race's start wrote, from the mipmapping option. The menus never set it themselves.

**In game:** the menus' textures had one level when the game started and three after any race (until the credits ran, which put it back to one), so the menus could look slightly different after racing.

**Seen or read:** read.

**Fix:** the menus' set-up puts the setting back to one level each time (`scene::set_up_front_end_scene`); races still take the option as before.

## A key blinks on for ever after Escape on the "redefine keys" pages

*Original*: `menu::redefine_key` (0x47D472).
*Fixed*: 2026-11-17.

- **What**: choosing a key to redefine marks its place; a new key clears the mark, but Escape ended the redefining without clearing it.
- **In game**: after pressing Escape instead of a key, the chosen key's slot kept blinking, on every player's page, until some key was stored somewhere.
- **Seen or read**: seen by the owner, with and without the hook, 2026-10-14.
- **Fix**: Escape clears the mark too (`src/menu/redefine_keys.rs`).

---

## Typing a replay's name changed the saved games' list

*Original*: `menu::edit_replay_name` (0x46F4E7).
*Fixed*: 2026-11-17.

- **What**: the replay name editor's clear key, backspace and its "name too wide" cut acted on the highlighted name of the saved games' list while the typed length they changed was the replay's.
- **In game**: clearing or deleting a letter of a replay's name left the name as it was on screen and cut or blanked the name of the saved game in the same slot.
- **Seen or read**: read in the code.
- **Fix**: all three act on the replay's own name (`src/menu/edit_replay_name.rs`, `backspace_name`).

---

## Race messages lost their bytes when they arrived out of order

*Original*: `net::store_received` (0x419677).
*Fixed*: 2026-11-17.

- **What**: a race message from another player is filed in sequence order. One arriving after a newer one moved the newer records up and was then filed with length 0 and none of its bytes. With the inbox full (0x40 messages), the new message was written one record past the end, over the neighbouring fields (the oldest and newest sequence bytes, the time, the copy handed to the race).
- **In game**: in a network race a late message was applied as an empty one (a lost update of the other car); a long stall of the race step could corrupt the inbox's bookkeeping.
- **Seen or read**: read in the code, and found by testing the Rust port against it.
- **Fix**: a message keeps its bytes wherever it is filed; into a full inbox the oldest record goes and the new one is filed in order (`rc2k_core::net::inbox::store`, unit-tested).

---

## The best times file left open, and a short write passed

*Original*: `saves::load_best_times` (0x4AB155), `saves::save_best_times` (0x4AB236).
*Fixed*: 2026-11-17.

- **What**: a `hst.dat` whose head was wrong was not closed after reading; the writes of the best times were never looked at.
- **In game**: nothing visible. A file left open could keep the next save from opening it on some systems; a full disk lost the best times with no word.
- **Seen or read**: read in the code.
- **Fix**: the file is closed on every path; a write that falls short is logged in `rc2k.log` (`src/saves/load_best_times.rs`, `save_best_times.rs`).

---

## A saved game or set-up that could not be written kept its name in the list

*Original*: `menu::edit_savegame_name`, `service::edit_setup_name` (the writers' result was not looked at).
*Fixed*: 2026-11-17.

- **What**: the name typed is put into the slot list before the file is written, and nothing took it out again when the write failed (for example because the `savedata` folders did not exist, which the original never made).
- **In game**: the list showed a saved game or set-up that was not there.
- **Seen or read**: seen by the owner, 2026-10-10.
- **Fix**: the standalone makes the `savedata` folders at start (it already did), and a slot whose file was not written gets the name from before and a line in `rc2k.log`.

---

## The windscreen Pong's bat never moved

*Original*: `effects::move_pong_bat` (0x4B82B0).
*Fixed*: 2026-11-17.

- **What**: the hidden Pong game over the race (cheat "blarby garby", seen from the in-car views) has code that moves the bat up and down by keys, but two jumps right after the bat's place is kept skip all of it.
- **In game**: a ball and a bat are drawn and the ball bounces, but the bat stays where it is: the game cannot be played.
- **Seen or read**: seen in the standalone and read in the disassembly.
- **Fix**: the bat follows player 1's steering (left up, right down; keys, pad or wheel as the race reads them), 32 units a frame within the original's walls (`rc2k_core::effects::pong::Court::move_bat`).

---

## A lobby kept showing after its session was gone

*Original*: `net::lobby_loop`.
*Fixed*: 2026-11-17.

- **What**: when the host's session ended while a joined machine sat in the lobby, the lobby's loop never looked at the link again.
- **In game**: the joiner stayed in a lobby that no longer worked (a frozen list of players) instead of going back to the join screen.
- **Seen or read**: seen in a two-copy trial (a joiner put out by the host's `/kick`, which takes the same path as a host that leaves).
- **Fix**: the lobby ends when the network link has been closed (`net::lobby_loop`); the joiner is back at the join screen.

---

## Filled rectangles drawn nothing on a 32-bit screen

*Original*: `gfx::fill_rect` (0x406869).
*Fixed*: 2026-11-17.

- **What**: the function that fills a rectangle with one colour skipped the fill for a 32-bit surface (its depth test jumped past it), and on an 8-bit one it filled with the byte and then ran on into a second fill of dwords that started below the rectangle and wrote four times its width.
- **In game**: with the screen at 32 bits (every saved set of options of today) the back of a message box was never filled, the installer's copying page never cleared the percentage before writing the next (the numbers piled up until the bar's glyphs covered them), and the tyre change's texts were written over the last ones without the area being cleared first.
- **Seen or read**: seen in the standalone's installer at 32 bits (the percentage), read in the disassembly for the rest.
- **Fix**: the rectangle is filled, clipped, at every depth the game's surfaces have (8, 16 and 32 bits), with the palette's colour (the byte itself on 8 bits) (`rc2k_core::gfx::raster::fill`, `gfx::rect::fill_rect`). The message box's back, the installer's percentage area and the tyre change's panel are cleared as they were meant to be.

---

## The drivers were never shuffled at a championship's start

*Original*: `menu::shuffle_drivers` (0x470AB9).
*Fixed*: 2026-11-17.

- **What**: at the start of an event the four driver tables are meant to be shuffled: two entries chosen at random swap places, once for each driver of the class. The original worked out the two entries' offsets but never added them to the tables' bases, so every swap exchanged the first entry with itself.
- **In game**: the computer drivers' order in the championship was the same at every event; the random numbers were still drawn, so the rest of the event's random numbers were not affected.
- **Seen or read**: read in the disassembly, not seen.
- **Fix**: the two chosen entries swap places as meant (`menu::shuffle_drivers`), with the same two random draws per swap as before: the random state after the shuffle is unchanged, but the order of the computer drivers in a championship now differs from event to event.
## A replay's camera taken over by the recorded change-view key

*Original*: `drive::read_car_controls`.
*Fixed*: 2026-11-17.

- **What**: while a replay played, the change-view presses the driver made while racing were read back from the file and changed the camera again.
- **In game**: a camera chosen in the replay viewer (for example the roadside camera) cut to the bumper, cockpit and chase views at the moments the driver had pressed the key, then came back.
- **Seen or read**: seen (a replay recorded with view changes, played with a roadside camera).
- **Fix**: while a replay plays, the recorded change-view bit is ignored (the lights' bit still acts); the camera stays as the viewer set it.

## A network replay's car could not be changed

*Original*: `race::mark_viewer_items`, `race::viewer_action`, `drive::choose_shown_cars`.
*Fixed*: 2026-11-17.

- **What**: the replay viewer's view item (which car is shown) was greyed for every one-player replay, and a network race's replay is one (the header's players is 1, with a car for each machine). The replay always opened on the first car. A network game still open (the lobby) also made the viewer show the lobby machine's own car whatever the view said.
- **In game**: in a replay of a network race the second car could not be watched, whichever player had saved it.
- **Seen or read**: seen (a replay saved after a two-player network race).
- **Fix**: the item works for a replay with two cars or more and goes round the four views (first car, second car, both, both the other way); a network race's recording opens on the saving machine's own car; the viewer's view wins over the open link.

## The tyre change's title cut at both ends

*Original*: `race::draw_tyre_change`.
*Fixed*: 2026-11-17.

- **What**: the tyre change screen's title is drawn into a strip of its texture 256 pixels wide, centred in a box of 248. The Portuguese title ("MUDANÇA DE PNEUS") is about 276 pixels wide, so its centred drawing began and ended outside the strip.
- **In game**: the first and last letters of the title were half drawn, cut straight at the strip's edges, at every screen size.
- **Seen or read**: seen (the roadside tyre change on a stage after a puncture, in Portuguese).
- **Fix**: the title has a texture of its own (texture 0x3F0, 256 by 256, pixel kind 3), drawn with the font's own letter spacing. A title that does not fit one 256 strip (box of 248) is drawn across two strips of that texture, its left half in the first row and its right half in the second, and shown as two quads side by side centred where the single strip was; a title that fits is one strip as before. The old strip's title row is left empty. The prompt ("PRIMA BOTÃO PARA PARAR" with the button's picture, when the player has one) has strips of its own in the same texture, two for each half of the screen, drawn the same way with the font's own letter spacing and shown centred where the prompt was; the letters are no longer drawn up to 3 pixels closer to make it fit.

## The second car of a network race's replay made no sound

*Original*: `race::set_up_race_sound_objects` (`race::viewer_load_stage`).
*Fixed*: 2026-11-17.

- **What**: a network race's replay is a one-player one by its header (with a car for each machine), so the replay viewer set up the sound of one car only.
- **In game**: with the replay's second car shown (the viewer's view item), the engine and the other car sounds were silent; the first car had them.
- **Seen or read**: seen (a replay saved after a two-player network race); read in the sound set-up.
- **Fix**: a replay whose header names two cars or more but one player gets the sound objects of a two-player race for its first two cars (the second car's kind from the header), as a two-player replay has.

## The players' results page: a cut time, extra rows and a stray sprite

*Original*: `screens::draw_players_results_points`, `screens::draw_players_results`, `screens::draw_players_standings`.
*Fixed*: 2026-11-17.

- **What**: the results page with points drew the third row's time 50 pixels further left than the others (under the car's picture); it drew four rows for two players (the last two from whatever the order table held), and tested its first row's "no time" differently from the rest. The page without points drew an extra picture over the third row's car, taken from a variable another screen had left. The players' standings page also drew four rows for two players.
- **In game**: in a three-player single rally the third row's time started under the car and was cut ("37.49" instead of "01:37.49"); with two players the page showed two more rows of someone else's data; the third row's car had another sprite on it.
- **Seen or read**: the cut time was seen (a three-player single rally); the rest was read in the code.
- **Fix**: every row's time is at the same place, one row is drawn per player (one to four) on both pages and the standings, and the extra picture is gone. The first row's "no time" test keeps its signed comparison, which matches the others for every time the game makes.

## A time trial's top speed drawn over the best time's car

*Original*: `screens::draw_time_trial_records`.
*Fixed*: 2026-11-17.

- **What**: the time trial's copy of the two record rows put the top speed's car picture at the first row's height instead of the second's.
- **In game**: the best time's row showed two car pictures on top of each other and the top speed's row none.
- **Seen or read**: read in the code.
- **Fix**: the car picture goes on the top speed's row, as on the stage panels.

## The fastest line missing from a record page with fewer than ten times

*Original*: `screens::draw_record_times`, `screens::draw_stage_results`.
*Fixed*: 2026-11-17.

- **What**: a best-times list that ended before ten rows jumped past the page's last line. In the stage results page the same jump left the stack astray (the original crashed there; the Rust version ended the game with a refusal).
- **In game**: the "fastest" line at the bottom of a record page was left out when the list was short; a time trial's stage results page for such a list ended the game.
- **Seen or read**: read in the code.
- **Fix**: the rows the list has are drawn and the fastest line is always drawn.

## A spaced-out empty menu text drew what the line buffer held

*Original*: `screens::draw_menu_item_text`.
*Fixed*: 2026-11-17.

- **What**: a menu item with spaced letters and an empty text drew its terminator and went on through the line buffer, past the empty text, up to the next zero byte.
- **In game**: an empty spaced item could show stray characters of an earlier line (none of the shipped menus has one).
- **Seen or read**: read in the code.
- **Fix**: an empty text draws nothing.

## The overall rally results judged a player's row by the stage time

*Original*: `screens::draw_overall_rally_results`.
*Fixed*: 2026-11-17.

- **What**: a player's row on the overall rally results was marked "retired or not finished" by the stage's time where every other row, and the class page, use the rally's.
- **In game**: a player who had no time in the last stage but a rally total (or the other way round) was shown with the wrong text in place of the total.
- **Seen or read**: read in the code.
- **Fix**: every row is judged by the rally time.

## A class with only the fourth player in it had no results page

*Original*: `menu::class_rally_results_pass`, `menu::class_stage_results_pass`, `menu::class_championship_pass`.
*Fixed*: 2026-11-17.

- **What**: the passes that show a results page for each class in which a player took part looked for players 1 to 3 only.
- **In game**: with four players, a class holding only the fourth player's car was skipped in the class results.
- **Seen or read**: read in the code.
- **Fix**: players 1 to 4 count.

## The network lobby's total drawn at the wrong height

*Original*: `net::lobby_frame`.
*Fixed*: 2026-11-17.

- **What**: for a player with no race time the total was drawn at the height the "no time" text left (its row plus the font's offset), not at the row's.
- **In game**: the totals of players still to finish sat a few pixels lower than the others'.
- **Seen or read**: read in the code.
- **Fix**: every total is at its row's height.

## Escape on the sessions list said the session failed

*Original*: `net::start_link`.
*Fixed*: 2026-11-17.

- **What**: leaving the list of sessions with Escape was handled as a failure to join.
- **In game**: the message "session failed" came up after backing out of the list.
- **Seen or read**: read in the code.
- **Fix**: Escape goes back to the menu without a message.

## The upper player's needle sign followed the lower player's

*Original*: `hud::layout_2`.
*Fixed*: 2026-11-17.

- **What**: in the two-player HUD with the players one above the other, both players' needle signs were chosen from the second player's start value (the one-row layout uses the first for the first).
- **In game**: the upper player's needle sign could be the lower player's.
- **Seen or read**: read in the code.
- **Fix**: each player's sign follows that player's own value.

## Right-aligned buttons drawn as two

*Original*: `screens::draw_button`.
*Fixed*: 2026-11-17.

- **What**: a right-aligned plain button wider than two pieces continued as a left-to-right button from its second piece, ending with a second right end; with a width that is a multiple of 32 it also drew a second, centred button.
- **In game**: wide right-aligned buttons had a second end piece and an extra button drawn over or beside them.
- **Seen or read**: read in the code.
- **Fix**: the pieces run right to left to the left end, as the highlighted button's do.

## The menus' tune not restarted after a saved replay

*Original*: `race::viewer_load_stage`, `menu::replay_item_c`.
*Fixed*: 2026-11-17.

- **What**: the replay viewer sets up the race sounds, which stops the music, and nothing started the menus' tune again when a saved replay chosen from the menus was left.
- **In game**: the menus stayed silent after watching a saved replay.
- **Seen or read**: heard (the original, 2026-10-21).
- **Fix**: the menus' tune starts when the viewer returns to the menus.

## The computer driver never tells a wide road from a narrow one (made optional)

*Original*: `drive::drive_computer_car` (0x4628EF .. 0x462903).
*Made optional*: 2026-11-18 (advanced page, "computer cars on wide roads"; off by default).

- **What**: the driver tests whether the road at the car's checkpoint is narrow (its face's two width bytes add up to less than 18) and both outcomes then set the same "narrow" flag, so the wide-road settings (the first speed table, the turn of 0x980 at which a bend is noted, the lookahead for the faster cars) are never used.
- **In game**: the computer's cars drive every road with the narrow road's slower speed table. With the wide way switched on, the cars on wide roads of the stages are about the same speed: in trials a stage's time changed by less than a percent either way, a pack of arcade rivals covered about one percent more of the course in the same time, and no car got stuck more often.
- **Seen or read**: read in the code; the cars' times compared in trials (2026-11-18).
- **Fix**: the flag is set only for a narrow road when the switch is on (`drive::computer_roads`); off, the original's way is kept so that races, replays and held tests are unchanged.

## A replay that filled its index ended the game

*Original*: `race::write_replay_block` and `race::close_replay`.

- **What**: a replay's index has room for 1800 blocks of 100 frames. When the last one is written the replay is closed, and closing pads the block in progress out to 100 frames; but the block just written still counted as 100 frames in progress, so the padding filled a block past the index's last and wrote its place beyond the index.
- **In game**: only on a stage that lasts about an hour of race time (a car stuck and never retired): the original wrote past the index; the conversion refused it and the game ended with an error.
- **Seen or read**: seen 2026-11-18 in a long test run (a computer-driven car stuck for 180,000 ticks).
- **Fix**: the written block is counted as done before the replay is closed, so the replay ends at its 1800 blocks and the race goes on (`race::replay`).
