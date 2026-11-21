# Module: clock

*Category: [Module reference](Modules)*

The game's time, in the layer next to the platform: every module may read it, and it uses
nothing of the game. (A remote run's speed and a rendered video's own clock plug in here.)

## Main entry points

| Item | File | What it does |
| --- | --- | --- |
| `tick_count` | `src/clock/tick_count.rs` | The millisecond clock, taking the `PictureClock` (the override the replay viewer sets while it saves pictures). The video renderer names its own clock with `use_video_clock`. |
| `sleep` | `src/clock/sleep.rs` | Sleeps; `duration` is how long a number of the game's milliseconds last. |
| `packed_local_time` | `src/clock/packed_local_time.rs` | The date and time as one number (as stored in replays and saves). |

## Calls and callers

It calls [`platform`](Module-platform) for the machine's clock. Nearly every module calls it;
[`system`](Module-system) holds the `PictureClock` in its owner.
