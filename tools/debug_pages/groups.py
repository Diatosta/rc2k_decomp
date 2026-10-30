import json
rows=json.load(open("rows.json"))
N={}
def put(start, names):
    for i,n in enumerate(names): N[start+i]=n
put(0,["Brake temperature, front","Brake temperature, rear","Tyre temperature, front","Tyre temperature, rear","Coolant temperature","Brakes wet"])
put(6,["Engine","Exhaust","Turbo","Cooling","Clutch","Differential","Electrics","Steering"])
put(14,["Tyre 1 (FL) wear","Tyre 2 (FR) wear","Tyre 3 (RL) wear","Tyre 4 (RR) wear","Front tyres","Rear tyres"])
put(20,["Reverse","Neutral"]+["Gear %d"%g for g in range(1,8)])
put(29,["Light FL","Light FR","Light RL","Light RR","Spot lights","Brake lights"])
put(35,["Window %d"%n for n in range(1,7)])
put(41,["Animation %02d"%n for n in range(17)])
put(58,["Suspension FL","Suspension FR","Suspension RL","Suspension RR","Wheel hit FL","Wheel hit FR","Wheel hit RL","Wheel hit RR","Bodywork hits","Gearbox","Body","Lights","Suspension, overall","Brakes","Tyres"])
put(73,["Frame random","Collision corner","Collision amount","Landing FL","Landing FR","Landing RL","Landing RR","Landing, overall","Gear changing","Gear in","Brake on","Turbo on","Speed","Front wheels' speed","Rear wheels' speed","Front wheels' spin","Rear wheels' spin","Water","Drive type","Tyre type","Surface","Wetness","Tyre contact","Brake balance","Suspension height","Suspension stiffness"])
put(99,["Low multiplier","Hard suspension","Soft suspension","Rough speed","Rough speed, hard","Rough speed, soft","Rev limit","Revs","Rev exponent","Rev scale","Tyre miles"])
put(110,["Chance: no electrics","Chance: no light FL","Chance: no light FR","Chance: no light RL","Chance: no light RR","Chance: no spots","Chance: no brake lights","Chance: electrics fail","Chance: light FL fails","Chance: light FR fails","Chance: light RL fails","Chance: light RR fails","Chance: spots fail","Chance: brake lights fail"])
put(124,["Engine power","Brake efficiency, front","Brake efficiency, rear","Tyre grip, front","Tyre grip, rear","Ride height, mean","Ride height FL","Ride height FR","Ride height RL","Ride height RR","Stiffness, mean","Stiffness FL","Stiffness FR","Stiffness RL","Stiffness RR","Steering pull","Handbrake failed","Gear stuck","Gear dropped","Gear bits","Electrics off","Light FL off","Light FR off","Light RL off","Light RR off","Spots off","Brake lights off"])
car=["Checkpoints left","Tyre change time","Throttle","Keys and buttons","Steering input","Brake input","Step clock","Start timer","Finished","Standing after go","Standing once going","Driver mode","Checks passed","Checkpoint","Checkpoint, last played","Checkpoint, nearest","Checkpoint offset x","Checkpoint offset z","Car kind","Brake balance","Ride height","Suspension stiffness","Gear ratios","Tyres","Steering sensitivity","Gearbox (2 automatic)","Squeeze FL","Squeeze FR","Squeeze RL","Squeeze RR","Sideways speed","Forward speed difference","Skid","Front wheels' speed","Rear wheels' speed","Forward speed","Sideways speed (2)","Spin x","Spin y (yaw)","Spin z","Wheel FL height","Wheel FR height","Wheel RL height","Wheel RR height","Wetness","Step clock (2)","Start timer (2)","Front wheels (2)","Rear wheels (2)","Forward speed (2)","Sideways speed (3)","Wheel load: spin","Wheel load: power","Wheel load: handbrake","Wanted yaw spin","Yaw spin","Yaw (2)","Steering angle","Steering, unscaled","Steering + trim","Steering pull"]+["Window %d"%n for n in range(1,7)]+["Impact, octant %d"%n for n in range(8)]
put(151,car); put(226,car)
put(301,["Players (network)","Local players","Players","Cars in the race","Views","View 1's car","View 2's car","Split"])
put(309,["Suspension height","Suspension travel","Suspension damping","Steering k","Steering j","Bump","Surface softness","Centre of gravity height","Wheel spin push","Centre of gravity z","Contact step","Accelerating grip","Braking grip","Side grip","Body hit ground","Wheel share","Drive limit 1","Drive limit 2","Drive limit","Drive slip","Slip limit","Wheel slip, front","Wheel slip, rear","Drive force, front","Drive force, rear","Axle slip, front","Axle slip, rear","Traction speed","Force, front grip","Force, rear grip","Force x","Force x, used","Force z","Force r","Force d","Grip limit","Skid speed"])
put(346,["Drive limit 1","Drive limit 2","Drive limit","Drive slip","Slip limit","Gearbox kind","0-60 time","0-100 time","Top speed","Texture changes","Points turned","Points","Polygons","Objects","Z-buffer runs","Joystick x","Joystick y","Joystick z","Ground searches","Samples","Smoothings","Found","Point checks","Checks","Gaps"])
put(371,["Surface FL","Surface FL, second","Load FL","Surface FR","Surface FR, second","Load FR","Surface RL","Surface RL, second","Load RL","Surface RR","Surface RR, second","Load RR"])
put(383,["Drive torque","Steering input","Steering, unscaled","Steering","Grip limit"])
assert all(i in N for i in range(len(rows))), [i for i in range(len(rows)) if i not in N]
G=[("Player car: heat",[0,1,2,3,4,5]),
("Player car: damage",list(range(6,14))+list(range(58,73))),
("Player car: tyres",list(range(14,20))+[92,95,109,127,128]),
("Player car: gear wear",list(range(20,29))),
("Player car: lights",list(range(29,35))+list(range(144,151))),
("Player car: failure chances",list(range(110,124))+[140,141,142,143]),
("Player car: engine and drive",[81,82,83,84,91,105,106,107,108,124,125,126,96]),
("Player car: speed and surface",[85,86,87,88,89,90,93,94,102,103,104,99]),
("Player car: suspension",[97,98,100,101]+list(range(129,140))),
("Player car: hits",[73,74,75,76,77,78,79,80]),
("Player car: windows and animation",list(range(35,58))),
]
def car_groups(base,who):
    o=base-151
    return [(who+": inputs",[x+o for x in [153,154,155,156,208,209,210,211]]),
    (who+": timing",[x+o for x in [151,152,157,158,159,160,161]]),
    (who+": course position",[x+o for x in range(162,169)]),
    (who+": set-up",[x+o for x in range(169,177)]),
    (who+": motion",[x+o for x in [181,182,183,184,185,186,187,188,189,190,195,202,203,204,205,206]]),
    (who+": suspension and wheels",[x+o for x in list(range(177,181))+list(range(191,195))]),
    (who+": windows and impacts",[x+o for x in range(212,226)])]
G+=car_groups(151,"Car 1")+car_groups(226,"Car 2")
G+=[("Session",list(range(301,309))),
("Physics: inputs and steering",[383,384,385,386,361,362,363]),
("Physics: suspension and grip",list(range(309,336))),
("Physics: forces",list(range(336,346))+[387]),
("Wheels on the ground",list(range(371,383))),
("Test drive",[352,353,354]),
("Scene and ground search",list(range(355,361))+list(range(364,371)))]
seen=set(); used=set(); out=[]
for g,idx in G:
    items=[]
    for i in idx:
        used.add(i)
        r=rows[i]
        key=(r[2],r[1])
        if key in seen: continue
        seen.add(key)
        items.append(i)
    out.append((g,items))
left=[i for i in range(len(rows)) if i not in used and (rows[i][2],rows[i][1]) not in seen]
print("unplaced and new", left)
kind={"int":"Whole","float":"Float","hex":"Hex","surf":"Surface"}
bars={}
it=json.load(open("wear_items.json"))
barset=set(x[5] for x in it if x[0]=="bar")
def esc(s): return s.replace('\\','\\\\').replace('"','\\"')
lines=[]
for g,items in out:
    lines.append('    Group {\n        name: "%s",\n        rows: &['%esc(g))
    for i in items:
        r=rows[i]
        at=int(r[2],16)
        lines.append('            Row::new("%s", "%s", Shown::%s, 0x%X, %s, (%d, %d)),'%(esc(N[i]),esc((r[0] or "").strip()),kind[r[1]],at,"true" if at in barset and r[1]=="float" else "false",r[3],r[4]))
    lines.append('        ],\n    },')
src='''//! The display page's figures (`debug::draw_wear_bars`' 380, IDA 2026-10-30) named and
//! grouped for the debug UI (owner, 2026-10-30: "a more meaningful name ... input with other
//! inputs, damage together"): generated from the page's items (`debug::wear_items`), with
//! the names IDA gives their places and the developers' labels; a place the page printed
//! twice is listed once. "Car 1" and "Car 2" are the cars' blocks 0 and 1 (0xFE0 apart);
//! "Player car" the first block's gauges and its state record (0x75B39C ..).

use super::ui::{Group, Row, Shown};

/// The groups, in the UI's order.
pub static GROUPS: &[Group] = &[
%s
];
'''%"\n".join(lines)
open("/mnt/discoII/RustRoverProjects/rc2k_hook/src/debug/display_rows.rs","w").write(src)
print(sum(len(i) for _,i in out))
