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
