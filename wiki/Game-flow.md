# Game flow

*Category index. Back to [Home](Home).*

This is a guided tour of the source for someone new to it. The pages follow the program in the
order things happen: it starts, shows its menus, loads a stage, runs a race, draws and sounds
it, and comes back. Each page says what happens and where in `src/` the code for it lives, so
that you can open the right file and read on from there.

The pages name code as the code does: `race::replay` is the file `src/race/replay.rs`, and a
function usually has a file of its own with the same name. For a short description of each
folder on its own, see the [module reference](Modules). Words such as *step*, *block*, *stage*,
*event*, *owner* and *cell* are defined in the [Glossary](Glossary).

## The pages, in reading order

1. [How the game runs](How-the-game-runs): the opening page. Start-up, the front end, into a
   race and back, the threads, and a table of where to look for what.
2. [The front end](The-front-end): the loop every menu screen runs, how a screen and its menu
   items are described, text entry, the event flows (championship, single rally, time trial,
   arcade), the service area, and how the launch switches start a race without the menus.
3. [A race, step by step](A-race-step-by-step): loading a stage, the race loop, one simulation
   step in detail, how a race ends, and the replay viewer.
4. [Car physics](Car-physics): the car model as the code has it: forces, grip by surface,
   engine and gearbox, damage and wear, the set-up values and the tuning switches.
5. [Drawing a frame](Drawing-a-frame): from the scene's geometry to finished triangles, the
   command list, the three renderers, the 2D layer, and views drawn side by side on several
   threads.
6. [Sound and music](Sound-and-music): the commands the game sends to the audio thread, voices
   and channels, placing sounds in 3D, music, the co-driver's speech, the sound modes.
7. [Input](Input): keys, controllers through SDL, force feedback, how inputs reach a car, the
   handheld's controls.
8. [Threads and shared state](Threads-and-shared-state): each thread's job, what the threads
   share, and how that is kept safe.
9. [State and memory](State-and-memory): owners, typed cells, stores and handles, why there
   are no globals, and where unsafe code is allowed.
10. [The installer](The-installer): its pages, copying the files, the install record,
    uninstalling, the folders on each system.
11. [Network internals](Network-internals): sessions, the receive thread, what a car sends
    each step, and how the other players' cars are moved.

## If you only read one page

Read [How the game runs](How-the-game-runs). It is a map of the whole program on one page, and
every other page here goes deeper into one part of it.

## Related

- [Modules](Modules): one short page per folder of `src/`.
- [Data formats](Data-formats): the files the game reads and writes.
- [Building from source](Building-from-source) and [Testing](Testing).
- [Code conventions](Code-conventions): how files, errors and comments are written.
