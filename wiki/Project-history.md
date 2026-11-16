# Project history

*Category: [About](About)*

How the port was made, in the order it happened, and how each stage was checked. The work was
done by Diatosta with AI coding assistants, over the autumn of 2026.

## 1. One function at a time, inside the running game

The original program was read in a disassembler, function by function, starting with the
functions that call nothing else and working upwards. Each was rewritten in Rust and loaded
into the running game in place of the original function.

Every replaced function could be switched between the original and the Rust version while the
game ran, or run in a comparing mode: both versions ran on the same inputs, the original's
result was used, and any difference in results or in the memory written was logged. Each
function also had a test of its own that fed both versions made-up inputs. A function counted
as done when its test passed and play showed no differences.

During this stage the original's mistakes were kept, so that the two versions could be
compared exactly. They were noted for later.

It ended when every function the game reaches was ported, about 2,400 of them, and whole
sessions ran with no call into original code.

## 2. A program of its own

In parallel, the parts of the original that talk to Windows and DirectX were given stand-ins
built on SDL3, a software rasteriser and FFmpeg, rather than ported one to one. With those, the
Rust code could run as a program of its own, without the original's code, reading the original
program's data from a copy. The installer was brought over the same way, with a file in place
of the registry.

Then the scaffolding went: the mechanism for replacing functions, the comparing mode, and
finally the original program's data image, whose tables became Rust source
([Built-in tables](Built-in-tables)).

## 3. Into ordinary Rust

At that point the code still thought like the original: variables at fixed addresses, 32-bit
pointers kept in tables, floating-point code written in assembly to match the old processor
instructions bit for bit. This stage turned it, module by module, into typed Rust:

- floating-point code as plain Rust that rounds as the original did;
- every variable a typed field of a store, in an owner, instead of a place in a block of memory
  ([State and memory](State-and-memory));
- addresses kept in data replaced by handles;
- Windows calls replaced by the standard library and SDL, which made the 64-bit, Linux and
  macOS builds possible;
- unsafe code reduced to a small, documented remainder; `rc2k_core` has none.

With no original to compare against any more, the checks became replays and held races.
A replay stores the cars' state every second, so playing an old replay shows at once if the
physics has drifted. A held race is a repeatable race whose every step can be compared between
two builds ([Testing](Testing)). Each change had to leave them identical, and the pictures too.

The original's bugs were fixed during this stage ([Fixed bugs](Fixed-bugs)).

## 4. Speed and ports

A comparison with the original program closed the conversion ([Performance](Performance)).
Then came the work on speed, mostly for the handheld: a GPU renderer for the desktop and an
OpenGL ES one for the handheld, the players' views and mirrors
prepared on several threads at once, and many smaller savings in the game's own 3D code. The
rule throughout was that the picture and the simulation stay identical.

The ports to the Anbernic RG34XXSP handheld and to macOS, and the experimental web build, were
made in this stage ([Platforms](Platforms)).

## 5. Improvements

The current stage. So far: a build script for Windows itself, starting straight into a race
from the command line, and a start-up without the original's pauses. New features come here;
the game as the player knows it stays the default.

## What the checks were

| Stage | Check |
| --- | --- |
| Porting | Each function against the original on the same inputs, in tests and in play. |
| The stand-alone program | Play, and sessions logged with no call into original code. |
| Conversion | Unit tests frozen from the earlier comparisons; replays; held races; screenshots compared; play. |
| Speed | The same, plus measured runs on the desktop and the handheld. |
| The installer | End-to-end installs into a throwaway home, on Linux and under Wine. |
