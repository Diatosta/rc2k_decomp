# The debug window

*Category: [Getting started](Getting-started)*

The 1999 game's developers had debug pages inside the game: screens of figures drawn over a
race. The code for them was still in the shipped program, switched off. This project brings
them back as one resizable, semi-transparent window with tabs, drawn over the game with Dear
ImGui, and adds a few pages of its own.

## Opening it

| Keys | Effect |
| --- | --- |
| Ctrl+F11 | shows or hides the window, in a race and in the menus |
| Ctrl+F12 | freezes the race (the simulation stops, the picture is still drawn) or lets it go |

The freeze is also a tick box in the window. While the window has the keyboard (a filter box
is being typed in, for example) the game does not see those keys.

The window's size, place and open tabs are remembered in `var/imgui.ini`.

## Which builds have it

| Build | Debug window |
| --- | --- |
| Windows 64-bit, Linux, macOS | yes |
| Windows 32-bit | no (the window's library has no 32-bit Windows version) |
| Anbernic handheld | no (its picture goes straight to the screen, with no window to draw over) |

## The tabs

| Tab | What it shows |
| --- | --- |
| Display | The developers' main page: the figures about the car and the race (speeds, forces, wheel and surface values, wear), named and grouped, bars shown as bars. A box filters by name. |
| Watch list | The developers' list of watched variables, grouped by car and area. |
| Dumps | Tables the developers' pages dumped in hexadecimal (league tables and the like). |
| Profiler | Where a frame's time goes: the developers' own timed items, and this project's timings of the frame's stages. Costs nothing while the window is closed. |
| GPU | Tick boxes that each remove one kind of GPU work from the next frames (fog, alpha test, texture lookups, blending ...), to see what it costs in frame rate. The picture is wrong while a box is ticked; that is the point. |
| Kstar modes | The developers' special display modes, one section each: the car's faces by group, the camera's record, a car's light places, the road's height around the car, the sun's place on the screen, the course seen from above. |
| Switches | The variables the developers cycled with the numeric keypad, as controls, each with a note on what reads it; also the light switch the developers held the O key for (objects of light kind 2 lit without their strength), which the key no longer does in a race. |
| Rivals | The table of the other drivers that ended the display page. |
| Effects | The effects code's own debug page (rain, snow, spray, windscreen). |
| Dyno | A graph of the current car: drive force by speed for each gear, the drag, the gearbox's shift points and an acceleration test. Measured once per car; the car's state is put back afterwards. |
| Players | Buttons that write each player's record and each car's state to the game's log, as two of the developers' dumps did. |
| Screens | Screens of the game that no menu leads to (the best times of each section of a stage), opened at the next menu frame. |
| Developers | The settings and races of the developers' own front end, a quick menu that came before the real menus: a test course, the finish sequence, the credits, the last replay. |

Next to the tabs is a "Probe points" box: ticked, it marks in the race where the physics asks
for the height of the ground under the car.

## Things to know

- Several tabs **write** the game's variables (Kstar modes, Switches, Developers). What they
  change lasts until the stage is loaded again or the game's own code puts it back. This is a
  developers' tool; a race played with changed switches is not a fair time.
- Some names on the Watch list and Effects tabs are the 1999 developers' own labels, kept as
  they were; where a label ends in `?` the developers were not sure either.
- The code is in `src/debug/` ([Modules](Modules)).
