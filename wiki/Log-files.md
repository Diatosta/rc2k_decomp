# Log files

*Category: [Data formats](Data-formats)*

The program writes two logs, both plain text, and the debug window keeps a small file of its
own. None of them is read back by the game; they are for people. When something goes wrong,
`rc2k.log` is the file to look at first and to send with a bug report.

| File | Written by | Holds |
| --- | --- | --- |
| `rc2k.log` | this program | Each step of start-up and shut-down, warnings, and the reason if the program stopped itself or crashed. A second copy started with `--allow-another` writes `rc2k-2.log` (and so on) instead. |
| `var/logfile.txt` | the game's own log code, as in 1999 | What the game says about the machine, its data and each stage it loads. |
| `var/imgui.ini` | the debug window | Where its windows and tables were left. |

## `rc2k.log`

The file is in the folder the program was started in. It is emptied at every start, so it
always describes the last run only. A different name can be given as the first word on the
command line that is not a switch:

```sh
rc2k mylog.txt --stage langton
```

Lines are appended as things happen, one message to a line, so the file is complete up to the
moment the program stopped. Most lines begin with the part of the program that wrote them:

```
standalone: start calls settings::read_settings
standalone: start calls logfile::start_log
...
standalone: ending: start came to its end
```

What to look for:

| Line | Meaning |
| --- | --- |
| `standalone: start calls ...` | A step of start-up or shut-down was reached. If the program stops at start-up, the last of these says where. |
| `standalone: ending: ...` | Why the program ended. `start came to its end` is the normal way out. |
| `<function> refused: <reason>` | The program refused to carry on with data it could not make sense of (a damaged file, a value out of range), and stopped. |
| `<function>: raises fatal error <code> <message>` | One of the game's own fatal errors, with its code. |
| a fault report | The program crashed: the kind of fault, the address, and the functions it was in. |

There is no fixed grammar beyond "one message per line"; do not write tools that depend on the
exact wording.

On the handheld and in the web build the log goes to the same file; the web build also shows
each line in the browser's console.

## `var/logfile.txt`

This is the 1999 game's log, written by the game's own logging routines. `logfile=` in
`var/ral.cfg` sets how much is written: 0 nothing, 1 the main messages (the default when the
key is absent), 2 or more everything (see [Settings files](Settings-files)).

The file is deleted and begun again at every start. It begins with a head like this (the values
are made up):

```
Log started Monday 3rd May 2027 14:07:31.52

$VER: Rally 5.30 (4.8.2000)

cpu= 15 00A20F10 AuthenticAMD
fpu= 1
mmx= 1
amdk3d= 0
katmai= 1
machinetype= 1
platform= 2 7 3 4 Linux 7.3.4
ram avail/total: 4194303 4194303 Kb
swp avail/total: 4194303 4194303 Kb
free space on partition= 44153324 Kb
```

| Line | Meaning |
| --- | --- |
| `Log started` | The date and time, to hundredths of a second. |
| `$VER:` | The version of the original game that this program reproduces. |
| `cpu=` | The processor's family, its identification number in hexadecimal, and its maker's name. |
| `fpu=`, `mmx=`, `amdk3d=`, `katmai=` | 1 if the processor has floating point, MMX, 3DNow! and SSE. The program reports the same set on every machine; the game's arithmetic has one path whatever they say. |
| `machinetype=` | The keyboard type. |
| `platform=` | The system's version numbers and name. |
| `ram`, `swp` | Free and total memory and swap space, in Kb, as far as a 32-bit number can say. |
| `free space on partition=` | Free space on the disc the game is on, in Kb. |

The original game's `cpu=` line ended with `clk=` and the processor's speed in MHz, which the
game measured by timing the processor for a quarter of a second at every start. This program
does not make that measurement (it was one of several pauses that made the 1999 start-up slow),
so the line has no `clk=` part.

After the head come the game's own messages: the compiling of the texts (lines beginning
`LC:`), the files it loads, each stage's loading steps, replay headers as they are read, and so
on. Lines are indented by nesting. The wording is the original's.

On Linux and macOS the file is kept open while the game runs and can be read at any time; on
Windows it is opened for each write.

## `var/imgui.ini`

The debug window (Ctrl+F11; see [The debug window](The-debug-window)) is built with Dear ImGui,
which remembers where its windows were and which columns its tables showed. It keeps that in
its own `.ini` format in `var/imgui.ini`:

```ini
[Window][Debug]
Pos=6,3
Size=790,589
Collapsed=0
```

The file can be deleted at any time; the window then opens in its default place. It is made
only by builds that have the debug window (not the 32-bit Windows build or the handheld's).

Code: `src/logfile/program_log.rs` and `src/standalone.rs` (`rc2k.log`), `src/bin/rc2k.rs`
(its name), `src/logfile/start_log.rs` and `src/logfile/write_line.rs` (`var/logfile.txt`),
`rc2k_core/src/files/log.rs` (the head's lines), `src/debug/imgui_sdl.rs` (`var/imgui.ini`).
