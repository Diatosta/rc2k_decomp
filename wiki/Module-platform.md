# Module: platform

*Category: [Module reference](Modules)*

Everything between the game and the computer. The game's own code never calls SDL or the
operating system directly; it calls this module, which keeps the meaning each service had for
the original game (what a call leaves behind, what it answers) and does the work with SDL3
or the standard library. If you are porting to a new system, this is the folder to read.

## State

Its own statics, not an owner: the window and the screen's pixels (`Screen` in
`src/platform/mod.rs`), the 3D layer's handles and the list of drawing commands being built
(`src/d3d/device.rs`, in `d3d` since it builds on the scene's drawing parts), the sound device and its buffers (`src/platform/sound.rs`).

## Main entry points

| Item | File | What it does |
| --- | --- | --- |
| `choose_screen_modes`, `set_mode_now` | `src/platform/mod.rs` | Screen modes: which exist, and switching to one. |
| `lock_surface`, `unlock_surface`, `show_frame_now` | `src/platform/mod.rs`, `src/shell/frame.rs` | Access to the screen's pixels (`platform`), and putting the finished picture on the window (`shell::frame`, which calls the platform's window and renderer wrappers) (a frame of another size than the window's, the menus' 800 x 600, is fitted by `rc2k_core::gfx::screen_fit`). |
| `frame_shown`, `set_display` | `src/platform/pacing.rs` | The frames' pace: after a frame is shown, the wait until the next one is due, on the display's refresh or by the graphics page's frame limit. |
| `pump` | `src/shell/events.rs` | Handles the window's and the devices' events (the game-facing half: see [`shell`](Module-small-modules)). |
| `draw_fan`, `flush`, `take_noted` | `src/d3d/device.rs` | Where the 3D layer's calls become drawing commands for a renderer. |
| `create_device`, `create_sound_buffer`, `play`, `set_volume` ... | `src/platform/sound.rs` | Sound buffers and their mixing into SDL's output. |
| `SdlBackend`, `pad_added`, `pad_removed` | `src/platform/input.rs` | Controllers, wheels and force feedback through SDL. |
| `play_film` | `src/shell/film.rs` | Plays a film: reads the file's container, decodes with `rc2k_core::film`, shows the frames and plays the sound. |
| `spawn`, `create_event`, `wait` | `src/platform/sync.rs` | Threads and the events they wait on. |
| `ticks`, `local_time`, `message_box` | `src/platform/os/mod.rs` | Operating-system facts, with `os/windows.rs` and `os/unix.rs` behind them. |
| `open_screen`, `present`, `pad_mapping` | `src/platform/handheld.rs` | The Anbernic handheld's screen and built-in pad. |
| `note_span`, `note_frame_shown`, `start_sampler` | `src/platform/timing.rs` | The frames' timing counters and the optional sampler; `shell::timing_report::report_timing` writes them to the log. |

## How it is laid out

One file per service, as in the table. `os/` holds what differs between Windows and the
Unix-like systems. `ffi/` holds the bindings to C libraries (`sdl.rs`, `sdl_gpu.rs`,
`sdl_input.rs`, `gl.rs`, `imgui.rs`, `win32.rs`, `unix.rs`); unsafe code that
talks to them is kept there and in the files above. `ffi/web.rs` and `web_pads.rs` (the browser's gamepads copied into SDL's virtual joysticks) are the
browser build's part, `ffi/arch.rs` holds the few processor-specific pieces, `ffi/vibrate.rs` the phone's rumble,
`ffi/process.rs` the handlers for ending the program and reporting a fault, and `event.rs` SDL's events decoded
into the few kinds the game looks at.

## Calls and callers

It hands drawing to [`render`](Module-render) and takes pixel formats from [`gfx`](Module-gfx).
It is called by [`audio`](Module-audio), [`system`](Module-system), [`menu`](Module-menu),
[`scene`](Module-scene), [`d3d`](Module-d3d), [`net`](Module-net), [`input`](Module-input) and
`threading`.

## Related pages

[Porting to a new platform](Porting-to-a-new-platform), [Drawing a frame](Drawing-a-frame),
[Films](Films), [Controllers](Controllers), [Building from source](Building-from-source).
