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
| South button (A on an Xbox pad) | Enter (held, it repeats as a held Enter key does: after about 0.4 s, then every 0.12 s) |
| East button (B) | Escape / back |
| West button (X) | the third answer, where a question has three; where a screen waits for any key (the films, a message), it also ends the wait |
| Left or right shoulder | Tab (repeats when held): the network lobby shows or hides the gaps (in a race they stay the car's controls) |
| Start | Enter in a menu (repeats when held); in a race, the same as Escape (asks whether to leave) |

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
below or above the field, never over it (the network settings and the join password question
tell it where their field is), shows what has been typed on it in a line at the top of its panel
(stars for a password), and has pages of symbols and accented letters. On a system
with its own screen keyboard (a phone, a Steam Deck under Steam) that one is shown instead. A
keyboard typed on closes ours. A hint line at the foot of its panel shows the buttons of the
controller in use for typing, deleting, cancelling and done, as pictures when the controller's
button pictures are on (see "Button pictures"). See [Input](Input).

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
  use at once. The page also has the "H shifter" switch (see below), the force feedback
  strength, "button icons" (see "Button pictures" below) and, for a device with force feedback,
  "engine vibration" (see "Force feedback" below).
- **Axes** (opened from the controller page): a list with a row for each axis the controller has
  (up to six, a gamepad's as the left stick x and y, the right stick x and y and the two
  triggers), each with a live bar of where the axis is now, as the race takes it (dark track;
  a centred axis fills from the middle to the side it leans, a pedal or trigger from the left),
  then "wheel" (yes or no; set from SDL's type at first and changeable with left and right) and,
  for a wheel, "full rotation" (900 degrees at first, steps of 10). The rows are packed: "wheel"
  comes straight after the last axis the controller has, with no empty place between, and the
  highlight skips what is not shown. Return on an axis opens that axis's page.
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
- **Buttons**: the controls on three pages (four with the H shifter switched on), turned by the
  "more controls" item; the last page is the menu controls (below). The axis rows
  come first: steering, accelerate and brake (each learns an axis: steering takes the side it is
  turned to as right, an axis moved the other way is inverted; accelerate and brake take a pedal
  from where it rests or a side of an axis). Then the rows learnt with a button or a hat
  direction: steer left, steer right, accelerate, brake, gear up, gear down, handbrake, camera,
  pause, lights and reset car (the game has no wipers: the original reads no wipers key). An axis row and a button row of the same control both
  count. With the H shifter switched on the buttons page has a page of rows for the
  shifter's gears (see "The H shifter"). The last page, "menus: ...", sets what the menus take from
  this controller: enter, back, up, down, left, right, third answer and Tab. Each row learns a
  button or a hat direction as the others do. A control left unbound keeps the controller's own
  default (a gamepad's South, East and West buttons, cross and shoulders; a wheel's second and
  third buttons and its hat), and its row shows that default; a bound one is taken from the button or hat direction learnt, and
  the default for that control stops working. The menus take the profile of whichever player has
  used the controller (a controller in the menus belongs to no player). The left stick of a
  gamepad, Start leaving a race and the keyboard are not changed. "Undo changes" puts the page's profile back as it was
  when the page was entered.

The texts of the axes pages and of the new rows are not in the language file: the game reads
them from a file of its own for each language, `files/locale/<language>.rc2k.toml`, with English
built in ([Language files](Language-files#the-games-own-texts-rc2ktoml)).

### Button pictures

Where a button or a hat direction is named (the buttons page's values, the on-screen keyboard's
hint line, the race's leave question) the game can show the button's picture instead of its
number or name. The pictures are Kenney's CC0 *Input Prompts* in three families: Xbox
(A, B, X, Y, LB, RB, LT, RT, the sticks' presses, view, menu), PlayStation (cross, circle,
square, triangle, L1 to R3, share / create, options, touchpad) and Steam Deck; each family also
has the left and right sticks pushed up, down, left, right, or along a whole axis. Each controller
has its own "button icons" setting on the controller page, kept in the player's profile for it:
**automatic** (the default), **Xbox**, **PlayStation**, **Steam Deck** or **numbers** (the
old texts; the keyboard always has its own).

- Automatic follows SDL's gamepad type: Xbox 360 and Xbox One pads get the Xbox pictures, PS3,
  PS4 and PS5 pads the PlayStation ones, any other gamepad the Xbox look. A Steam Deck (USB id
  28DE:1205), a Steam controller, and any Valve pad when the program was started by Steam on a
  Deck (`SteamDeck=1`) get the Steam Deck pictures. A joystick or wheel that is not a gamepad
  shows numbers until the player picks a family.
- A gamepad's buttons are bound by position (south, east, west, north, shoulders, back, start,
  the sticks' presses; SDL's own mapping puts the physical button there), so the picture is the
  one on the real button: south is A on an Xbox pad and the cross on a PlayStation one.
- A joystick or wheel set to a family has its button numbers read in DirectInput's order for
  that family: for PlayStation 1 square, 2 cross, 3 circle, 4 triangle, 5 L1, 6 R1, 7 L2, 8 R2,
  9 share, 10 options, 11 L3, 12 R3, 13 PS, 14 touchpad; for Xbox and Steam Deck 1 A, 2 B, 3 X,
  4 Y, 5 LB, 6 RB, 7 view, 8 menu, 9 LS, 10 RS. A button the order has no picture for (or the PS
  button, which the pack lacks) stays a number. A hat direction is a directional-pad picture.
- On the buttons page a gamepad's axes are pictures too: the left or right stick pushed the way
  the binding uses it (the stick's direction picture; the steering, a centred axis, shows the
  stick's left-right picture), a trigger the LT / RT (L2 / R2) picture. The axes of a joystick or
  wheel stay text ("(+x)", "(-rz)"), as the page cannot know what they are.
- The race's leave question shows the pictures of the buttons that answer yes and no inline,
  after the key letters of its own text ("RETIRAR? (s [cross]/n [circle])": the letters are
  found as the first two (a third, "(a/b/c", with the third answer's picture when that answer is
  offered) of the text's last "(a/b" group, so any language works), drawn over the frame in the
  question's font, enlarged with the screen (the screen's height over 600, as the HUD's pictures
  and the timer's digits are) on a dark plate, and centred where the question is. The menus' enter and back of
  the controller are used (what the profile binds, else the controller's default). A text
  without such a group gets the pictures and the words "yes" and "no" on a dark plate under it.
  With numbers or the keyboard the question is the original's. The pictures are as tall as the
  text's capitals and centred on them. The advanced page's "buttons in the quit question" switch
  (`quit_pictures` in `rc2k.toml`) turns them off: the question is then the original's.
  The same switch covers the roadside tyre change's prompt ("press a button to stop"): with a
  controller that has pictures, the picture of the button that stops the gauge follows the
  prompt, as tall as its capitals, the text and picture centred together (any of the eight
  button actions of the race ends the gauge, so the first one the profile binds, gear up first,
  is shown); with the keyboard, numbers or the switch off the prompt is the original's text.

The pictures are blended over the screen as white shapes with soft edges (16 and 32 bits, in
the menus and over a race's frame), scaled down from the atlas's 128 px cells to the size drawn
with an area filter (each pixel the mean of the cell pixels it covers, weighted by how much of
each it covers) so the circles stay smooth. Where the GPU's own picture shows through a pixel the
canvas has not written, the pixel is written white with the picture's alpha in its top byte
(0 and 255 mean opaque, as for every other pixel) and the renderers' overlay blends it with what
is under it, so the menus' pictures have the same soft edges without a plate. The pictures that
go over a race's frame are still drawn on a dark plate (the hints, the question), as a race's
frame is not the canvas's to blend with. In the menus a picture's outline is centred on the
capitals of the text beside it (`button_hints::draw_beside_text`). `tools/glyphs/pack.py` makes the atlases in `assets/glyphs/` from the pack's Double
size; each is a grey PNG of 128 x 128 cells (the alpha) with an index of names and rects.

### The H shifter

The controller page's "H shifter" switch (on or off, as the original's "ACT LABS RS Shifter"
switch was) says the controller has a gear lever with its gears on buttons or hat directions
(a wheel's separate shifter, such as the Fanatec one). With it on, the buttons page lists seven
more rows after the others: gear 1 .. gear 7 and reverse, each learnt like the button rows (a
button or a hat direction; the page asks for it for five seconds). In a race the first of
those, in that order, whose input is held is the gear asked for; taking the lever out of a gate
asks for neutral. With no gate held the gear up and gear down bindings work as without the
shifter, so a wheel's paddles or sequential lever can be used beside it. A
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
player who had chosen it has it back. The force feedback effects the game had set on it (and the strength) are made again on the returned device, so a spring or a rumble running when it left runs again.

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
The strength chosen on the controller set-up page is the device's gain where SDL has one for it;
for a device without (many wheels, the Fanatec ones among them) each effect's force is scaled by
it instead.

A gamepad that has only rumble motors (an Xbox pad, a DualSense and most others) rumbles
instead: the game's engine and surface shakes as the light motor, its knocks, landings and
slow shakes as the heavy one, at the strength chosen on the controller set-up page ("force
feedback"; off, the pad stays still). The springs and dampers that hold a wheel straight have
nothing to shake on a pad and are left out. While the race is paused, or a question is on the
screen, the pad is still. To use it, choose the pad as the player's controller and raise its
force feedback from off.

A gamepad whose triggers have motors of their own (an Xbox One or Series pad, when SDL reports the
motors; SDL's DualSense driver has none to offer) also rumbles its triggers, through SDL's trigger rumble, while a race runs. The
right trigger (the accelerator) shakes when the driven wheels spin faster than the car moves,
and faintly with the engine's load at high revs; the left trigger (the brake) shakes when the
braking wheels lock (the pedal pressed, the wheels slower than the car). Each is scaled by how far
its pedal is pressed; both are still while the car is off the ground and while the game is paused,
and both take the same strength as the motors (the controller page's force feedback; off, the
triggers stay still). The advanced page's "trigger rumble" row (on its second part) switches
them off (`trigger_rumble` in `rc2k.toml`); a pad without trigger motors is not affected. The
shake goes to the left and right trigger motors whatever the player's profile binds to those
triggers. A DualSense's adaptive trigger resistance (SDL offers it only as raw effect packets) is not used.

#### When the triggers stay still

The game's log (`rc2k.log`) says why. When a pad is opened it logs whether SDL reports motors
under its triggers ("is a gamepad with / without trigger rumble"), and the first time levels are
sent to a pad it logs SDL's answer ("SDL took them", or "SDL refused them" with SDL's error text).
What SDL's source (3.4) shows for Linux:

- The kernel-driver path (SDL's evdev joystick driver, used when SDL's own HIDAPI drivers do not
  take the pad) has no trigger rumble at all: the call answers "unsupported". A pad that SDL opens
  this way has the main motors only.
- Trigger motors come from SDL's HIDAPI drivers for Xbox One and Series pads (which report them
  for every Microsoft pad) and for the GIP driver (USB Xbox pads that SDL reaches through libusb).
  The HIDAPI drivers are on by default on Linux, but they need access to the device itself: a pad
  over Bluetooth is read through `/dev/hidraw*`, a pad over USB through libusb (`/dev/bus/usb`).
  Without a permission for those files SDL cannot use its own driver for the pad, and the log says
  "without trigger rumble".
- What can be done: give the user access to the pad's hidraw / USB node with a udev rule (a rule
  that adds `TAG+="uaccess"` for the pad's vendor id `045e`, as Steam's `60-steam-input.rules`
  does, or installing a distribution package of such rules), then replug or reconnect the pad.
  Running the game with `SDL_JOYSTICK_HIDAPI=1` (or `SDL_JOYSTICK_HIDAPI_XBOX_ONE=1`) makes sure
  the drivers are not switched off by the environment; it does not give them access to the pad.
  A kernel driver that takes the pad over (`xpad` over USB) can keep SDL's HIDAPI driver from
  seeing it; the log line shows which of the two ran. These are what the source offers; which one
  applies to a given pad and system is read from the log line.
- The triggers also stay still when the controller page's force feedback of the player's
  controller is off (strength 0), as for the main motors.

### Engine vibration

The original's force feedback has one part that comes from the engine and not from the road: when
the car is nearly still (the first wheel sum at 5 or less, a car at a standstill, or crawling, or
pushed against a wall) its surface effect, a periodic shake, is not the road's but follows the
engine revs, their square, so a wheel buzzes while the engine idles and more as it is revved. The
controller page's "engine vibration" row (on at first: the original's feel; one setting for each
player and controller, kept in the player's profile) switches that part off. With it off:

- the shake of a car nearly still is not asked for (the game's silent row of the effect, no
  magnitude), on a wheel and on a pad's rumble motors alike;
- the accelerator trigger's faint shake with the engine's load (see above) is left out.

The road's part is as it was with the switch on or off: the surface's shake once the car moves, the
knocks, the landings, the springs and dampers that steer the wheel and the changes of speed. The
springs and dampers a car nearly still has are the same either way. The row is greyed out for a
device without force feedback. It changes only what the device is told: the car does the same
whichever way it is set.

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
