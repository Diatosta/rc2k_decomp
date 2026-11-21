# Module: input

*Category: [Module reference](Modules)*

Keyboard, controllers and force feedback, as the game sees them. It keeps the table of held
keys and the one pending key the menus read, lists and reads the game controllers, and runs
the force-feedback effects from what the car is doing. The devices themselves come from SDL
through `platform::input`.

## State

The owner is `Input` (`src/input/owner.rs`); its parts are in `src/input/store.rs`
(devices, readings, effect records) and `src/input/layouts.rs` (keyboard layouts).

## Main entry points

| Item | File | What it does |
| --- | --- | --- |
| `input_thread` | `src/input/input_thread.rs` | The input thread's routine. |
| `input_tick` | `src/input/input_tick.rs` | One turn of it: reads the devices, steps the force feedback, counts the key clock. |
| `take_key` | `src/input/take_key.rs` | Gives the menus the pending key, if any. |
| `copy_keys`, `repeat_keys` | `src/input/keys.rs` | The held-key table and key repeat. |
| `MenuPad` | `src/input/menu_pad.rs` | A gamepad in the menus: buttons become the keys they stand for. |
| `list_joysticks` | `src/input/list_joysticks.rs` | Finds the controllers present. |
| `set_up_device`, `acquire_device` | files of the same names | Prepares a controller for use. |
| `read_device`, `poll_device` | files of the same names | Reads a controller's axes and buttons. |
| `update_car_effects`, `update_effects` | files of the same names | Force feedback from the car's state. |
| `append_key_name` | `src/input/append_key_name.rs` | A key's name as text, for the controls screen. |
| `Layouts` | `src/input/layouts.rs` | The keyboard layout in use. |

## How it is laid out

One function per file. Groups: `*_device*` (controllers), `*effect*` and `*force*` (force
feedback), `*_found.rs` (what listing the devices reports), `keys.rs`, `key_to_char.rs` and
`yes_no_*` (keyboard).

## Calls and callers

It reads devices through [`platform`](Module-platform). [`menu`](Module-menu),
[`service`](Module-service) and [`net`](Module-net) take keys from it; [`drive`](Module-drive)
and [`race`](Module-race) read the held keys and the axes for the cars.

## Related pages

[Input](Input), [Controllers](Controllers), [Threads and shared state](Threads-and-shared-state).
