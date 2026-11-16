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

Escape in a race asks whether to leave it; so does a player's pause binding, a key
or a button of the player's controller.

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

A wheel or another joystick that SDL does not know as a gamepad works the menus too: its hat
moves the highlight like the D-pad, turning the wheel (its first axis, measured from where it
rests; a small turn is enough, and held it repeats) moves left and right, its first button
enters and its second goes back. Each button press in the menus is noted in `rc2k.log` with its
number, so a wheel whose buttons are numbered otherwise can be told. In a race these inputs are
the player's controls only.

On a yes / no question the South and East buttons answer yes and no directly, in any
language.

Typing a name (a player's, a saved game's) still needs a keyboard.

### In a race

A pad's menu buttons do not drive the car. For driving, the pad is one of the game's
controllers and is chosen in the game's own controller set-up, as a wheel or joystick was in
1999 (below).

A gamepad SDL knows is presented to the game the way an Xbox pad looked to games of that time
(the profiles see it differently: its left stick, right stick and each trigger as an axis of
their own, each trigger from not pressed to pressed like a pedal):

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

Learning a button, a hat direction or an axis (the page asks for the control for five seconds)
takes what moves from where the controller rests when the learning starts, so a pedal that rests
at the end of its travel, a clutch for one, is not taken for the axis you moved. A button held
when the learning starts (the one that opened it) counts only once it has been let go and
pressed again. When nothing comes in five seconds, or space is pressed, the control stays as it
was.

### Profiles

Each player keeps a **profile for every controller** they have used: what its axes, buttons and
hat directions do, so changing from a pad to a wheel and back needs no new set-up. A profile is
made the first time a player uses a controller:

- a gamepad: the left stick steers, the right trigger accelerates, the left trigger brakes, the
  D-pad steers too, and the buttons A, B, X, Y are gear up, gear down, handbrake and camera;
- a wheel (SDL's type says so): the first axis steers; the pedals are learnt;
- any other joystick: the first axis steers, the second accelerates up and brakes down, the
  buttons as above.

Steering, accelerating and braking can be bound to an **axis**: the whole of a centred axis
(steering), a pedal (from the end it rests at), or one side of a centred axis. Every control can
be bound to a **button** or a hat direction instead; a button gives all of the control at once.
The controller's old axis modes, the two dead zones of the controller page and the DirectInput
properties no longer have an effect on a controller; each axis has its own settings in the
profile instead:

| Setting | Meaning |
| --- | --- |
| dead zone | Per mille of the travel from the rest that reads nothing. |
| range | Per mille of the travel that gives all of it: 1000 is all of it, 300 gives full lock at 30 % of the way (a wheel's rotation is then 30 % of its full rotation, 270 degrees for 900). |
| sensitivity | -100 to 100: 0 is linear; above 0 the axis is softer near its rest and sharper at the end, below 0 the other way round. |
| invert | The axis the other way round. |

A profile has `wheel` (true for a wheel) and `full_rotation_degrees` (900 unless changed), for
showing a wheel's range in degrees. Profiles are in the options file
([Options files](Options-files)); the controller pages show the controls' bindings from them
and "cancel changes" puts the page's profile back as it was when the page was entered.

The choice is kept per player with the options. A player whose controller is not plugged in at
start goes back to "none" (the keyboard).

A controller plugged in while the program runs is added to the list as soon as SDL reports it,
so it can be chosen on the controller pages and driven with at once, without a restart. One
unplugged keeps its place and reads as at rest; plugged back in, it takes that place again, so a
player who had chosen it has it back.

The program keeps each player's choice by which controller it is, not by its place in the list:
its name and, when it reports one, its serial number (a DualSense gives its Bluetooth address).
Two pads of the same model are told apart by their serial numbers, whatever order they are
plugged in; pads that report none are told apart by order (the first and the second of that
name). A player whose controller is not plugged in drives with the keyboard meanwhile and keeps
the choice: when the controller is plugged in, at start or later, the player has it back. A
player whose pad is away is never given another pad of the same model, which may be another
player's.

## Force feedback

The game's force feedback effects (a constant force, springs and dampers, the periodic
shakes) are played through SDL's haptic support, on the steering axis, on a device for which
SDL opens a haptic interface with such effects: in practice wheels and some joysticks. An
effect the device or SDL cannot do is noted once in `rc2k.log` and skipped; the others go on.

A gamepad that has only rumble motors (an Xbox pad, a DualSense and most others) rumbles
instead: the game's engine and surface shakes as the light motor, its knocks, landings and
slow shakes as the heavy one, at the strength chosen on the controller set-up page ("force
feedback"; off, the pad stays still). The springs and dampers that hold a wheel straight have
nothing to shake on a pad and are left out. While the race is paused, or a question is on the
screen, the pad is still. To use it, choose the pad as the player's controller and raise its
force feedback from off.

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
