# Controllers

*Category: [Getting started](Getting-started)*

The game is driven with the keyboard, a gamepad or a steering wheel. Devices are read through
SDL, so anything SDL knows as a joystick or gamepad can be used.

## Keyboard

The menus use the arrow keys, Enter and Escape. In a race, each player drives with twelve keys
(steering left and right, throttle, brake, and the buttons such as gear changes, handbrake and
view). The keys are set per player in the game's options, on the "redefine keys" pages:
choose an action, then press the key for it. A key already used by another action swaps with
it; a few reserved keys are not accepted.

Escape in a race asks whether to leave it.

## Gamepads

### In the menus

Any gamepad SDL recognises works in the menus on every system, with nothing to set up:

| Button | Does |
| --- | --- |
| D-pad, or the left stick | moves the highlight (held, it repeats) |
| South button (A on an Xbox pad) | Enter |
| East button (B) | Escape / back |
| West button (X) | the third answer, where a question has three |
| Start | Enter in a menu; in a race, the same as Escape (asks whether to leave) |

On a yes / no question the South and East buttons answer yes and no directly, in any
language.

Typing a name (a player's, a saved game's) still needs a keyboard.

### In a race

A pad's menu buttons do not drive the car. For driving, the pad is one of the game's
controllers and is chosen in the game's own controller set-up, as a wheel or joystick was in
1999 (below).

A gamepad SDL knows is presented to the game the way an Xbox pad looked to games of that time:

| The game sees | From the pad |
| --- | --- |
| x, y axes | left stick |
| z axis | both triggers on one axis: left one way, right the other, at rest in the middle |
| two more axes | right stick |
| ten buttons | A, B, X, Y, the shoulders, back, start, the two stick presses |
| a hat | the D-pad |

Any other joystick or wheel is presented as SDL reports it: up to six axes, 32 buttons and four
hats.

## Setting up a controller for driving

In the game's options, each player has a controller page and a buttons page:

- **Controller**: step through "none" and the controllers found; the chosen one is taken into
  use at once.
- **Sensitivity**: four steps.
- **Dead zones**: two of them, which can be raised until little travel is left between them.
- **Buttons and axes**: which axis steers, which is the throttle and the brake, and which
  buttons do the rest.

The choice is kept per player with the options. A player whose controller is not plugged in at
start goes back to "none" (the keyboard).

A controller plugged in while the program runs is added to the list as soon as SDL reports it,
so it can be chosen on the controller pages and driven with at once, without a restart. One
unplugged keeps its place and reads as at rest; plugged back in (the same model), it takes that
place again, so a player who had chosen it has it back. A player's saved controller that was not
plugged in when the program started has gone back to "none": choose it again once it is in.

## Force feedback

The game's force feedback effects (a constant force, springs and dampers, the periodic
shakes) are played through SDL's haptic support, on the steering axis. They work on a device
for which SDL opens a haptic interface, which in practice means wheels and some joysticks.

- An effect the device or SDL cannot do is noted once in `rc2k.log` and skipped; the others
  go on.
- Many gamepads offer SDL only simple rumble and no such effects, so they stay still.

## The handheld

On the Anbernic RG34XXSP the built-in pad is the only input:

- the menus work as in the table above (A enters, B goes back);
- for driving, the built-in pad is chosen as the player's controller in the game's controller
  set-up, like any other pad;
- MENU + START leaves the game (the device has no window to close).

## Good to know

- With two players on one machine, each player has their own keys and their own controller.
- The debug window's keys (Ctrl+F11, Ctrl+F12) are taken before the game sees them
  ([The debug window](The-debug-window)).
- How input travels from a device to a car is described in [Input](Input).
