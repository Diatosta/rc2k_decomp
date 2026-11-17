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
| Left or right shoulder | Tab: the network lobby shows or hides the gaps (in a race they stay the car's controls) |
| Start | Enter in a menu; in a race, the same as Escape (asks whether to leave) |

A wheel or another joystick that SDL does not know as a gamepad works the menus too: its hat
moves the highlight like the D-pad (turning the wheel or pressing a pedal moves nothing in the
menus), its second button enters and its third goes back (cross and circle on a wheel with a
PlayStation pad's faces). Each button press in the menus is noted in `rc2k.log` with its
number, so a wheel whose buttons are numbered otherwise can be told. In a race the axes and hat
are the player's controls only; the two buttons also answer the race's "leave?" question (the
second leaves, the third stays), as a pad's do.

On a yes / no question the South and East buttons answer yes and no directly, in any
language.

Typing a name (a player's, a saved game's, the network settings and the chat) with a pad shows
an on-screen keyboard when the field is entered with the pad's confirm: the cross or stick moves
over the keys, A types one, X deletes, B cancels the field, Start is done. The keyboard sits
below or above the field, never over it, and has pages of symbols and accented letters. On a system
with its own screen keyboard (a phone, a Steam Deck under Steam) that one is shown instead. A
keyboard typed on closes ours. See [Input](Input).

A pad's South, East or Start (a wheel's second and third buttons) also end what ends on any key: the
credits race of the attract mode, the drive after the finish line, a replay played from the film
loop and the pause after the window lost the focus. While the network race waits for the other
players, East or Start leaves, as Escape does. The replay viewer works with the D-pad, the stick
or the wheel like the menus.

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

In the game's options, each player has a controller page, an axes list with a page for each axis, and a buttons page:

- **Controller**: step through "none" and the controllers found; the chosen one is taken into
  use at once. The page also has the "H shifter" switch (see below) and the force feedback
  strength.
- **Axes** (opened from the controller page): a list with a row for each axis the controller has
  (up to six, a gamepad's as the left stick x and y, the right stick x and y and the two
  triggers), each with a live bar of where the axis is now, as the race takes it (dark track;
  a centred axis fills from the middle to the side it leans, a pedal or trigger from the left),
  then "wheel" (yes or no; set from SDL's type at first and changeable with left and right) and,
  for a wheel, "full rotation" (900 degrees at first, steps of 10). Return on an axis opens that
  axis's page.
- **Axis** (one axis's page, opened from the list): rows "axis" (left and right pick another
  axis of the controller), "dead zone" and "range" (per cent, with the game's sliders; a wheel's
  range reads as per cent and degrees of its full rotation, "50% (450 degrees)", without a
  slider), "response curve" (a figure from -100 to 100 in steps of 10 and what it does: linear at
  0, gentle above 0, finer near the rest; quick below 0, quicker near the rest; not the player
  page's steering sensitivity, which is the car's set-up and acts on every controller and the
  keyboard alike), "invert" (yes or
  no) and a live bar of where the axis is, shaped by the settings above so a change shows at
  once. Left and right change the highlighted row (Return flips the invert). Every change goes
  to the player's profile for the controller at once. Back returns to the list.
- **Buttons**: the controls on two pages (three with the H shifter switched on), turned by the
  "more controls" item. The axis rows
  come first: steering, accelerate and brake (each learns an axis: steering takes the side it is
  turned to as right, an axis moved the other way is inverted; accelerate and brake take a pedal
  from where it rests or a side of an axis). Then the rows learnt with a button or a hat
  direction: steer left, steer right, accelerate, brake, gear up, gear down, handbrake, camera,
  pause, lights and reset car (the game has no wipers: the original reads no wipers key). An axis row and a button row of the same control both
  count. With the H shifter switched on the buttons page has a third page of rows for the
  shifter's gears (see "The H shifter"). "Undo changes" puts the page's profile back as it was
  when the page was entered.

The texts of the axes pages and of the new rows are not in the language file: the game reads
them from a file of its own for each language, `files/locale/<language>.rc2k.toml`, with English
built in ([Language files](Language-files#the-games-own-texts-rc2ktoml)).

### The H shifter

The controller page's "H shifter" switch (on or off, as the original's "ACT LABS RS Shifter"
switch was) says the controller has a gear lever with its gears on buttons or hat directions
(a wheel's separate shifter, such as the Fanatec one). With it on, the buttons page lists seven
more rows after the others: gear 1 .. gear 7 and reverse, each learnt like the button rows (a
button or a hat direction; the page asks for it for five seconds). In a race the first of
those, in that order, whose input is held is the gear asked for; none held is neutral. A
profile with no gear bound at all reads the shifter the way the switch did before the gears
could be bound, so an old set-up still works: buttons 6, 7, 1 and 2 are gears 1 .. 4, the hat
up, left and down gears 5 .. 7, and the hat right reverse. With the switch off the rows are
hidden and the hat and buttons mean what they are bound to.

The switch belongs to the controller, not to the player: it is kept in the player's profile for
that controller (`shifter`, see [Options files](Options-files)), so a pad used on its own does not
inherit the wheel's switch. With no controller chosen the row is not shown. The player's old
"controller mode" value is kept in the saved file as it was but no longer read.

Learning a button, a hat direction or an axis (the page asks for the control for five seconds)
takes what moves from where the controller rests when the learning starts, so a pedal that rests
at the end of its travel, a clutch for one, is not taken for the axis you moved. A button held
when the learning starts (the one that opened it) counts only once it has been let go and
pressed again. When nothing comes in five seconds, or space is pressed, or a gamepad's Back
(Select) is held for a second, the control stays as it was; Back let go sooner is learnt like
another button.

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
| sensitivity | The axis page's "response curve", -100 to 100: 0 is linear; above 0 the axis is softer near its rest and sharper at the end, below 0 the other way round. |
| invert | The axis the other way round. |

A profile has `wheel` (true for a wheel; the axes list switches it) and `full_rotation_degrees`
(900 unless changed), for showing a wheel's range in degrees. Profiles are in the options file
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
