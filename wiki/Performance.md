# Performance

*Category: [About](About)*

Measured speed, and how it was measured. The figures are from the project's own runs and are
rounded; they say what to expect, not what every machine will do.

## How the game spends its time

The simulation always runs at 100 steps a second ([A race, step by step](A-race-step-by-step)).
How many pictures are drawn in that time is the frame rate, and it is what these figures are
about. A slow machine draws fewer pictures; the race itself does not slow down or change.

A frame's work is in two parts: the game's own 3D code, which turns the scene into finished
triangles on the processor, and the renderer, which draws them
([Drawing a frame](Drawing-a-frame)). With a GPU renderer the second part costs the processor
little; with the software rasteriser it is the larger part and is spread over the processor's
cores.

## Desktop, compared with the original

The same two replays were played by the original program and by this one, three runs of a
minute each, at 1024 × 768 on a 60 Hz display. The original and the Windows build ran under
Proton on Linux; the Linux build ran natively. Processor use is in per cent of one core.

| Program | Frames a second | Processor | Memory |
| --- | --- | --- | --- |
| The original | 60 | about 110 % | about 250 MB |
| This program, Linux, GPU renderer | 62 | about 17 % | about 185 MB |
| This program, Windows, GPU renderer | 62 | about 37 % | about 210 MB |
| This program, Linux, software rasteriser | 62 | about 310 % | about 210 MB |
| This program, Windows, software rasteriser | 62 | about 435 % | about 220 MB |

Every build held the frame rate. On the GPU renderer this program used between a seventh
(Linux) and a third (Windows under Proton) of the original's processor time. The software
rasteriser used three to four cores at this size.

These figures are from before the later speed work, which mostly served the handheld; they
were not taken again afterwards. The 62 frames a second were the program's own limit then (a
wait of 16 ms a frame); it now follows the display's refresh, 60 a second on this display. The script is `tools/vanilla_bench.py`.

## The handheld

The Anbernic RG34XXSP has four Cortex-A53 cores and a Mali-G31 GPU. With the OpenGL ES
renderer, on Knulli, the computer driver driving:

| What | Measured |
| --- | --- |
| A night stage, dry | about 55 frames a second |
| A stage in rain | about 46 frames a second |
| The menus | 47 to 59 frames a second |
| Loading a stage | about 2 seconds |
| From the program's start to the main menu | about 4 seconds |
| Back to the menus after a race | about 2.7 seconds |

Runs of the same stage differ by about a frame a second. The GPU is the limit on this
device: its frames are shown at the panel's refresh (just under 60 a second), a frame it does
not finish in time waits for the next refresh, and the see-through layers (smoke, spray,
trees' edges, the windscreen's effects) take most of its time. With the 3D left out of the
GPU's work, the rest of the program holds the refresh rate.

The race's 2D (the dials, the clock, the map) is sent to the GPU as the rows of a picture
that changed. Those rows go into one of four textures in turn: written into a single one, each
upload waited for the GPU to finish the frame before, which still read it, and the dry stage
ran at about 47 frames a second, the rain at about 43.

Most of what the game draws goes to the GPU with blending and an alpha test on, whether or
not a pixel can be see-through. A batch whose texture and corner colours are fully opaque is
drawn with the blend off and by a copy of the shader with no alpha test: no pixel changes,
and the GPU can reject hidden pixels before it shades them (about a frame and a half a
second more on the dry stage; `RC2K_FAST_PATH=0` turns it off).

## Start-up

The 1999 program paused for about a second in several places while starting, and timed the
processor's clock. This program does neither: on a desktop it reaches the first film or menu
in under half a second, where it took about four and a half with the pauses kept.

## Choices that affect speed

| Choice | Effect |
| --- | --- |
| `RC2K_RENDERER=software` | Forces the software rasteriser: no GPU needed, several cores used. |
| One processor core | The views and mirrors are prepared one after another instead of side by side. The picture is the same. |
| Split screen, mirrors, rain | More to draw per frame. |

The environment variables are listed on [Running and options](Running-and-options); what runs
where is on [Platforms](Platforms).
