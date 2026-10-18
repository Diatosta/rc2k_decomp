//! The race's drawing (`sub_4541BF`'s tree, IDA 2026-10-17): its leaves that are game logic,
//! not platform. The panel's fields at 0x6D14xx are a reading.

use crate::errors::{BadInput, Refused};
use crate::framework::{EAX, EBX, ECX, EDI, EDX, ESI, game_read, game_write, rva};
use crate::race::frame::VIEW_BLOCKS;
use crate::race::objects::{CAR_BLOCKS_RVA, CARS_RVA};
use crate::screens::harness::Left;

unsafe fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// `race::car_label` (0x45B1B7): at edi seven bytes, " CAR nn" with nn eax's two digits
/// (eax / 10 and eax % 10, each + '0' as a byte), or seven spaces when ecx is -1. Every
/// register kept.
pub unsafe fn car_label(edi: u32, eax: u32, ecx: u32) {
    let p = edi as *mut u8;
    let label: [u8; 7] = if ecx == u32::MAX {
        [b' '; 7]
    } else {
        [
            b' ',
            b'C',
            b'A',
            b'R',
            b' ',
            ((eax / 10) as u8).wrapping_add(b'0'),
            ((eax % 10) as u8).wrapping_add(b'0'),
        ]
    };
    unsafe { std::ptr::copy_nonoverlapping(label.as_ptr(), p, 7) };
}

pub const MARK_COUNT_RVA: usize = rva(0x601830); // dword_601830: the dwords cleared to 0xC
/// The two callers' buffers (`dword_6D160C`, `dword_6D170C`) are 0x40 dwords each.
pub const MARK_BUFFER_DWORDS: u32 = 0x40;

/// `race::split_colours` (0x45B16A): `dword_601830` dwords at edi set to 0xC; then for each
/// of the block esi's +0x90 values at +0x140 (none when +0x90 is 0 or negative), edi's
/// dword 8 when it is not above the one before (0x63 before the first; the sign of the
/// difference, as `cmp`/`jns`), else 4. Leaves ecx the count (+0x90), and after a value eax
/// the last colour and edx the last value. Refuses (before anything) a count past the
/// buffers' 0x40 dwords.
pub unsafe fn split_colours(esi: u32, edi: u32, fill: u32) -> Result<Left, Refused> {
    unsafe {
        let n = get(esi + 0x90);
        if fill > MARK_BUFFER_DWORDS || (n as i32 > 0 && n > MARK_BUFFER_DWORDS) {
            return Err(BadInput("a count past the colour buffer's 0x40 dwords").into());
        }
        for k in 0..fill {
            put(edi + 4 * k, 0xC);
        }
        let left = Left::nothing().with(ECX, n);
        if n as i32 <= 0 {
            return Ok(left);
        }
        let mut edx = 0x63u32;
        let mut eax = 0;
        for k in 0..n {
            let value = get(esi + 0x140 + 4 * k);
            eax = if (edx.wrapping_sub(value) as i32) < 0 {
                4
            } else {
                8
            };
            put(edi + 4 * k, eax);
            edx = value;
        }
        Ok(left.with(EAX, eax).with(EDX, edx))
    }
}

pub const PANEL_FIELDS: [usize; 4] = [rva(0x6D141A), rva(0x6D144A), rva(0x6D1426), rva(0x6D142E)];
const REPLAYING_RVA: usize = rva(0x62389C); // dword_62389C
const PLAYERS_RVA: usize = rva(0x63DB94); // dword_63DB94

/// `race::panel_players` (0x45ACDB): unless `dword_62389C` is set, `dword_6D141A` and
/// `dword_6D144A` -1; with more than one player also `dword_6D1426` and `dword_6D142E` -1,
/// and `dword_6D141A` / `dword_6D144A` the two view blocks' +0xFA4. Leaves eax as it does.
pub unsafe fn panel_players() -> Left {
    unsafe {
        if game_read::<u32>(REPLAYING_RVA) != 0 {
            return Left::nothing();
        }
        game_write(PANEL_FIELDS[0], u32::MAX);
        game_write(PANEL_FIELDS[1], u32::MAX);
        if game_read::<u32>(PLAYERS_RVA) == 1 {
            return Left::nothing();
        }
        game_write(PANEL_FIELDS[2], u32::MAX);
        game_write(PANEL_FIELDS[3], u32::MAX);
        let a = get(game_read::<u32>(VIEW_BLOCKS[0]) + 0xFA4);
        game_write(PANEL_FIELDS[0], a);
        let b = get(game_read::<u32>(VIEW_BLOCKS[1]) + 0xFA4);
        game_write(PANEL_FIELDS[1], b);
        Left::nothing().with(EAX, b)
    }
}

pub const LAMP_COUNT_RVA: usize = rva(0x63DC48); // g_lampCount
pub const LAMPS_WANTED_RVA: usize = rva(0x63DB58); // dword_63DB58

/// `race::lamps_wanted` (0x4536D7): `dword_63DB58` = `g_lampCount`, with bit 0 set when a
/// race car has +0x25C and +0x84 both set. Leaves eax that, ecx the cars (0 when none),
/// esi the last block. Refuses (before anything) more than eight cars (past
/// `g_carBlocks`).
pub unsafe fn lamps_wanted() -> Result<Left, Refused> {
    unsafe {
        let cars: u32 = game_read(CARS_RVA);
        if cars > 8 {
            return Err(BadInput("more than eight race cars").into());
        }
        let mut eax: u32 = game_read(LAMP_COUNT_RVA);
        let mut left = Left::nothing().with(ECX, 0);
        for n in 0..cars {
            let b: u32 = game_read(CAR_BLOCKS_RVA + 4 * n as usize);
            if get(b + 0x25C) != 0 && get(b + 0x84) != 0 {
                eax |= 1;
            }
            left = left.with(ECX, cars).with(ESI, b);
        }
        game_write(LAMPS_WANTED_RVA, eax);
        Ok(left.with(EAX, eax))
    }
}

pub const VIEW_RVA: usize = rva(0x63DB9C); // dword_63DB9C: the view being drawn (0, 1)
pub const VIEW_IN_RVA: usize = rva(0x6415C4); // dword_6415C4
pub const VIEW_OUT_RVA: usize = rva(0x6415D8); // dword_6415D8

/// The view block `dword_63DB9C` names (`off_63DBB0[]`); refuses an index past the two.
unsafe fn drawn_view() -> Result<u32, Refused> {
    unsafe {
        let n: u32 = game_read(VIEW_RVA);
        if n > 1 {
            return Err(BadInput("a view past the two").into());
        }
        Ok(game_read(VIEW_BLOCKS[n as usize]))
    }
}

/// `race::store_view_value` (0x455528): the drawn view's block +0xFB8 = `dword_6415C4`.
/// Leaves eax that, esi the block.
pub unsafe fn store_view_value() -> Result<Left, Refused> {
    unsafe {
        let b = drawn_view()?;
        let v: u32 = game_read(VIEW_IN_RVA);
        put(b + 0xFB8, v);
        Ok(Left::nothing().with(EAX, v).with(ESI, b))
    }
}

/// `race::fetch_view_value` (0x45550E): `dword_6415D8` = the drawn view's block +0xFBC.
/// Every register kept.
pub unsafe fn fetch_view_value() -> Result<Left, Refused> {
    unsafe {
        let b = drawn_view()?;
        game_write(VIEW_OUT_RVA, get(b + 0xFBC));
        Ok(Left::nothing())
    }
}

pub const SPOT_COUNT_RVA: usize = rva(0x6D1DC4); // dword_6D1DC4: the spots made this frame
pub const SPOTS_RVA: usize = rva(0x6D1AF4); // unk_6D1AF4: 36 spots of 0x14 bytes
pub const SHARED_COUNT_RVA: usize = rva(0x5165EC); // dword_5165EC
pub const SHARED_SPOTS_RVA: usize = rva(0x5165F0); // unk_5165F0: 36 of 0x14 (to 0x5168C0)
pub const SPOTS_MAX: u32 = 36;

/// `race::share_sound_spots` (0x45C7B9): `dword_5165EC` = `dword_6D1DC4`, and that many
/// 0x14-byte spots copied from 0x6D1AF4 to 0x5165F0 (read by the sound side,
/// `sub_40E642`). Leaves ecx 0, esi past the spots (0x6D1AF4 with none), edi past the
/// copy. Refuses (before anything) more than 36 (past both tables).
pub unsafe fn share_sound_spots() -> Result<Left, Refused> {
    unsafe {
        let n: u32 = game_read(SPOT_COUNT_RVA);
        if n > SPOTS_MAX {
            return Err(BadInput("more than 36 sound spots").into());
        }
        game_write(SHARED_COUNT_RVA, n);
        let from = crate::framework::game_addr(SPOTS_RVA) as u32;
        let to = crate::framework::game_addr(SHARED_SPOTS_RVA) as u32;
        std::ptr::copy(from as *const u8, to as *mut u8, 0x14 * n as usize);
        Ok(Left::nothing()
            .with(ECX, 0)
            .with(ESI, from + 0x14 * n)
            .with(EDI, to + 0x14 * n))
    }
}

/// `race::clear_point_records` (0x4500FF), on its variables: `dword_5C6FC4` 0, then the
/// 0x1000 point records of 0x20 bytes at `g_pointRecords`: each first dword -1, or with
/// `g_useSse` set each first 16 bytes `xmmword_62DAE0` (`movntps`). Leaves ecx 0, edi
/// past the records.
pub unsafe fn clear_point_records_at(count: u32, sse: u32, records: u32, fill: u32) -> Left {
    unsafe {
        put(count, 0);
        let start = get(records);
        let pattern = (fill as *const [u8; 16]).read_unaligned();
        for k in 0..0x1000u32 {
            let at = start.wrapping_add(0x20 * k);
            if get(sse) != 0 {
                (at as *mut [u8; 16]).write_unaligned(pattern);
            } else {
                put(at, u32::MAX);
            }
        }
        Left::nothing()
            .with(ECX, 0)
            .with(EDI, start.wrapping_add(0x20 * 0x1000))
    }
}

pub const CLEAR_COUNT_RVA: usize = rva(0x5C6FC4); // dword_5C6FC4
pub const USE_SSE_RVA: usize = rva(0x621A58); // g_useSse
pub const POINT_RECORDS_RVA: usize = rva(0x5C6FB8); // g_pointRecords
pub const SSE_FILL_RVA: usize = rva(0x62DAE0); // xmmword_62DAE0

pub unsafe fn clear_point_records() -> Left {
    use crate::framework::game_addr;
    unsafe {
        clear_point_records_at(
            game_addr(CLEAR_COUNT_RVA) as u32,
            game_addr(USE_SSE_RVA) as u32,
            game_addr(POINT_RECORDS_RVA) as u32,
            game_addr(SSE_FILL_RVA) as u32,
        )
    }
}

/// `race::object_bytes`' tables (IDA addresses), in the order the core uses them: the
/// objects' 0x40-byte records `dword_979F50` (+0 the kind, +0xE bit 0), the kinds' bytes
/// `dword_5D3AE8`, the objects' face counts `dword_632F70` and first faces `dword_632B70`,
/// `g_faces`, and the four 0x100-byte outputs `byte_63DC5C`, `byte_63DD5C`, `byte_63DE5C`,
/// `byte_63DF5C`.
pub const OBJECT_TABLES: [usize; 9] = [
    0x979F50, 0x5D3AE8, 0x632F70, 0x632B70, 0x9A3730, 0x63DC5C, 0x63DD5C, 0x63DE5C, 0x63DF5C,
];

/// `race::object_bytes` (0x455263), on tables at `t` (`OBJECT_TABLES`' order): for each of
/// the 0x100 objects, out0 = record +0xE bit 0, out1 = 1, out2 = the kinds' byte of the
/// record's kind, out3 = 0xFF, or with faces the kinds' byte of the first face's +4 word.
/// Leaves ebx 0x100, ecx 0x3FC0, eax the last kind value read.
pub unsafe fn object_bytes_at(t: &[u32]) -> Left {
    unsafe {
        let kind = |n: u32| get(t[1].wrapping_add(n.wrapping_mul(4)));
        let mut eax = 0;
        for n in 0..0x100u32 {
            let record = t[0] + n * 0x40;
            let put8 = |table: u32, v: u8| ((table + n) as *mut u8).write(v);
            put8(t[5], 0);
            if ((record + 0xE) as *const u8).read() & 1 != 0 {
                put8(t[5], 1);
            }
            put8(t[6], 1);
            eax = kind(get(record));
            put8(t[7], eax as u8);
            put8(t[8], 0xFF);
            if get(t[2] + 4 * n) != 0 {
                let face = get(t[4].wrapping_add(get(t[3] + 4 * n).wrapping_mul(4)));
                let word = ((face.wrapping_add(4)) as *const u16).read_unaligned();
                eax = kind(word as u32);
                put8(t[8], eax as u8);
            }
        }
        Left::nothing()
            .with(EBX, 0x100)
            .with(ECX, 0x3FC0)
            .with(EAX, eax)
    }
}

pub fn object_tables() -> Vec<u32> {
    OBJECT_TABLES
        .iter()
        .map(|&a| crate::framework::game_addr(rva(a)) as u32)
        .collect()
}

pub unsafe fn object_bytes() -> Left {
    unsafe { object_bytes_at(&object_tables()) }
}

/// The panel's values (IDA addresses), `race::panel_values`: from 0x6D13FE to 0x6D1472.
pub const PANEL_START_RVA: usize = rva(0x6D13FE);
pub const PANEL_LEN: usize = 0x74;
pub const SCRATCH_RVA: usize = rva(0x62400C); // dword_62400C: the original's scratch dword
const DISTANCES_RVA: usize = rva(0x9F78D8); // g_checkpointDistances
const START_CHECKPOINT_RVA: usize = rva(0x60102C); // dword_60102C
const DISTANCE_UNIT_RVA: usize = rva(0x5C5D98); // flt_5C5D98 (65536.0)
const METRES_IN_MILE_RVA: usize = rva(0x5C5CF0); // g_metresInMile

/// One view's half of `race::panel_values`: block `b`'s values into the panel fields
/// (+0x718 rounded, +0xAB0, +0x1D4, the distance, +0x88 x 10, +0xA0C, -(+0xA10),
/// +0xA14, and for the first view also +0xA18, +0xA1C, +0xA20, +0xA24). Gives eax.
unsafe fn panel_half(b: u32, first: bool) -> u32 {
    use crate::math::x87::to_int;
    use crate::math::x87_load::int_over_two_doubled;
    unsafe {
        let f = |a: u32| crate::framework::game_addr(rva(a as usize)) as u32;
        let (base, distance) = if first {
            (0x6D140E, 0x6D146A)
        } else {
            (0x6D143E, 0x6D146E)
        };
        put(f(base), to_int(get(b + 0x718)) as u32);
        put(f(base + 4), get(b + 0xAB0));
        put(f(base + 8), get(b + 0x1D4));
        let at = |n: u32| game_read::<u32>(DISTANCES_RVA + 4 * n as usize);
        let from: u32 = game_read(START_CHECKPOINT_RVA);
        let d = at(get(b + 0x98)).saturating_sub(at(from)) >> 1;
        game_write(SCRATCH_RVA, d);
        put(
            f(distance),
            int_over_two_doubled(
                d as i32,
                game_read(DISTANCE_UNIT_RVA),
                game_read(METRES_IN_MILE_RVA),
            ),
        );
        let eax = get(b + 0x88).wrapping_mul(10);
        put(f(base + 0x10), eax);
        put(f(base + 0xC), get(b + 0xA0C));
        // -(+0xA10): `dword_6D142A` for the first view, `dword_6D1456` (+0x18) for the second.
        let negated = if first { 0x1C } else { 0x18 };
        put(f(base + negated), get(b + 0xA10).wrapping_neg());
        put(f(base + 0x14), get(b + 0xA14));
        if first {
            put(f(0x6D142E), get(b + 0xA18));
            put(f(0x6D1426), get(b + 0xA1C));
            put(f(0x6D1402), get(b + 0xA20));
            put(f(0x6D13FE), get(b + 0xA24));
        }
        eax
    }
}

/// `race::panel_values` (0x45AB5A): both view blocks' values into the panel (see
/// `panel_half`). Leaves esi the second block, ebx `dword_60102C`, eax its +0x88 x 10.
pub unsafe fn panel_values() -> Left {
    unsafe {
        let a: u32 = game_read(VIEW_BLOCKS[0]);
        panel_half(a, true);
        let b: u32 = game_read(VIEW_BLOCKS[1]);
        let eax = panel_half(b, false);
        Left::nothing()
            .with(ESI, b)
            .with(EBX, game_read(START_CHECKPOINT_RVA))
            .with(EAX, eax)
    }
}

const SOUND_OBJECTS_RVA: usize = rva(0x512A10); // g_raceSoundObjects: 8 pointers
const SHOWN_RVA: usize = rva(0x641680); // dword_641680[8]: each car's view (-1 not shown)

/// `race::wake_part_sounds` (0x45D1EA): for each race car (car 0 at least: the loop tests
/// at its end) and each of its six slots, when +0x48 is 0 and +0x700 at least 10, +0x48 =
/// 1; then, only when +0x700 was exactly 10 (the `jnz` reads the `cmp`'s flags), and when
/// `dword_641680[g_currentCar]` is not -1, the car's sound object (`g_raceSoundObjects[car
/// & 7]`) +0x264 = 1 and +0x10 = 1. Leaves ecx the cars done, ebx 6, esi the last block,
/// edi its sound object, eax the last view read. Refuses (before anything) more than eight
/// cars, a null block or object it would write, and `g_currentCar` past 7 when it is read.
pub unsafe fn wake_part_sounds() -> Result<Left, Refused> {
    unsafe {
        let cars = game_read::<u32>(CARS_RVA).max(1);
        if cars > 8 {
            return Err(BadInput("more than eight race cars").into());
        }
        let current: u32 = game_read(crate::drive::car::CURRENT_CAR_RVA);
        // The plan: (block slot to set, whether to wake the object) by car.
        let mut plan = Vec::new();
        let mut left = Left::nothing();
        for car in 0..cars {
            let b: u32 = game_read(CAR_BLOCKS_RVA + 4 * car as usize);
            let o: u32 = game_read(SOUND_OBJECTS_RVA + 4 * (car & 7) as usize);
            let mut wake = false;
            for slot in 0..6 {
                if b == 0 {
                    return Err(BadInput("a null car block").into());
                }
                let count = get(b + 0x700 + 4 * slot);
                if get(b + 0x48 + 4 * slot) != 0 || count < 10 {
                    continue;
                }
                plan.push(b + 0x48 + 4 * slot);
                if count != 10 {
                    continue;
                }
                if current > 7 {
                    return Err(BadInput("g_currentCar past the eight").into());
                }
                let view: u32 = game_read(SHOWN_RVA + 4 * current as usize);
                left = left.with(EAX, view);
                if view != u32::MAX {
                    if o == 0 {
                        return Err(BadInput("a null sound object").into());
                    }
                    wake = true;
                }
            }
            if wake {
                plan.push(o + 0x264);
                plan.push(o + 0x10);
            }
            left = left.with(ESI, b).with(EDI, o);
        }
        for at in plan {
            put(at, 1);
        }
        Ok(left.with(ECX, cars).with(EBX, 6))
    }
}

fn g(a: u32) -> u32 {
    crate::framework::game_addr(rva(a as usize)) as u32
}

const MODE_RVA: usize = rva(0x62389C); // dword_62389C: the race's mode (2: the cars by place)
const CAR_BY_PLACE: u32 = 0x604EC8; // dword_604EC8[]: the car at each place (from place 0)
const LAST_PLACE_RVA: usize = rva(0x6051B4); // dword_6051B4
const LABEL_BEHIND: u32 = 0x6D1838; // dword_6D1838: 7 bytes
const LABEL_AHEAD: u32 = 0x6D1844; // dword_6D1844: 7 bytes

/// `race::panel_neighbours` (0x45AD36): in mode 2 (`dword_62389C`), the first view's place
/// (+0x654) and its neighbours into the panel: `dword_6D141A` the place, `dword_6D141E`
/// +0x8C x 10, `dword_6D1426` the car behind's +0x24 (-1 at the last place,
/// `dword_6D1426`), its label at 0x6D1838 (`car_label` of `dword_605034[place]`),
/// `dword_6D1422` +0x634, `dword_6D142E` the car ahead's +0x24 (-1 at place 1), its label at
/// 0x6D1844 (`dword_605030[place]`), `dword_6D142A` -(+0x630). Refuses (before anything)
/// a neighbour's car number past 7 or a null block.
pub unsafe fn panel_neighbours() -> Result<Left, Refused> {
    unsafe {
        if game_read::<u32>(MODE_RVA) != 2 {
            return Ok(Left::nothing());
        }
        let a: u32 = game_read(VIEW_BLOCKS[0]);
        let place = get(a + 0x654);
        let table = |base: u32| get(g(base).wrapping_add(place.wrapping_mul(4)));
        let block_of = |car: u32| -> Result<u32, Refused> {
            if car > 7 {
                return Err(BadInput("a neighbour's car past the eight").into());
            }
            let b: u32 = game_read(CAR_BLOCKS_RVA + 4 * car as usize);
            if b == 0 {
                return Err(BadInput("a neighbour's block is null").into());
            }
            Ok(b)
        };
        let behind = block_of(table(CAR_BY_PLACE + 4))?;
        let ahead = block_of(table(CAR_BY_PLACE))?;
        put(g(0x6D141A), place);
        put(g(0x6D141E), get(a + 0x8C).wrapping_mul(10));
        let mut back = get(behind + 0x24);
        if place == game_read::<u32>(LAST_PLACE_RVA) {
            back = u32::MAX;
        }
        put(g(0x6D1426), back);
        car_label(g(LABEL_BEHIND), table(0x605034), back);
        put(g(0x6D1422), get(a + 0x634));
        let mut front = get(ahead + 0x24);
        if place == 1 {
            front = u32::MAX;
        }
        put(g(0x6D142E), front);
        car_label(g(LABEL_AHEAD), table(0x605030), front);
        let eax = get(a + 0x630).wrapping_neg();
        put(g(0x6D142A), eax);
        Ok(Left::nothing()
            .with(EAX, eax)
            .with(EBX, place)
            .with(ECX, front)
            .with(ESI, a)
            .with(EDI, g(LABEL_AHEAD))
            .with(crate::framework::EBP, ahead))
    }
}

const SCENE_CLOCK_RVA: usize = rva(0x613830); // g_sceneClock
const SHOWN_SINCE_RVA: usize = rva(0x62290C); // dword_62290C
const HALF_RVA: usize = rva(0x5C5E40); // flt_5C5E40 (0.5)
const ONE_OVER_65536_RVA: usize = rva(0x5C5DA0); // flt_5C5DA0
const SIXTY_FIVE_K_RVA: usize = rva(0x5C5D98); // flt_5C5D98 (65536.0)

/// `race::panel_motion` (0x45AE1D): for each view block, +0xAE8 and +0xAEC x 0.5 / 65536
/// (floats) and the heading, (-(+0x9C0)) >> 14 x pi x 0.5 / 65536, into the panel
/// (`flt_6D1432`.. for the first, `flt_6D145A`.. for the second); then `dword_62400C` =
/// `g_sceneClock` - `dword_62290C` - 800, 0 when negative. Leaves esi the second block,
/// eax that.
pub unsafe fn panel_motion() -> Left {
    use crate::math::x87_load::{int_pi_times_over, int_times_two};
    unsafe {
        let half: u32 = game_read(HALF_RVA);
        let small: u32 = game_read(ONE_OVER_65536_RVA);
        let big: u32 = game_read(SIXTY_FIVE_K_RVA);
        let mut b = 0;
        for (view, out) in [(0, 0x6D1432), (1, 0x6D145A)] {
            b = game_read::<u32>(VIEW_BLOCKS[view]);
            put(g(out), int_times_two(get(b + 0xAE8) as i32, half, small));
            put(
                g(out + 4),
                int_times_two(get(b + 0xAEC) as i32, half, small),
            );
            let turn = get(b + 0x9C0).wrapping_neg() >> 14;
            game_write(SCRATCH_RVA, turn);
            put(g(out + 8), int_pi_times_over(turn as i32, half, big));
        }
        let mut eax = game_read::<u32>(SCENE_CLOCK_RVA)
            .wrapping_sub(game_read(SHOWN_SINCE_RVA))
            .wrapping_sub(0x320);
        if (eax as i32) < 0 {
            eax = 0;
        }
        game_write(SCRATCH_RVA, eax);
        Left::nothing().with(ESI, b).with(EAX, eax)
    }
}

/// `race::panel_marks` (0x45AEFE): the eight dwords at 0x6D14A2 .. 0x6D14C2 (but 0x6D14B2)
/// 0; for each view block, `dword_6D14B2` / `dword_6D14C6` = 400 + (+0xFA0), and its first
/// +0xF98 (at most 4) values at +0xF78, each + 1, at 0x6D14A2 / 0x6D14B6. Leaves esi the
/// second block, ebp its count (if any), ecx the last count when a loop ran, eax the last
/// value written.
pub unsafe fn panel_marks() -> Left {
    use crate::framework::EBP;
    unsafe {
        for a in [
            0x6D14A2, 0x6D14A6, 0x6D14AA, 0x6D14AE, 0x6D14B6, 0x6D14BA, 0x6D14BE, 0x6D14C2,
        ] {
            put(g(a), 0);
        }
        let mut left = Left::nothing();
        for (view, base, marks) in [(0, 0x6D14B2, 0x6D14A2), (1, 0x6D14C6, 0x6D14B6)] {
            let b: u32 = game_read(VIEW_BLOCKS[view]);
            let mut eax = 0x190u32.wrapping_add(get(b + 0xFA0));
            put(g(base), eax);
            let mut n = get(b + 0xF98);
            left = left.with(ESI, b).with(EBP, n);
            if n != 0 {
                n = n.min(4);
                for k in 0..n {
                    eax = get(b + 0xF78 + 4 * k).wrapping_add(1);
                    put(g(marks) + 4 * k, eax);
                }
                left = left.with(EBP, n).with(ECX, n);
            }
            left = left.with(EAX, eax);
        }
        left
    }
}

const LIGHTS_SHOWN_RVA: usize = rva(0x6D1810); // dword_6D1810: the lights lit (0 .. 5)
const LIGHTS_START_RVA: usize = rva(0x6D1814); // dword_6D1814: the countdown's start

/// `race::countdown_lights` (0x45B008): one view's start lights. `edi` 0 or 0x10 (the
/// view's fields at 0x6D1482 + edi ..), esi the block, ebp where a "step" flag goes.
/// See the header for the steps. Gives the registers it leaves.
pub unsafe fn countdown_lights(entry: &crate::framework::Regs) -> crate::framework::Regs {
    unsafe {
        let mut r = *entry;
        let f = |off: u32| g(0x6D1482).wrapping_add(off).wrapping_add(r.edi);
        let (lit, fade, size, shift) = (f(0), f(4), f(8), f(0xC));
        put(fade, 0);
        put(shift, 0);
        put(r.ebp, 0);
        let b = r.esi;
        if get(b + 0x1C) != 0 {
            return r;
        }
        r.eax = get(b + 0x14).wrapping_sub(game_read(LIGHTS_START_RVA));
        if (r.eax as i32) < 0 {
            return r;
        }
        let t = r.eax.min(0x5DC);
        let (q, rem) = (t / 100, t % 100);
        r.ebx = 100;
        r.edx = rem;
        game_write(LIGHTS_SHOWN_RVA, q.min(5));
        let shown = if q < 6 { q } else { u32::MAX };
        put(lit, shown);
        r.eax = if rem == 0 && shown != u32::MAX {
            put(r.ebp, 1);
            0
        } else {
            rem * 2
        };
        let rest = 0x100 - r.eax;
        if game_read::<u32>(LIGHTS_SHOWN_RVA) == 5 {
            let v = (rest as i32).clamp(0, 0x100);
            put(size, rest);
            let product = v as i64 * 0x80;
            let q2 = (product / 0x30) as i32;
            let r2 = (product % 0x30) as i32;
            put(fade, q2 as u32);
            r.ecx = (v.wrapping_sub(q2) >> 1) as u32;
            put(shift, r.ecx);
            r.eax = q2 as u32;
            r.edx = r2 as u32;
            r.ebx = 0x30;
        } else {
            put(fade, rest);
            put(size, rest);
            put(shift, 0);
        }
        r
    }
}

/// `race::both_countdown_lights` (0x45AFDB): `countdown_lights` for the first view (edi 0,
/// flag `dword_6D1830`) and the second (edi 0x10, `dword_6D1834`).
pub unsafe fn both_countdown_lights(entry: &crate::framework::Regs) -> crate::framework::Regs {
    unsafe {
        let mut r = *entry;
        for (view, edi, flag) in [(0, 0, 0x6D1830), (1, 0x10, 0x6D1834)] {
            r.edi = edi;
            r.esi = game_read(VIEW_BLOCKS[view]);
            r.ebp = g(flag);
            r = countdown_lights(&r);
        }
        r
    }
}

/// Every register as `after` holds it, for a Rust version that gives whole registers.
pub fn all_regs(after: &crate::framework::Regs) -> Left {
    use crate::framework::EBP;
    Left::nothing()
        .with(EAX, after.eax)
        .with(EBX, after.ebx)
        .with(ECX, after.ecx)
        .with(EDX, after.edx)
        .with(ESI, after.esi)
        .with(EDI, after.edi)
        .with(EBP, after.ebp)
}

/// The panel's fields `panel_*` write: 0x6D13FE .. 0x6D14CA.
pub fn panel_regions() -> Vec<crate::service::harness::Region> {
    vec![
        (g(0x6D13FE), 0xCC),
        crate::service::harness::at(SCRATCH_RVA),
    ]
}

/// The countdown's fields: 0x6D1482 .. 0x6D14A2 and `dword_6D1810`.
pub fn light_regions() -> Vec<crate::service::harness::Region> {
    vec![
        (g(0x6D1482), 0x20),
        crate::service::harness::at(LIGHTS_SHOWN_RVA),
    ]
}

pub const LIGHTS_START: usize = LIGHTS_START_RVA;
pub const SCENE_CLOCK: usize = SCENE_CLOCK_RVA;
pub const SHOWN_SINCE: usize = SHOWN_SINCE_RVA;

/// The two views' colour buffers for `split_colours`: `dword_6D160C`, `dword_6D170C`.
pub const COLOUR_BUFFERS: [u32; 2] = [0x6D160C, 0x6D170C];

/// `split_colours`' refusal, for both views, before anything is written.
pub unsafe fn check_split_colours() -> Result<(), Refused> {
    unsafe {
        let fill: u32 = game_read(MARK_COUNT_RVA);
        for view in VIEW_BLOCKS {
            let n = get(game_read::<u32>(view) + 0x90);
            if fill > MARK_BUFFER_DWORDS || (n as i32 > 0 && n > MARK_BUFFER_DWORDS) {
                return Err(BadInput("a count past the colour buffer's 0x40 dwords").into());
            }
        }
        Ok(())
    }
}

/// `race::both_split_colours` (0x45B147): `split_colours` for the first view's block into
/// `dword_6D160C`, then the second's into `dword_6D170C`. Refuses (before anything) as
/// `split_colours` would for either. Gives the registers it leaves.
pub unsafe fn both_split_colours(
    entry: &crate::framework::Regs,
) -> Result<crate::framework::Regs, Refused> {
    unsafe {
        check_split_colours()?;
        let mut r = *entry;
        for (view, buffer) in VIEW_BLOCKS.iter().zip(COLOUR_BUFFERS) {
            r.esi = game_read(*view);
            r.edi = g(buffer);
            split_colours(r.esi, r.edi, game_read(MARK_COUNT_RVA))?.apply(&mut r);
        }
        Ok(r)
    }
}

/// `panel_neighbours`' refusal, before anything is written.
unsafe fn check_panel_neighbours() -> Result<(), Refused> {
    unsafe {
        if game_read::<u32>(MODE_RVA) != 2 {
            return Ok(());
        }
        let place = get(game_read::<u32>(VIEW_BLOCKS[0]) + 0x654);
        for base in [CAR_BY_PLACE, CAR_BY_PLACE + 4] {
            let car = get(g(base).wrapping_add(place.wrapping_mul(4)));
            if car > 7 || game_read::<u32>(CAR_BLOCKS_RVA + 4 * car as usize) == 0 {
                return Err(BadInput("a neighbour's car past the eight or null").into());
            }
        }
        Ok(())
    }
}

/// `race::set_up_panel` (0x45AB36): `panel_values`, `panel_neighbours`, `panel_players`,
/// `panel_motion`, `panel_marks`, `both_countdown_lights`, `both_split_colours`. Refuses
/// (before anything) what `panel_neighbours` or `both_split_colours` would. Gives the
/// registers it leaves.
pub unsafe fn set_up_panel(
    entry: &crate::framework::Regs,
) -> Result<crate::framework::Regs, Refused> {
    unsafe {
        check_panel_neighbours()?;
        check_split_colours()?;
        let mut r = *entry;
        panel_values().apply(&mut r);
        panel_neighbours()?.apply(&mut r);
        panel_players().apply(&mut r);
        panel_motion().apply(&mut r);
        panel_marks().apply(&mut r);
        r = both_countdown_lights(&r);
        both_split_colours(&r)
    }
}

/// `race::reset_lamps` (0x45487A): `g_lampCount`, `dword_63DC4C`, `dword_63DC50` (the sort
/// entries' count) and `dword_64158C` 0, then `race::object_bytes`.
pub unsafe fn reset_lamps() -> Left {
    unsafe {
        for a in [LAMP_COUNT_RVA, rva(0x63DC4C), SORT_COUNT_RVA, rva(0x64158C)] {
            game_write(a, 0u32);
        }
        object_bytes()
    }
}

const LIMIT_RVA: usize = rva(0x71C6D4); // dword_71C6D4

/// `race::hold_to_limit` (0x4650A9): eax held to `dword_71C6D4` (unsigned).
pub fn hold_to_limit(eax: u32) -> u32 {
    eax.min(unsafe { game_read(LIMIT_RVA) })
}

pub const SORT_COUNT_RVA: usize = rva(0x63DC50); // dword_63DC50
pub const SORT_ENTRIES_RVA: usize = rva(0x9DB730); // dword_9DB730: 0x1000 of 8 bytes
pub const SORT_MAX: u32 = 0x1000;

/// `race::add_sort_entry` (0x454D57): entry `dword_63DC50` of `dword_9DB730`: +4 ebx, +0
/// 0, or with (ebx & 0x7FFFFFFF) below 0x1000000 (eax << 16) | bx; the count + 1. Leaves
/// edx the entry, ebx & 0x7FFFFFFF, eax as written (when written), CF clear. Refuses
/// (before anything) an entry past the 0x1000.
pub unsafe fn add_sort_entry(eax: u32, ebx: u32) -> Result<Left, Refused> {
    unsafe {
        let n: u32 = game_read(SORT_COUNT_RVA);
        if n >= SORT_MAX {
            return Err(BadInput("a sort entry past the 0x1000").into());
        }
        let at = g(0x9DB730) + 8 * n;
        put(at, 0);
        put(at + 4, ebx);
        let key = ebx & 0x7FFF_FFFF;
        let mut left = Left::carry(false).with(EDX, n).with(EBX, key);
        if key < 0x100_0000 {
            let v = (eax << 16) | (key & 0xFFFF);
            put(at, v);
            left = left.with(EAX, v);
        }
        game_write(SORT_COUNT_RVA, n.wrapping_add(1));
        Ok(left)
    }
}

/// `race::checkpoint_colours` (0x4548E3): the current checkpoint's (`dword_71A070`) face
/// (`g_checkpointFaces`, records of 0x40 at `g_courseFaces`): its +0x10 byte's low and high
/// nibbles (0 read as 0x10) to `dword_614D08` / `dword_614D0C`; `dword_614D10` =
/// `dword_9ED808[checkpoint]` - `dword_71A998`; `dword_63F17C` the low (twice the distance
/// below 0x40000000 as signed), the high (from 0xC0000000), else the larger. Leaves eax
/// that, ebx the checkpoint, esi the face. The tables are read unchecked, as the original.
pub unsafe fn checkpoint_colours() -> Left {
    unsafe {
        let n: u32 = game_read(rva(0x71A070));
        let face = get(g(0x9E3738).wrapping_add(n.wrapping_mul(4)));
        let esi = (face << 6).wrapping_add(game_read(rva(0x5CCCBC)));
        let byte = ((esi + 0x10) as *const u8).read() as u32;
        let nibble = |v: u32| if v == 0 { 0x10 } else { v };
        let low = nibble(byte & 0xF);
        let high = nibble(byte >> 4 & 0xF);
        game_write(rva(0x614D08), low);
        game_write(rva(0x614D0C), high);
        let d =
            get(g(0x9ED808).wrapping_add(n.wrapping_mul(4))).wrapping_sub(game_read(rva(0x71A998)));
        game_write(rva(0x614D10), d);
        let e = (d as i32).wrapping_abs() as u32;
        let e = e.wrapping_add(e);
        let eax = if (e.wrapping_sub(0x4000_0000) as i32) < 0 {
            low
        } else if (e.wrapping_sub(0xC000_0000) as i32) >= 0 {
            high
        } else {
            low.max(high)
        };
        game_write(rva(0x63F17C), eax);
        Left::nothing().with(EAX, eax).with(EBX, n).with(ESI, esi)
    }
}
