# Films

*Category: [Data formats](Data-formats)*

The game has three films: two company logos shown at start-up and the intro. They are in
standard formats. This page says which, and what a replacement must be.

## The files

| File | Container | Video | Sound | When it plays |
| --- | --- | --- | --- | --- |
| `files/fmv/ALLOGO.AVI` | AVI | Indeo 4 (`IV41`), 320 × 240, 30 frames a second | none | First, at start-up. |
| `files/fmv/MFLOGO.MPG` | MPEG-1 program stream | MPEG-1 video, 352 × 288 | MPEG audio layer 2, stereo, 44,100 Hz | Second, at start-up. |
| `files/fmv/INTRO.MPG` | MPEG-1 program stream | MPEG-1 video, 352 × 288 | MPEG audio layer 2, stereo, 44,100 Hz | After the logos, and again whenever the main menu is left idle. |

A missing film is skipped. If `ALLOGO.AVI` is missing, `ALLOGO.MPG` is tried in its place.
The setting `nofmv` in `var/ral.cfg` turns all films off, and starting straight into a race
from the command line skips them too.

Any key ends a film.

## What the player accepts

The original program handed the films to Windows' film player. This project has a small
player of its own, which reads the two containers itself and uses FFmpeg only to decode the
pictures and the sound. The project builds FFmpeg with just three decoders, so nothing else
will play:

| Decoder | Used for |
| --- | --- |
| Indeo 4 | The AVI's pictures. |
| MPEG-1 video | The MPEG files' pictures. |
| MPEG audio layer 2 | The MPEG files' sound. |

### AVI

| Requirement | Detail |
| --- | --- |
| Form | `RIFF` .. `AVI `, with the usual `hdrl` and `movi` lists. |
| Video | The first video stream is played. Its coding (the four characters in the stream's header) must be `IV41`. |
| Timing | The time a frame from the main header (microseconds); if that is 0, from the stream header's scale and rate. |
| Frames | The stream's chunks in the order they lie in `movi`. A chunk of no bytes shows the previous picture again. The index at the end of the file is not needed. |
| Sound | Not played from an AVI. |

### MPEG

| Requirement | Detail |
| --- | --- |
| Form | An MPEG-1 program stream (the ordinary `.mpg` of the 1990s; not a transport stream, not MPEG-2). |
| Video | Stream `0xE0`, MPEG-1 video. The frame rate is taken from the video's own sequence header. |
| Sound | Stream `0xC0`, MPEG audio layer 2. Optional. |

The sound is decoded first and played from the moment the first picture is shown. The
pictures then follow the sound: each waits for its time, and one that is more than two
frames late is dropped. A film without sound is timed by the clock.

### Picture

Each picture is stretched to fill the whole screen, whatever its size and shape, without
smoothing. There is no letterboxing: a replacement film should have the shape of the screen
(4 : 3) to look right.

## Making a replacement

With FFmpeg's command-line program, a film the game plays can be made like this (file names
made up):

```sh
ffmpeg -i my_intro.mp4 -c:v mpeg1video -b:v 1500k -s 352x288 -r 25 \
       -c:a mp2 -ar 44100 -ac 2 -b:a 192k -f mpeg INTRO.MPG
```

FFmpeg cannot write Indeo 4, so the simplest replacement for `ALLOGO.AVI` is to remove it
and put an `ALLOGO.MPG` made the same way in its place.

## Code

`src/menu/play_films.rs`, `rc2k_core/src/menu/films.rs` (which film when),
`src/platform/film.rs` (the AVI reader, the MPEG reader, timing, drawing),
`src/platform/ffi/ffmpeg.rs` (the decoders), `tools/build_ffmpeg.sh` (how FFmpeg is built).
