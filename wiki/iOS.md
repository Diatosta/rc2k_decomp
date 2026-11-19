# iOS

*Category: [Getting started](Getting-started)*

A build for the iPhone and iPad. It runs on an iPhone: the menus and races by touch or a game
controller (MFi, Xbox or PlayStation pads, through SDL), vibration, screenshots into a "Rally
Championship" album, replays. It is built, signed and installed from Linux: no Mac is needed.

## What it is

The program with SDL3's UIKit and Metal backends linked in (SDL built from its source). The
game, the rasteriser and the GPU renderer are the same as on the other systems; the GPU one runs
on Metal through SDL's GPU API. Landscape only, the screen kept awake. The race frame covers the
whole screen, edge to edge; the HUD and the mirror keep clear of the notch and the home
indicator, and a menu's 4:3 page is as tall as the screen, centred, with the blurred background
at its sides. Left out of this build: the debug window, Steam, the installer's shortcuts.

## The game's files

The app does not install the game. Its Documents folder is shown in the Files app (and to a
computer's file sharing): copy the game's folder there, or a zip of it, which the app unpacks
at its first start. `var` and `savedata` are written there too, so saved games stay when the app
is installed again. With nothing there the app says so and ends.

## Screen size

The graphics page lists the screen's own size in pixels and its 3/4, 2/3 and 1/2. A race starts
at the largest of these no wider than 1920, since the menus' canvas is drawn by the processor;
the largest in the list is the sharpest.

## Touch controls

The same as on [Android](Android#touch-controls); see also [Controllers](Controllers).

## Network games

iOS lets an app neither send nor hear a broadcast on the Wi-Fi without a permission Apple grants
on request, so an iPhone finds games, and is found, through Bonjour (mDNS) instead: every build
of the game announces the games it hosts and listens for the others'. The first time, iOS asks
whether the game may find devices on the local network: it must be allowed. A host's address can
still be typed (Settings, Wi-Fi, the network's (i), IP Address) where Bonjour does not reach.

## Building and installing

`./build.sh ios` makes `dist/ios/Rally Championship.ipa` (unsigned); `tools/ios/install.sh` signs
it with an Apple ID and installs it over USB through [xtool](https://github.com/xtool-org/xtool).
The steps (Apple's SDK from an Xcode download, the packages, pairing the phone) are in
`tools/ios/README.md`.

A free Apple ID's signing lasts 7 days: after that the app no longer opens until it is installed
again (its Documents folder stays). A paid developer account's lasts a year.

## The log

`rc2k.log` is in the Documents folder. In a race it gets a line every 5 seconds with the frame
rate and where a frame's time goes.
