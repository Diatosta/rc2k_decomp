# Rally Championship 2000 Decomp

A reimplementation in Rust of *Rally Championship 2000* (developed by Magnetic Fields; the
program `RAL.EXE`). It is a complete program of its own, `rc2k`: none of the original's code
runs. It reads the game's data files from the player's own copy of the game, and runs on
Windows, Linux, macOS and a handheld.

Players: [what is different from the 1999 game](Differences-from-the-original) is the list of
everything new, with the highlights at its top.

This wiki is for people who want to play it, build it, find their way around its source, or
read and write the game's files.

## Getting started

[Getting started](Getting-started) is the index. Its pages:

- [Installing the game](Installing-the-game): what you need and how the installer works.
- [Running and options](Running-and-options): command-line switches (including starting
  straight into a race), options, environment variables, logs.
- [Controllers](Controllers): keyboard, pads, wheels, force feedback.
- [Network play](Network-play): a game between machines.
- [The debug window](The-debug-window): the developers' pages as a window over the game.
- [Platforms](Platforms): what runs where.
- [Android](Android): the phone and tablet build: the touch controls, building the app, putting the game's files in.
- [iOS](iOS): the iPhone and iPad build: built and installed from Linux, the game's files through the Files app.

## Building and running from source

[Building and running](Building-and-running) is the index. Its pages:

- [Building from source](Building-from-source): what you need and the commands per system.
- [Testing](Testing): the tests and checks, and the remote test driver.
- [Porting to a new platform](Porting-to-a-new-platform): what a new system must provide.
- [Code conventions](Code-conventions): how the code is laid out and written.

## Game flow: a guide to the code

[Game flow](Game-flow) is the index. Start with
[How the game runs](How-the-game-runs): from start-up through the menus to a race and back,
the threads, and where each part lives in `src/`. Then:

- [The front end](The-front-end), [A race, step by step](A-race-step-by-step),
  [Car physics](Car-physics)
- [Drawing a frame](Drawing-a-frame), [Sound and music](Sound-and-music), [Input](Input)
- [Threads and shared state](Threads-and-shared-state), [State and memory](State-and-memory)
- [The installer](The-installer), [Network internals](Network-internals)

## Module reference

[Modules](Modules): one line per folder of `src/`, with a short page for each.

[Code layers](Code-layers): which modules may use which, and the script that checks it.

## Data formats

[Data formats](Data-formats) lists every file the game reads or writes, with a page for each
format: [replays](Replay-files), [saved games](Saved-games), [options](Options-files),
[best times](Best-times), [courses](Course-files), [objects](Object-files),
[textures](Textures-PCX), [fonts](Fonts), [sounds](Sound-samples), [texts](Language-files),
the [network messages](Network-messages) and the rest.

Modding recipes that build on these pages are planned for later.

## About the project

[About](About) is the index. Its pages:

- [Fixed bugs](Fixed-bugs): the original game's bugs fixed here: what each one was, what it
  did in the game, and how it was fixed.
- [Differences from the original](Differences-from-the-original): what was changed on
  purpose.
- [Project history](Project-history): how the port was made and checked.
- [Performance](Performance): measured frame rates and times.
- [Glossary](Glossary): the words these pages and the code use.
