# Testing

*Category: [Building and running](Building-and-running)*

The game has no specification except the 1999 program, so the tests are built around one
question: does it still behave exactly as it did? This page lists the kinds of test, the tools
in the repository that drive the game automatically, and what to run before sending a change.

## Unit tests

```sh
cargo test -p rc2k_core
CARGO_TARGET_DIR=target/linux cargo test --target x86_64-unknown-linux-gnu -p rc2k_decomp
```

- **`rc2k_core`** holds the game's rules with no platform and no unsafe code, and most of the
  unit tests. They need no game files and run anywhere. They must also pass on Windows:

  ```sh
  cargo xwin test -p rc2k_core --xwin-arch x86 --target i686-pc-windows-msvc    # run under Wine
  ```

- **`rc2k_decomp`** (the game crate) has tests for the code that holds state.

Many tests of numeric code compare against recorded results rather than a second
implementation:

| Kind | Where | What it is |
| --- | --- | --- |
| Recorded vectors | `data/vectors/*.bin` | For a function's test inputs (made again from a seed each run), the results the original arithmetic gave, recorded once and compared on every system. |
| Frozen results | hash tables in the tests | One hash per named set of cases; a test checks its results still hash the same. |

If a change is *meant* to alter results, the tables are made again deliberately (the test
support code in `src/test_support/` says how); a table that changes by accident is a bug
found.

The tests are built lightly optimised (`[profile.test]` in `Cargo.toml`), which makes them
several times faster.

## Lints and formatting

```sh
cargo fmt
cargo clippy --target i686-pc-windows-msvc
cargo clippy --target i686-pc-windows-msvc --tests -p rc2k_decomp
CARGO_TARGET_DIR=target/linux cargo clippy --target x86_64-unknown-linux-gnu -p rc2k_decomp --tests
```

Clippy must give no warnings. The handheld's code is checked in its container:

```sh
tools/anbernic/run.sh cargo clippy --target aarch64-unknown-linux-gnu -p rc2k_decomp --features remote --tests
```

## Replays as a physics test

A [replay file](Replay-files) holds, for every second of a race, the cars' complete state and
the players' inputs. Playing it back runs the real physics on the recorded inputs. So every
replay is a test: if the physics has changed in any way, the played-back race drifts from the
recorded one.

The program checks this itself while a replay plays. Before each second's stored state is
loaded, it is compared with the state the physics reached, and differences are written to
`rc2k.log` (lines beginning `replay check`). A replay recorded by the original 1999 program
checks the physics against the original.

## Driving the game from a script: `tools/remote.py`

A build made with the cargo feature `remote` can be driven from outside: key presses, gamepad
buttons, a stand-in joystick, screenshots, and questions about its state. Ordinary builds do
not contain this code.

```sh
tools/remote.py build                       # builds rc2k with --features remote
tools/remote.py start -- --stage langton    # starts it; words after -- go to the game
tools/remote.py wait race=1 60              # waits until a race is running
tools/remote.py down Up                     # holds a key (SDL's key names)
tools/remote.py shot first_corner           # saves the next frame as a PNG
tools/remote.py state                       # frames drawn, menu, speed, time, place ...
tools/remote.py stop
```

Other commands: `key`, `up`, `press` (a key, then wait for the screen to change), `pad`, `joy`,
`pilot` and `drive` (the game's computer driver takes the player's car), `chat`, `time`, `log`.
The comment at the top of the script lists them all.

On a PC the script runs the game on a **copy** of an installed game's `var` and `savedata`, so
a test run does not touch your options and saves. The installed game is looked for in
`~/.local/share/rally-championship` (`~/rally` on a Mac), or the folder named in
`RC2K_REMOTE_INSTALLED`; the working copy is in `~/.cache/rc2k_remote`, or `RC2K_REMOTE_HOME`.

With `--handheld`, or `RC2K_REMOTE_HOST=root@address`, the same commands drive the handheld
over SSH. The script holds no address or key of its own: it reads `RC2K_REMOTE_HOST` and
`RC2K_REMOTE_KEY`, else `ANBERNIC_HOST` and `ANBERNIC_KEY` from `deploy.local`
([Building from source](Building-from-source)), and stops with a message when one is
missing. There the game's real folder is used, so the script sets `var` and `savedata` aside
at start and puts them back at stop.

`tools/remote_to_race.sh [rally] [stage]` starts the remote build straight into a stage (both
numbers counted from 0, in the stage screen's order) and returns once the race is running.
`PLAYERS=2` in the environment makes it a split-screen race.

## Held races

A "held race" is a race made repeatable: the same stage, car and random numbers, driven by the
game's computer driver, with the game's clock stepped by a fixed amount each frame instead of
read from the wall clock. Two runs of a held race must then be identical tick for tick.

With a `remote` build:

- `RC2K_HELD_RACE` holds the clock, starts the random numbers from a fixed state and puts
  the game's computer driver at the wheel;
- `RC2K_SIM_HASH` logs a running hash of the simulation's state, so two runs can be compared
  line by line. `RC2K_FAN_HASH` and `RC2K_SOUND_HASH` do the same for the drawing commands and
  the sound commands.

A change that should not alter behaviour is checked by running the same held race before and
after and comparing the logs. It is also how builds for different systems are shown to
simulate alike.

## The installer tests

```sh
RC2K_DOWNLOAD=/home/sam/rally_cd timeout 900 bash tools/instest.sh
RC2K_DOWNLOAD=/home/sam/rally_cd timeout 1500 bash tools/winstest.sh
```

`RC2K_DOWNLOAD` names a folder with the CD's files. `instest.sh` runs the Linux installer from
a throwaway copy of that folder into a throwaway home, accepting every page, and prints
`INSTALL_OK` when every file is installed and the record written. `winstest.sh` does the same
for the Windows build under Wine, in a throwaway Wine prefix. Neither touches your real home
or installation. Run both after changing anything in `src/install/`.

## Fuzzing

```sh
tools/fuzz.sh [seconds per target] [targets]
```

Fuzzes `rc2k_core`'s readers of outside data: packed files, PNG and PCX pictures, run-length
data, the text tables, language files and network messages. The targets are in
`fuzz/fuzz_targets/`. It needs a nightly toolchain and `cargo-fuzz`.

## Other tools

| Tool | What it does |
| --- | --- |
| `tools/count_unsafe.py` | Counts the unsafe blocks and functions outside tests, to keep an eye on the total. |
| `tools/vanilla_bench.py` | The script used to compare this program's speed with the original's ([Performance](Performance)). |

## Before sending a change

1. `cargo fmt`, and clippy with no warnings on the targets above.
2. `cargo test -p rc2k_core` and the game crate's tests.
3. If the change could touch the simulation: play a replay and look for `replay check`
   differences in `rc2k.log`, or compare a held race before and after.
4. If it could touch drawing: compare screenshots of the same held race or replay.
5. If it touches `src/install/`: both installer tests.
6. If it changes anything the wiki describes (a switch, a file format, a module's job), change
   the wiki page in the same commit.
