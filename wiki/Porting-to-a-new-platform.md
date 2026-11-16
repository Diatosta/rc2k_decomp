# Porting to a new platform

*Category: [Building and running](Building-and-running)*

The game's own code never calls the operating system, SDL or a graphics API directly.
Everything between the game and the machine is in two folders:

- `src/platform/`: the window, events, input devices, sound output, films, threads, facts
  about the system;
- `src/render/`: the renderers that turn the game's drawing commands into a picture.

A port is mostly work in these two, plus a few places that choose per system. This page says
what each part must provide and uses the two existing ports beyond the desktop, the Anbernic
handheld and macOS, as examples.

## How much SDL already covers

The platform layer is built on SDL3. Where SDL runs, most of a port exists already: the
window, keyboard, gamepads and wheels, haptics, and sound output all go through SDL. FFmpeg
decodes the films. So the first question for a new system is: does SDL3 have a video and an
audio driver for it, and is there a GPU API SDL's GPU device supports (Vulkan, Metal)?

| If the system has | Then |
| --- | --- |
| SDL video + a GPU SDL supports | Little to do: the desktop path works (see macOS below). |
| SDL video, no usable GPU | The software rasteriser draws everything; nothing to write. |
| No SDL video at all | You show the frames yourself, and write a renderer if you want the GPU (see the handheld below). |

## What the platform layer provides

| File in `src/platform/` | Its job | For a port |
| --- | --- | --- |
| `mod.rs` | The screen: the game draws 2D into a block of memory in the screen mode's pixel layout; `show_frame` puts it on the window. Also setting a screen mode and making the window. | Works wherever SDL has a video driver. Otherwise add your own way of showing a frame. |
| `events.rs` | Turns SDL's events into what the game expects: the quit switch, window focus, the pending key for the menus, gamepad buttons as menu keys. | Add a way to quit if the device has no window to close. |
| `input.rs` | Joysticks, gamepads and wheels as the game's controllers; force feedback through SDL's haptics. | Add a controller mapping if SDL does not know the built-in pad. |
| `sound.rs` | Sound output: a small mixer that plays the game's sound buffers, with volume, pan and 3D placing, into one SDL audio stream. | Nothing, where SDL has audio. |
| `film.rs` | The films, decoded with FFmpeg and shown as frames. | Build FFmpeg's decoders for the target (`tools/build_ffmpeg.sh`). |
| `direct3d.rs` | The 3D layer the game's drawing code stands on: textures, the start and end of a scene, and collecting the drawing commands for a renderer. | Nothing. |
| `sync.rs` | Threads, events and locks, on Rust's standard library. | Nothing. |
| `timing.rs` | Clocks and the optional profiler. | Nothing, unless you want the profiler. |
| `os/` | Facts about the system the game asks for: the clock, the command line, folders, free space, message boxes. `windows.rs` and `unix.rs` answer the per-system ones. | A Unix-like system uses `unix.rs`; add the calls that differ. |
| `ffi/` | Thin safe wrappers around the C libraries (SDL, SDL's GPU API, FFmpeg, OpenGL ES, libc, Win32, Dear ImGui). The only place in the platform that may use unsafe code. | A new library gets a new file here. |
| `arch.rs` | What differs between processors (intrinsics). | A new processor may need entries. |
| `handheld.rs`, `web.rs` | The two ports that needed a file of their own. | A model for yours. |

## What a renderer provides

The game does its own 3D geometry and produces finished, screen-space triangles with their
textures and drawing states. These are collected into a list of commands. A renderer is one
trait with one method (`src/render/mod.rs`):

```rust
pub trait Backend: Send {
    fn draw(&mut self, target: &Target, commands: CommandList);
}
```

`draw` must draw all the commands, in order, before it returns. The rules every renderer must
agree on (what each state means, how textures are converted) are in `src/render/mod.rs` and
`src/render/batches.rs`, shared by all of them.

| Renderer | Files | Notes |
| --- | --- | --- |
| Software rasteriser | `software.rs`, `rasteriser.rs`, `rasteriser_neon.rs` | Needs nothing from the system. Splits the picture over the processor's cores. It is the reference: the others are compared against it. |
| SDL GPU (Vulkan, Metal) | `gpu.rs`, `gpu/`, `shaders/` | The desktop's default. |
| OpenGL ES 3 | `gles.rs`, `gles/`, `gl_batching.rs` | The handheld's. |

`src/render/choice.rs` decides which one is tried: the GPU first, and if any step of starting
it fails, the reason is logged and the software rasteriser draws instead. A port should keep
that fallback.

The game's 2D (menus, text, the in-race display) is drawn by the game into memory. A GPU
renderer has to combine that with its 3D; `gpu.rs` and `gles/layer2d.rs` show how.

## The other places that choose per system

| Place | What is chosen |
| --- | --- |
| `Cargo.toml` | How SDL is built and linked for the target, and whether the debug window's library is included. |
| `build.rs` | Where FFmpeg's libraries are for the target; whether the debug window is built in. |
| `src/install/folders/` | The installer's rules for naming and checking folders and its default folder. One implementation per system. |
| `src/install/shortcuts.rs` | The menu entries the installer makes. |
| `src/files/` | Reading the game's paths (written the Windows way, without regard to case) on a system whose file names are case-sensitive. Already done for Unix. |
| `deploy_standalone.sh`, `tools/` | Building and copying for the target. |

## Example: the Anbernic RG34XXSP

A handheld with a 64-bit ARM processor, a Mali GPU, and a system with no display server SDL
knows. In the code it is "Linux on 64-bit ARM".

What the port needed:

- **Showing frames.** SDL runs with its dummy video driver, for sound and the pad only.
  `src/platform/handheld.rs` writes the frames to the framebuffer device itself.
- **A renderer.** There is no Vulkan driver, so SDL's GPU device cannot start. `gles.rs` draws
  with OpenGL ES 3 through the system's own GPU library. It has a thread of its own for the
  GPU calls and another that prepares batches, because the processor is slow.
- **A fallback that is fast enough.** With the software rasteriser, races are drawn at half
  size and enlarged.
- **The pad.** A controller mapping for the built-in pad (`handheld.rs`), and MENU + START as
  the way out (`events.rs`), since there is no window to close.
- **Processor-specific code.** A NEON version of the rasteriser's inner loops
  (`rasteriser_neon.rs`).
- **No debug window**, since there is no window to draw it over (`build.rs`).
- **Build tooling.** A container that matches the device's system (`tools/anbernic/`), a
  launcher script for the device's menu, and a `deploy_standalone.sh anbernic` step.

## Example: macOS

SDL has video, audio and a GPU API (Metal) there, so the desktop path was reused.

What the port needed:

- **Shaders for Metal.** The shaders are written once and committed in three forms: source,
  SPIR-V for Vulkan and Metal's language (`src/render/shaders/`, made by
  `tools/compile_shaders.sh`), so a build needs no shader compiler.
- **System facts.** `os/unix.rs` serves Linux and macOS; where the two differ (memory
  figures, a few low-level calls) `ffi/unix.rs` has a version for each.
- **The bundle.** A program inside `Rally Championship.app` must work in the folder that holds
  the bundle, not inside it; the program moves there before it reads any file.
- **The installer.** The Unix folder rules with a Mac default folder, and menu entries as
  small application bundles.
- **Build tooling.** FFmpeg for both processors, `lipo` to join the two programs, ad hoc
  signing (`deploy_standalone.sh mac`, `tools/mac/`).

One renderer detail differs: on macOS the menus' 2D is still uploaded as pixels rather than
drawn by the GPU as rectangles.

## A checklist

1. Get SDL3 and FFmpeg's decoders building for the target.
2. Build with the software rasteriser first (`RC2K_RENDERER=software`). Menus, a race, sound
   and input should all work before any GPU work starts.
3. Check a replay plays back without `replay check` differences in `rc2k.log`, and compare a
   held race with another system's ([Testing](Testing)). The simulation must be identical on
   every system; this is why the code uses its own maths functions instead of the system's
   ([Code conventions](Code-conventions)).
4. Then the GPU renderer, compared against the rasteriser's pictures.
5. The installer's folders and menu entries, if the system has an installer step.
6. Add the system to [Platforms](Platforms) and the build steps to
   [Building from source](Building-from-source).
