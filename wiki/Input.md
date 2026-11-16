# Input

*Category: [Game flow](Game-flow)*

This page follows a key press or a turn of a wheel from the operating system to the car, and
force feedback back the other way. It covers the keyboard, controllers through SDL, the
menus' keys, the players' bindings, and the handheld's controls. For the player's view of the
same things (what is supported, how to bind), see [Controllers](Controllers).

| Folder | What is there | Owner |
| --- | --- | --- |
| `src/input/` | The game's side: the key table, the device records, bindings, force-feedback requests, the input thread. | `Input` |
| `src/platform/input.rs`, `src/platform/events.rs` | SDL's side: joysticks, haptics, window and key events. | |
| `src/drive/` | The last stretch: a device's readings into a car's controls. | `Race` |
| `rc2k_core/src/input/` | Rules as plain functions: layouts, bindings, dead zones. | |

All input reaches the game through SDL. There is no direct reading of the keyboard or of
device files.

## Two ways in

Input arrives in two different shapes, and it helps to keep them apart.

**Keys** come as events on the main thread. `platform::events::pump`
(`src/platform/events.rs`) reads SDL's events whenever the game handles its window. A key
going down or up updates the **key table** (one entry per scan code, down or not) and, on the
way down, becomes the **pending key**: the one most recent key press, waiting to be taken.

**Devices** (pads, wheels, joysticks) are polled. The input thread reads each player's device
every 20 ms and leaves the readings in a record.

## Keys in the menus

A menu wants one key at a time, so it takes the pending key: `input::take_key`
(`src/input/take_key.rs`) returns it and clears it in one step. `menu::handle_keys` then acts on
it (see [The front end](The-front-end)).

Typing a name uses the same pending key, turned into a character by the current keyboard
layout (`input::key_to_char`, `src/input/layouts.rs`). The layout comes with the language
(see [Language files](Language-files)) and handles shift, AltGr and dead keys for accents.

The yes/no questions accept the first letter of "yes" and "no" in the game's language
(`src/input/yes_no_key.rs`).

### A pad in the menus

The 1999 menus knew only the keyboard. `src/input/menu_pad.rs` lets any gamepad SDL recognises
drive them, by turning a button into the pending key it stands for:

| Button | In a menu | In a race |
| --- | --- | --- |
| D-pad, or the left stick | Move the highlight (repeats when held) | |
| South (A / Cross) | Enter | |
| East (B / Circle) | Back (Escape) | |
| Start | Enter | The pause and "leave?" question, as Escape |

South and East also answer the race's yes/no question directly, whatever the language. Outside
a race the left stick counts as the D-pad: leaning it past about half its travel presses that
direction, bringing it back within a third lets go (the gap keeps it from chattering near the
line). In a race the sticks and triggers belong to driving only (below).

## Keys in a race

Once per frame, `update_input` (`src/input/keys.rs`) copies the key table and works out which
keys went down *this frame* (with auto-repeat for held keys). The race's own keys read that:
pause, the view change, the screenshot key (`race_keys` in `src/race/frame.rs`).

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
- If SDL can open force feedback for a device, its effects are available.

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

### Bindings

Each player has twelve **controls** (steering left and right, throttle, brake, the gears and
the rest). A control is bound to a keyboard key, and can also be bound to a
device's axis (with a direction) or button. The bindings are part of the player's record in
the options (see [Options files](Options-files)).

`input::read_device` turns a device's raw state into four **readings** using the bindings:

| Reading | Range | From |
| --- | --- | --- |
| Steering | −256 .. 256 | Left and right: a key (full lock), an axis, or a button. |
| Throttle | 0 .. 256 | A key (full), an axis, or a button. |
| Brake | 0 .. 256 | The same. |
| Buttons | bits | The other controls, one bit each; the hat. |

A flag for each of steering, throttle and brake says whether it came from a key or button
("all or nothing") or from an axis. Dead zones at the centre and at the ends, and a sensitivity
level, are applied per player (`src/input/set_up_device.rs`).

The controller pages of the options let the player pick a controller, set sensitivity and dead
zones (`src/menu/controller_items.rs`), **learn** a binding by moving the wanted axis or
pressing the button within three seconds (`src/menu/learn_bindings.rs`,
`src/screens/controller.rs`), and redefine keyboard keys (`src/menu/redefine_keys.rs`).

## From readings to the car

In each simulation step, for each car (see [A race, step by step](A-race-step-by-step)):

| Function | What it does |
| --- | --- |
| `drive::car_input_words` | Takes the four readings of the car's device (`drive::device_axes`). With the "non-linear steering" option, small steering inputs are made smaller still, for finer control near the centre. |
| `drive::record_inputs_if` / `drive::play_inputs_if` | Replay: the readings are recorded, or replaced by recorded ones. |
| `drive::read_car_controls` | Button bits become actions: lights, view change. A computer-driven car gets its controls from `drive::drive_computer_car` instead. |
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
buttons and stick positions to the running game and takes screenshots (`src/remote.rs`). The
project's automated races are driven this way; see [Testing](Testing).

## Where to look

| To change or understand | Start in |
| --- | --- |
| SDL events to keys | `src/platform/events.rs` |
| The pending key | `src/input/take_key.rs` |
| Pressed-this-frame keys | `src/input/keys.rs` |
| Keyboard layouts and typing | `src/input/layouts.rs`, `key_to_char.rs` |
| A pad in the menus | `src/input/menu_pad.rs` |
| How SDL's devices are presented | `src/platform/input.rs`, `src/input/di_objects.rs` |
| The input thread | `src/input/input_thread.rs`, `input_tick.rs` |
| Bindings to readings | `src/input/read_device.rs`, `set_up_device.rs` |
| The controller pages | `src/menu/controller_items.rs`, `learn_bindings.rs`, `redefine_keys.rs` |
| Readings to steering and pedals | `src/drive/steer_inputs.rs`, `pedal_inputs.rs` |
| Force feedback | `src/input/update_car_effects.rs`, `effects.rs`, `device_effects.rs` |

Next: [Threads and shared state](Threads-and-shared-state). See also
[Module-input](Module-input), [Module-platform](Module-platform), [Controllers](Controllers).
