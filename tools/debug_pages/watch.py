#!/usr/bin/env python3
"""src/debug/watch_rows.rs from watch_items.json (g_debugWatchList 0x5EBB5A, read in IDA:
label, kind, address, car block, IDA's name or the block's member, offset), one row per
(address, kind). The names and groups are written here by hand; run from the repository."""
import json, re, collections
items = json.load(open("tools/debug_pages/watch_items.json"))
# (group, name) for the variables outside the cars' blocks, by the developers' label.
OUT = {
 "or.steer": ("Controls", "Steering, overlay's"), "usesteer": ("Controls", "Steering used"),
 "paused": ("Race state", "Paused"), "gmpaused": ("Race state", "Scene switch (game paused)"),
 "rep mode": ("Race state", "Replay mode"), "g spmode": ("Race state", "Split mode, wanted"),
 "g nump": ("Race state", "Players, wanted"), "s spmode": ("Race state", "Split mode"),
 "s nump": ("Race state", "Players"), "numplayr": ("Race state", "Players"),
 "numcars": ("Race state", "Cars in the race"), "slow by4": ("Race state", "Race ticks (the levers' path only)"),
 "shaunrps": ("Race state", "After the credits"), "shaunscn": ("Race state", "View switch"),
 "shaunspl": ("Race state", "Split mode"), "shaunp1o": ("Race state", "View 1 on"),
 "shaunp2o": ("Race state", "View 2 on"), "no updat": ("Race state", "Not updating"),
 "no drawf": ("Race state", "Not drawing"), "f timer": ("Race state", "Finish time"),
 "f timeru": ("Race state", "Finish time, unrounded"), "timechkd": ("Times and places", "Split car time"),
 "totatime": ("Times and places", "Stage's total time"), "numcheck": ("Times and places", "Split colours"),
 "numsortd": ("Times and places", "Checkpoints"), "bloffst3": ("Times and places", "Split point's distance"),
 "arcstpos": ("Times and places", "Arcade: start place"), "arcbestp": ("Times and places", "Arcade: best place"),
 "arcacnum": ("Times and places", "Arcade: start slot"),
 "btoadd a": ("Mud on the cars", "Splashes to add"), "btoadd b": ("Mud on the cars", "Splashes to add"),
 "btoadd c": ("Mud on the cars", "Splashes to add"), "btoadd d": ("Mud on the cars", "Splashes to add"),
 "boncar a": ("Mud on the cars", "Splashes on the car"), "boncar b": ("Mud on the cars", "Splashes on the car"),
 "boncar c": ("Mud on the cars", "Splashes on the car"), "boncar d": ("Mud on the cars", "Splashes on the car"),
 "mtoadd a": ("Mud on the cars", "Mud to add"), "mtoadd b": ("Mud on the cars", "Mud to add"),
 "mtoadd c": ("Mud on the cars", "Mud to add"), "mtoadd d": ("Mud on the cars", "Mud to add"),
 "moncar a": ("Mud on the cars", "Mud on the car"), "moncar b": ("Mud on the cars", "Mud on the car"),
 "moncar c": ("Mud on the cars", "Mud on the car"), "moncar d": ("Mud on the cars", "Mud on the car"),
 "nrvbpols": ("Sound", "Reverb (EAX) faces"), "rvbtotal": ("Sound", "Reverb environment"),
 "commu.hd": ("Sound", "Listener near"), "gumbo": ("Sound", "Sound objects shown"),
 "cteffect": ("Sound", "Car 1, sound ct: effect"), "ctvolume": ("Sound", "Car 1, sound ct: volume"),
 "ctposn.x": ("Sound", "Car 1, sound ct: x"), "ctposn.y": ("Sound", "Car 1, sound ct: y"),
 "ctposn.z": ("Sound", "Car 1, sound ct: z"), "sceffect": ("Sound", "Car 1, sound sc: effect"),
 "scvolume": ("Sound", "Car 1, sound sc: volume"), "scposn.x": ("Sound", "Car 1, sound sc: x"),
 "scposn.y": ("Sound", "Car 1, sound sc: y"), "scposn.z": ("Sound", "Car 1, sound sc: z"),
 "cvr1revs": ("Sound", "Car 1: revs"), "car2srev": ("Sound", "Car 2: revs"), "car3srev": ("Sound", "Car 3: revs"),
 "car4srev": ("Sound", "Car 4: revs"), "car1xpos": ("Sound", "Car 1: x"), "car2xpos": ("Sound", "Car 2: x"),
 "car3xpos": ("Sound", "Car 3: x"), "car4xpos": ("Sound", "Car 4: x"), "car1ypos": ("Sound", "Car 1: y"),
 "car2ypos": ("Sound", "Car 2: y"), "car3ypos": ("Sound", "Car 3: y"), "car4ypos": ("Sound", "Car 4: y"),
 "car1zpos": ("Sound", "Car 1: z"), "car2zpos": ("Sound", "Car 2: z"), "car3zpos": ("Sound", "Car 3: z"),
 "car4zpos": ("Sound", "Car 4: z"), "car1locl": ("Sound", "Car 1: local"), "car2locl": ("Sound", "Car 2: local"),
 "car3locl": ("Sound", "Car 3: local"), "car4locl": ("Sound", "Car 4: local"),
 "trackdis": ("Camera", "Distance"), "car1offx": ("Camera", "Offset x"), "car1offy": ("Camera", "Offset y"),
 "car1offz": ("Camera", "Offset z"), "ctfrac": ("Camera", "View share"), "rcamjump": ("Camera", "Height bias"),
 "crotywan": ("Camera", "Start heading"), "levddist": ("Camera", "Grid reach"),
 "skidowna": ("Skid marks", "Owner"), "nextskid": ("Skid marks", "Next"), "nskidmrk": ("Skid marks", "Next"),
 "skidamtt": ("Skid marks", "Amount, total"),
 "lightxs1": ("Lights and sun", "Light 1: x"), "lightys1": ("Lights and sun", "Light 1: y"),
 "lightzs1": ("Lights and sun", "Light 1: z"), "lightss1": ("Lights and sun", "Light 1: s"),
 "lightcs1": ("Lights and sun", "Light 1: colour"), "lightds1": ("Lights and sun", "Light 1: d"),
 "lightxrs": ("Lights and sun", "Light 1: x, turned"), "lightyrs": ("Lights and sun", "Light 1: y, turned"),
 "lightzrs": ("Lights and sun", "Light 1: z, turned"), "uselight": ("Lights and sun", "Lamps wanted"),
 "flarnten": ("Lights and sun", "Flare intensity"), "hazenten": ("Lights and sun", "Haze intensity"),
 "sunxscn": ("Lights and sun", "Sun on the screen, x"), "sunyscn": ("Lights and sun", "Sun on the screen, y"),
 "drnalpha": ("Lights and sun", "Alpha (drnalpha)"), "cdrawmsk": ("Scene", "Car's draw mask"),
 "courseps": ("Scene", "Course's place"), "cpartsal": ("Scene", "Car parts allocated"),
 "c w c c": ("Scene", "Mark's car file"), "pagesupd": ("Scene", "Texture pages updated"),
 "uplobits": ("Scene", "Texture bits uploaded"), "olvimode": ("Overlay", "View mode"),
 "olayp1ps": ("Overlay", "Player 1's place"), "olayp2ps": ("Overlay", "Player 2's place"),
 "olayfcd1": ("Overlay", "Distance's fraction"), "olayfdis": ("Overlay", "Distance"),
 "vidcar0a": ("Scene", "Video car a"), "vidcar0b": ("Scene", "Video car b"),
 "vidcar0c": ("Scene", "Video car c"), "vidcar0d": ("Scene", "Video car d"),
 "window": ("Damage", "Window"), "stemp": ("Scratch", "Temporary"),
 "whlspnfx": ("Wheels' spin records", "Spin x"), "whlspnfz": ("Wheels' spin records", "Spin z"),
 "rplsizes": ("Replay", "Sizes"), "rpchunk": ("Replay", "Block"), "pbsecon": ("Replay", "Playback seconds"),
 "rpframe": ("Replay", "Frame"), "pbticks": ("Replay", "Playback ticks"), "rpcams": ("Replay", "Cameras"),
 "car fzsi": ("Car spec", "Front z size"), "car rzsi": ("Car spec", "Rear z size"), "car sisi": ("Car spec", "Side size"),
}
for n in range(1, 8):
    OUT["part %dn" % n] = ("Damage", "Part %d, shown" % n)
for s, name in zip("123456789abc", range(1, 13)):
    for k, what in (("s", "steering"), ("a", "acceleration"), ("b", "braking")):
        OUT["s%s.%sgrip" % (s, k)] = ("Grip by surface", "Surface %d: %s grip" % (name, what))
CAR = [("Cars: inputs and engine", r"^(input_|keys_|throttle|fast_settle|handbrake|steering|gear$|engine|option_bits)"),
       ("Cars: progress and times", r"^(checkpoint|check_count|split_|time_|step_clock|previous_step_clock|off_line|turned_|.*stand_timer|finish|out$|two_car_place$|pace_|mode2|grade_mode|lights_on|heading)"),
       ("Cars: camera", r"^(camera|pending|start_)"),
       ("Cars: messages", r"^notice"),
       ("Cars: wheels and skids", r"^(skid|slip|wheel_|drive_speed|local_motion|tyre_)"),
       ("Cars: damage", r"^(impact|part_)")]
def nice(m):
    m, tail = re.match(r"([^\[]*)(\[\d+\])?", m).groups()
    return m.replace("_", " ").capitalize() + (" " + tail if tail else "")
groups = collections.OrderedDict()
count = collections.Counter()
for label, kind, at, car, ida, off in items:
    label = label.strip()
    if car:
        n = int(car[3:]) + 1
        g = next((g for g, r in CAR if re.match(r, ida)), "Cars: other")
        name = "Car %d: %s" % (n, nice(ida))
        if ida.startswith("unknown"):
            name = "Car %d: %s (block +%s)" % (n, label, off)
    else:
        if label not in OUT:
            raise SystemExit("no name for " + label)
        g, name = OUT[label]
        count[(g, name)] += 1
    groups.setdefault(g, []).append((name, label, kind, at))
ORDER = ["Controls", "Race state", "Times and places", "Cars: inputs and engine", "Cars: progress and times",
 "Cars: wheels and skids", "Cars: damage", "Cars: camera", "Cars: messages", "Cars: other", "Car spec",
 "Grip by surface", "Damage", "Wheels' spin records", "Mud on the cars", "Skid marks", "Camera", "Lights and sun",
 "Scene", "Overlay", "Sound", "Replay", "Scratch"]
assert set(ORDER) == set(groups), set(groups) ^ set(ORDER)
lines = []
seen = collections.Counter()
for g in ORDER:
    lines.append('    Group {\n        name: "%s",\n        rows: &[' % g)
    for name, label, kind, at in groups[g]:
        if count[(g, name)] > 1:
            seen[(g, name)] += 1
            name = "%s %d" % (name, seen[(g, name)])
        lines.append('            Row::new("%s", "%s", Shown::%s, 0x%X, false, (0, 0)),' % (name.replace('"', '\\"'), label, "Float" if kind == 3 else "Whole", at))
    lines.append("        ],\n    },")
open("src/debug/watch_rows.rs", "w").write('''//! The watch list's variables (`g_debugWatchList`, 0x5EBB5A, 483 entries, shown by
//! `debug::draw_watch_list` 0x427992 on pages 4 .. 7; IDA 2026-10-30), one row per place,
//! named and grouped for the debug UI: generated by `tools/debug_pages/watch.py`. "Car n" is
//! the block n - 1 (0xFE0 apart). Names outside the cars' blocks are read from the
//! developers' labels, not from what the code does: a reading.

use super::ui::{Group, Row, Shown};

/// The groups, in the UI's order.
pub static GROUPS: &[Group] = &[
%s
];
''' % "\n".join(lines))
print(sum(len(v) for v in groups.values()), {g: len(groups[g]) for g in ORDER})
