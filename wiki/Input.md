# Input

*Category: [Game flow](Game-flow)*

This page follows a key press or a turn of a wheel from the operating system to the car, and
force feedback back the other way. It covers the keyboard, controllers through SDL, the
menus' keys, the players' bindings, and the handheld's controls. For the player's view of the
same things (what is supported, how to bind), see [Controllers](Controllers).

| Folder | What is there | Owner |
| --- | --- | --- |
| `src/input/` | The game's side: the key table, the device records, bindings, force-feedback requests, the input thread. | `Input` |
| `src/platform/input.rs`, `src/shell/events.rs` | SDL's side: joysticks, haptics, window and key events. | |
| `src/drive/` | The last stretch: a device's readings into a car's controls. | `Race` |
| `rc2k_core/src/input/` | Rules as plain functions: layouts, bindings, dead zones. | |

All input reaches the game through SDL. There is no direct reading of the keyboard or of
device files.

## Two ways in

Input arrives in two different shapes, and it helps to keep them apart.

**Keys** come as events on the main thread. `shell::events::pump`
(`src/shell/events.rs`) reads SDL's events whenever the game handles its window. A key
going down or up updates the **key table** (one entry per scan code, down or not) and, on the
way down, becomes the **pending key**: the one most recent key press, waiting to be taken.

**Devices** (pads, wheels, joysticks) are polled. The input thread reads each player's device
every 20 ms and leaves the readings in a record.

## Keys in the menus

A menu wants one key at a time, so it takes the pending key: `input::take_key`
(`src/input/take_key.rs`) returns it and clears it in one step. `menu::handle_keys` then acts on
it (see [The front end](The-front-end)).

### Text entry

While a text field is being edited (a player's name, a saved game's, replay's or set-up's name,
the network settings and the lobby's chat line, the installer's folder) SDL's text input is on
(`input::typed_text`, `src/input/typed_text.rs`). The characters SDL reports go into a queue as
Windows-1252 bytes (a character the code page lacks is dropped) and each field takes them from
it, keeping its own rules: its length, its width in its font, the cheats on the fourth player's
name. Escape, Enter, Delete and Backspace still come as keys; the scan codes of printable keys
are not read while text input is on, so no letter comes twice; a printable key that arrives as a
key alone (before text input has started, a few frames after the field opens) types nothing.
Any keyboard layout or input method SDL knows therefore types, accented letters included (the
characters Windows-1252 has: e, c, a with their accents; others are dropped). The game's own
layout tables (`input::key_to_char`, `src/input/layouts.rs`, with the language, see
[Language files](Language-files)) no longer type into fields; they serve the yes/no questions
and the key names. While a field is edited SDL is told where it is (`SDL_SetTextInputArea`: a band
across the window at the field's height, the middle where the field does not say) so an input
method's candidate list and a screen keyboard keep clear of it. Text input stays on while a field
asked for its characters in the last half second and goes off after the editing.

A field entered from a controller shows an on-screen keyboard (see below) unless the system has
a screen keyboard of its own (`SDL_HasScreenKeyboardSupport`: phones, a Steam Deck under Steam),
which SDL then shows with the text input.

The yes/no questions accept the first letter of "yes" and "no" in the game's language
(`src/input/yes_no_key.rs`).

### A pad in the menus

When a text field is entered with a controller's confirm (the last confirm in the menus came
from a pad, wheel or other controller, not from the keyboard), a keyboard is drawn on a dark
panel below the field, or above it when there is no room below, so the field stays in sight
(`src/input/osk_draw.rs`; the grid, its pages and its moves are `rc2k_core::input::osk`). Each
key is a tile, the key under the cursor on a red one. Three pages: digits and letters, symbols
(`! ? * # $ % & ( ) /` and more), and the accented letters with a few signs (`¡ ¿ « » © £`);
the command row has the case key, the two page keys, space, delete and ok. The cross or stick
moves over it, South (A) types the key under it, West (X) deletes, East (B) cancels the field
(Escape: the text from before comes back), Start is done (Enter), Back (Select) cancels too.
Typing on a physical keyboard closes the grid.

The 1999 menus knew only the keyboard. `src/input/menu_pad.rs` lets any gamepad SDL recognises
drive them, by turning a button into the pending key it stands for:

| Button | In a menu | In a race |
| --- | --- | --- |
| D-pad, or the left stick | Move the highlight (repeats when held) | |
| South (A / Cross) | Enter | |
| East (B / Circle) | Back (Escape) | |
| Start | Enter | The pause and "leave?" question, as Escape |
| Left or right shoulder | Tab (the network lobby shows or hides the gaps) | the car's own control |

South and East also answer the race's yes/no question directly, whatever the language. Outside
a race the left stick counts as the D-pad: leaning it past about half its travel presses that
direction, bringing it back within a third lets go (the gap keeps it from chattering near the
line). In a race the sticks and triggers belong to driving only (below). A joystick that is not
a gamepad (a wheel) sends SDL's joystick events instead: outside a race its hat is the cross (its
axes move nothing in the menus), its buttons 2 and 3 are Enter and Back (`joystick_in_menus` in `src/shell/events.rs`). In a race only those two
buttons go on to the menus' side, so they answer the "leave?" question as a pad's buttons do.

The pictures of a controller's buttons (`rc2k_core::input::icons`: the style and family, the
number-to-picture orders; `src/gfx/button_pictures.rs` draws them, `button_hints.rs` the hint
lines of the on-screen keyboard and the race's leave question) are described in
[Controllers](Controllers#button-pictures). A gamepad's family comes from SDL's type and USB ids
(`platform::ffi::sdl_input::pad_family`, kept in `DeviceInfo.icon_family`).

Each controller's profile can bind the menu controls (enter, back, up, down, left, right, third
answer, Tab) to buttons or hat directions of its own, on the last page of the buttons page
([Controllers](Controllers)). `bound_menu_controls` in `src/shell/events.rs` looks the
controller up by name among the players' profiles (`input::profiles::menu_map_of`, the
bindings as `rc2k_core::input::profile::MenuMap`); a bound control is pressed from its button
or hat direction (for a gamepad, its gamepad buttons in the profiles' numbering, `PAD_BUTTONS`,
the cross as hat 0) as the pad's button that stands for it,
and the default for that control (a gamepad's South, a wheel's button 2, the cross' direction)
is dropped for that controller. Start leaving a race and the left stick stay as they are.

The pad's West button also gives a key the menus ignore, so the films and the messages that wait for any key end on it. Places that end on "any key" look at the pad too, through a count of presses
(`Input::pad_pressed_since`): the pad's South, East or Start (a wheel's second and third buttons) end
the credits race of the attract mode, skip the drive after the finish line, stop a replay played
from the film loop, and end the pause that follows a window losing the focus. In the network
race's wait for the other players, East (Back) leaves as Escape does and so does Start.
In the replay viewer a race is loaded but its list is a menu, so the stick, the wheel and the
shoulders work it as they work the menus (`driving` in `src/shell/events.rs`).

Race keys that have no pad binding: F1 next view, F2 split screen layout (with three or four
players it steps through their layouts),  F3 and F4 volume down
and up (with Left Ctrl held, the music's), F5 the shared-view flag, F6 .. F12 the HUD and view
option flags (the screenshot key too, with screenshots on), and Left Ctrl itself, which turns
the view keys to the second player's view. None has a default pad button.

## Three and four players

In a single rally for three or four players on one screen (`--players 3` / `4`, or the
"3 players on one screen" / "4 players on one screen" entries of the single rally's players menu),
each player's car reads the device of that player's
record: the third and fourth players have devices of their own after the first two (the input
thread keeps four records, `src/input/store.rs`), set up and read like the others when the race
starts and given back when it ends. A controller that an earlier player has already chosen is
dropped to the keyboard for the later one (a pad cannot drive two cars), as the second player's
is when it is the first's. The keys come from each player's record, so players who have not
redefined theirs share the first player's defaults: three or four players on the keyboard need
their keys redefined on the player pages, or a controller each. Each player's pause binding asks the
"leave?" question as the first two do (`drive::race_loop`), and the third and fourth players' force
feedback is fed from their own cars and sent to their own devices after the first two's
(`input::update_effects`). The co-driver's sounds follow the first two players only.

## Keys in a race

Once per frame, `update_input` (`src/input/keys.rs`) copies the key table and works out which
keys went down *this frame* (with auto-repeat for held keys). The race's own keys read that:
the view change, the screenshot key (`race_keys` in `src/race/frame.rs`). The pause binding
and Escape ask the "leave?" question (`drive::race_loop`).

Driving does not read keys this way. It reads the players' devices.

## Devices

### What SDL provides

`src/platform/input.rs` lists SDL's joysticks in the order they were first seen. A device that
is unplugged keeps its place in the list and reads as centred, so the players' controller
numbers do not shift; the same model plugged back in takes that place again. When SDL reports a
joystick added or removed, the input thread lists the new ones into the game's controllers'
table (`list_new_joysticks` in `src/input/list_joysticks.rs`), so a pad plugged in after the
start is offered on the controller pages and can be set up for a race.

- A **gamepad** SDL knows is presented with a fixed layout: the left stick as two axes, both
  triggers together on one axis (resting in the middle), the right stick as two more, ten
  buttons, and the D-pad as a hat.
- Any **other joystick or wheel** is presented as SDL has it: up to six axes, 32 buttons and
  four hats.
- If SDL can open force feedback for a device, its effects are available; a pad with only
  rumble motors reports force feedback too, and its effects are played on the motors (below).

The original talked to DirectInput. `src/input/di_objects.rs` keeps that shape (list devices,
open one, read its state, create and update effects) as plain functions on top of the list
above, so the game-side code above it did not have to change.

### The input thread

`input::input_thread` (`src/input/input_thread.rs`) runs every 20 ms until the game quits.
Each turn (`src/input/input_tick.rs`):

1. the key clock is moved on (the menus use it for timing);
2. a pending command is carried out, if there is one: set a device up for a player, or let it
   go (`src/input/device_command.rs`);
3. otherwise each player's device is read (`input::read_device`) and its force-feedback
   changes are passed on (`input::device_effects`).

While the window is not the active one the thread turns only once a second; it looks for the
window's becoming active every 10 ms, so the controllers answer at once on coming back. The
original slept the whole second, and also slept a second at every race's start before it began
the force-feedback effects (time for DirectInput to take the device); that sleep is gone.

There are two device records, for player 1 and player 2 (`src/input/devices.rs`). One player
can also use two devices at once, a wheel and separate pedals for instance; the second
device's readings are then added to the first's.

### Bindings and profiles

Each player has twelve **controls** (steering left and right, throttle, brake, the gears and
the rest). A control is bound to a keyboard key (`key_scan_codes` in the options), and the
controller binds it through a **profile**: one for each player and each controller the player
has used (`rc2k_core::input::profile`, kept by `input::profiles`, in the options file under the
player). A profile binds steering, throttle and brake to an axis (centred, a pedal, or one side
of an axis) and every control to a button or a hat direction, and shapes each axis (dead zone,
range, sensitivity curve, invert; see [Controllers](Controllers)). The record's old binding
words stay in the record, as saved games carry them, but nothing reads them.

In a race the input thread reads the controller as SDL gives it (`input::di_objects::
profile_reading`: axes -32768 .. 32767, buttons, hats; a gamepad's triggers as pedals) and
`input::read_device` turns it into four **readings** with `profile::readings`. The profile used
is the one of the player whose controls the car took (`Input::race_players`), copied when the
race sets the device up. Keys always win, and a player with no controller reads from keys alone.

| Reading | Range | From |
| --- | --- | --- |
| Steering | −256 .. 256 | Left and right: a key or a button (full lock), else the steering axis. |
| Throttle | 0 .. 256 | A key or button (full), else the accelerate axis. |
| Brake | 0 .. 256 | The same. |
| Buttons | bits | The other controls, one bit each; the hat. |

A flag for each of steering, throttle and brake says whether it came from a key or button
("all or nothing") or from an axis. Replays and network games carry the car's inputs, not the
devices, so they do not change.

The controller pages of the options let the player pick a controller, edit each axis's settings
(the axes list and axis pages: `src/menu/player_axes_loop.rs`, `src/menu/player_axis_loop.rs`,
`src/screens/axes_page.rs`,
`src/menu/axes_items.rs`, with the rules in `rc2k_core::input::axes_page`), **learn** a binding by
moving the wanted axis or pressing the button within five seconds
(`src/menu/learn_bindings.rs`, `src/screens/controller.rs`, `profile::detect` and
`profile::learn`: only what moves from the controller's rest counts; the buttons page's axis rows
take axes, its other rows buttons and hat directions), and redefine keyboard keys
(`src/menu/redefine_keys.rs`). The pages' own texts (`rc2k_core::strings::own`, string ids from
0x2000, English and Portuguese) are chosen by the language file's name.

## From readings to the car

In each simulation step, for each car (see [A race, step by step](A-race-step-by-step)):

| Function | What it does |
| --- | --- |
| `drive::car_input_words` | Takes the four readings of the car's device (`drive::device_axes`). With the "non-linear steering" option, small steering inputs are made smaller still, for finer control near the centre. |
| `drive::record_inputs_if` / `drive::play_inputs_if` | Replay: the readings are recorded, or replaced by recorded ones. |
| `drive::read_car_controls` | Button bits become actions: lights, view change (not while a replay plays). A computer-driven car gets its controls from `drive::drive_computer_car` instead. |
| `drive::steer_inputs` | Key steering moves the wheels towards full lock at a fixed rate and lets them return to centre faster. Axis steering sets the angle directly. Either way the steering sensitivity set-up value scales it. |
| `drive::pedal_inputs` | Key pedals ramp up and down; axis pedals are taken as they are. |
| `physics::gear_input_rules` | Which gear requests count. |

So a keyboard player and a wheel player go through the same code from `steer_inputs` on; the
only difference is the ramping.

In a network game the other players' cars get their readings from the network instead of a
device; see [Network internals](Network-internals).

## Force feedback

Force feedback runs the other way, in three stages:

1. **The race decides.** Each frame, from the race's sound turn
   (`input::step_effects_clock`), `input::update_car_effects`
   (`src/input/update_car_effects.rs`) looks at a player's car: its speed, the surface under
   the wheels, a knock and its direction, a landing, the change of speed. From those it files
   up to seven requests: a steering force, an engine rumble, a surface effect, a knock, and so
   on.
2. **A request is a small note** on one of the device's six effects: start, set or stop, with
   two numbers (`input::set_effect`, `src/input/effects.rs`). Filing one does not touch the
   device.
3. **The input thread delivers.** On its next turn, `input::device_effects` hands each waiting
   request to the platform, which translates it into an SDL haptic effect: a constant force, a
   ramp, or a periodic effect (`src/platform/input.rs`).

A pad with rumble motors and no such effects keeps the effects instead, and on each reading
the input thread turns those running into the two motors' levels (`rc2k_core::input::rumble`):
a steady force, a ramp or a slow shake (a period of 100 ms or more) on the heavy motor, a fast
shake on the light one, each as strong as its envelope says at that moment, added up and held
to full, at the device's gain; conditions (springs, dampers) are left out. In a race the game
sets its effects every frame, so a quarter second without a setting (a pause, a question) stills
the motors.

Effects are started and stopped around a race (`src/input/begin_effects.rs`,
`end_effects.rs`). The strength is a per-player option.

## The handheld

On the Anbernic RG34XXSP the built-in controls are a gamepad to SDL, so everything above
applies: the D-pad, the left stick and the face buttons work the menus through `menu_pad`, and driving uses the
pad as a device with the player's bindings. SDL runs without a display there (the picture goes
straight to the screen's framebuffer, `src/platform/handheld.rs`), but still provides the pad
and the sound.

MENU + START leaves the game. Setting the device up is described in
[Platforms](Platforms).

## Testing input

A build made with the `remote` feature can be driven by `tools/remote.py`: it sends keys, pad
buttons and stick positions to the running game, types text (`remote.py text Gonçalves`, as SDL's text input gives it) and takes screenshots (`src/remote/driver.rs`). The
project's automated races are driven this way; see [Testing](Testing).

## Where to look

| To change or understand | Start in |
| --- | --- |
| SDL events to keys | `src/shell/events.rs` |
| The pending key | `src/input/take_key.rs` |
| Pressed-this-frame keys | `src/input/keys.rs` |
| Keyboard layouts and typing | `src/input/layouts.rs`, `key_to_char.rs` |
| Typed text and the on-screen keyboard | `src/input/typed_text.rs`, `osk_draw.rs`, `rc2k_core/src/input/osk.rs` |
| A pad in the menus | `src/input/menu_pad.rs` |
| How SDL's devices are presented | `src/platform/input.rs`, `src/input/di_objects.rs` |
| The input thread | `src/input/input_thread.rs`, `input_tick.rs` |
| Bindings to readings | `src/input/read_device.rs`, `set_up_device.rs` |
| The controller pages | `src/menu/controller_items.rs`, `axes_items.rs`, `learn_bindings.rs`, `redefine_keys.rs`, `src/screens/axes_page.rs` |
| Readings to steering and pedals | `src/drive/steer_inputs.rs`, `pedal_inputs.rs` |
| Force feedback | `src/input/update_car_effects.rs`, `effects.rs`, `device_effects.rs` |

Next: [Threads and shared state](Threads-and-shared-state). See also
[Module-input](Module-input), [Module-platform](Module-platform), [Controllers](Controllers).
