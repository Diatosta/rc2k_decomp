# iOS

*Category: [Getting started](Getting-started)*

A build for the iPhone and iPad (iOS 13 or later). It runs on an iPhone: the menus and races by
touch or a game controller (MFi, Xbox or PlayStation pads, through SDL), vibration, screenshots
into a "Rally Championship" album, replays. The project's tests were on an iPhone.

The app is not in the App Store and the released `.ipa` is not signed: the player signs it with
their own Apple ID while installing it. [Installing](#installing) says how.

## Installing

Two things go onto the phone: the app, and the game's files.

### The app, with AltStore

[AltStore](https://altstore.io) is a sideloading tool: it signs an `.ipa` with your Apple ID and
installs it. The original AltStore is meant here (it works everywhere), not AltStore PAL, the
European Union's marketplace, which takes only apps notarised by Apple.

You need:

- an iPhone or iPad with iOS 13 or later (AltStore itself may ask for a newer iOS: see its
  site);
- an Apple ID; a free one is enough;
- a Windows PC or a Mac to run AltServer on, and a USB cable for the first time;
- `Rally Championship.ipa`: it is inside `rc2k-ios.zip` on the releases page (stable releases
  only), or made by `./build.sh ios`.

Steps:

1. Install AltServer on the computer and, through it, AltStore on the phone, following
   AltStore's own guide ([faq.altstore.io](https://faq.altstore.io)). On Windows it needs
   Apple's iTunes and iCloud installed. The phone then has an AltStore app; trust your Apple ID
   under Settings, General, VPN & Device Management.
2. On iOS 16 and later switch on Developer Mode: Settings, Privacy & Security, Developer Mode.
   The phone restarts and asks to confirm. (The switch appears once a sideloaded app is on the
   phone.)
3. Get `Rally Championship.ipa` onto the phone: unzip `rc2k-ios.zip` and send the `.ipa` over
   (AirDrop, iCloud Drive, a download in Safari), so that the Files app has it.
4. Open AltStore, go to "My Apps", tap + and choose the `.ipa`. AltStore asks for the Apple ID
   the first time, signs the app and installs it as "Rally Championship".
5. Put the game's files in ([below](#the-games-files)) and start the app.

What a free Apple ID means:

- The signing lasts 7 days. After that the app no longer opens until it is signed again:
  AltStore refreshes it when the phone can reach AltServer on the same network, or by hand
  ("Refresh All" on "My Apps"). The app's Documents folder, with the game and the saves, stays.
- At most three sideloaded apps on a device at a time, AltStore itself being one.
- A paid developer account's signing lasts a year.

The game has been installed and played through AltStore, and also from Linux with xtool
([below](#building-and-installing-from-linux)), which signs the same way. If a step above
differs on your version of AltStore, its guide is the one to trust. SideStore, a variant of AltStore that refreshes without a computer, takes the
same `.ipa`; it has not been tried here either.

## What it is

The program with SDL3's UIKit and Metal backends linked in (SDL built from its source). The
game, the rasteriser and the GPU renderer are the same as on the other systems; the GPU one runs
on Metal through SDL's GPU API. Landscape only, the screen kept awake. The race frame covers the
whole screen, edge to edge; the HUD and the mirror keep clear of the notch and the home
indicator, and a menu's 4:3 page is as tall as the screen, centred, with the blurred background
at its sides. Left out of this build: the debug window, Steam, the installer's shortcuts.

## The game's files

The app does not install the game: there is no installer on a phone, and the game is played
from the folder it is copied into. An installed game's folder (the one with `files` and `var`,
from a PC) is the case that has been tried.

1. Start the app once. Finding no game, it says so, leaves a note named
   `Put the game here.txt` in its Documents folder and ends. (The Files app lists an app's
   folder only once it holds something.)
2. Open the Files app: Browse, On My iPhone, Rally Championship.
3. Copy the game's folder there, or a zip of it (from iCloud Drive, AirDrop, a USB stick, or a
   computer's file sharing: Finder or iTunes show the same folder). The game's folder may be one
   level inside the zip. A zip larger than 4 GB, or an encrypted one, is not read.
4. Start the app again. A zip is unpacked first, which takes a moment.

`var` and `savedata` are written there too, so saved games stay when the app is installed
again or refreshed. Deleting the app from the home screen deletes them.

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

## Building and installing from Linux

No Mac is needed to build, sign or install. `./build.sh ios` makes `dist/ios/Rally Championship.ipa` (unsigned); `tools/ios/install.sh` signs
it with an Apple ID and installs it over USB through [xtool](https://github.com/xtool-org/xtool).
The steps (Apple's SDK from an Xcode download, the packages, pairing the phone) are in
`tools/ios/README.md`.

The automatic builds make the same unsigned `.ipa` on a Mac runner, with Xcode's iOS SDK
(`rc2k-ios.zip` in a release, for the main branch). The 7 days of a free Apple ID's signing
apply here as well: run `tools/ios/install.sh` again to renew it.

## The log

`rc2k.log` is in the Documents folder. In a race it gets a line every 5 seconds with the frame
rate and where a frame's time goes.
