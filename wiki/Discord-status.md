# Discord status

While the game runs on Windows, Linux or macOS, the Discord app can show what the player is
doing (Discord's "Rich Presence"). The handheld, Android, iOS and web builds do not have it.

## What it shows

| Where | Details line | State line |
| --- | --- | --- |
| The menus | In the menus | (none) |
| A single rally, time trial or championship | Rallying a Seat Cordoba WRC | Single rally · Ballagyr (or Time trial, Championship, A8 championship) |
| The arcade | Arcade, level 1 | Seat Ibiza Cupra Sport at Port Soderick |
| A network race | Network race | Langton · 3 players |
| A replay in the viewer | Watching a replay | the stage's name |

While racing, Discord shows the time elapsed since the race began. Names are in English whatever
language the game runs in; a car's class mark ("(A8)") is left off. What the game cannot tell
is left out of the text (a car in a network race or a replay, say). The big picture is the
car's, named by the car's English name in lower case with every run of other characters turned
into one underscore (`seat_cordoba_wrc`, `moo_1_8bse_turbo`, `renault_clio`), the logo (`logo`)
beside it; in the menus the logo alone.

## The switch

`var/rc2k.toml`, `[display]` (see [Options files](Options-files)):

| Key | What it does |
| --- | --- |
| `discord_status` | `false` turns the status off. Left out when on (the default). It has no row on the options menu, only the file. |
| `discord_application_id` | The Discord application whose status is set, in place of the one built in. Left out for the built-in one. |

## What it talks to

Only the Discord app on the same machine, over its local socket (Windows: the named pipe
`discord-ipc-0` .. `9`; Linux and macOS: the socket of that name in the runtime or temporary
folder, also where the Flatpak and Snap versions keep it). No token, account or secret is
used, and nothing is sent over the network by the game. The application's number is public: it
only says which application's name and pictures Discord shows.

The status is kept by a thread of its own: the game never waits for Discord. It sends a change
at most every two seconds, looks for Discord again every fifteen seconds while the app is not
running, and writes one line to `rc2k.log` when it connects and when it loses Discord.
A run with no window (`SDL_VIDEODRIVER=offscreen`, as the checks' copies run) and the program
that renders a replay to a video leave the status alone.

## Making the application (for someone building their own)

1. Go to discord.com/developers/applications, choose New Application and name it
   "Rally Championship" (the name is what Discord shows after "Playing").
2. Copy the Application ID from General Information. Put it in `APPLICATION_ID`
   (`src/platform/discord.rs`) or in `discord_application_id` in `rc2k.toml`.
3. Optional: under Rich Presence, Art Assets, upload an image keyed `logo` and one for each car
   keyed as above. Without them Discord shows no picture.

The texts and the frames are `rc2k_core::discord`; the connection is `src/platform/discord.rs`;
the game's state is read at the end of each frame in `src/shell/discord.rs`.
