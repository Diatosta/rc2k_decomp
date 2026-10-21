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
    // Converted: `rc2k_core::race::start::car_label`.
    let label = rc2k_core::race::start::car_label(eax, ecx == u32::MAX);
    unsafe { std::ptr::copy_nonoverlapping(label.as_ptr(), edi as *mut u8, 7) };
}

pub const MARK_COUNT_RVA: usize = rva(0x601830); // dword_601830: the dwords cleared to 0xC
/// The two callers' buffers (`dword_6D160C`, `dword_6D170C`) are 0x40 dwords each.
pub const MARK_BUFFER_DWORDS: u32 = rc2k_core::race::frame::COLOUR_BUFFER_DWORDS;

/// `race::split_colours` (0x45B16A): `dword_601830` dwords at edi set to 0xC; then for each
/// of the block esi's +0x90 values at +0x140 (none when +0x90 is 0 or negative), edi's
/// dword 8 when it is not above the one before (0x63 before the first; the sign of the
/// difference, as `cmp`/`jns`), else 4. Leaves ecx the count (+0x90), and after a value eax
/// the last colour and edx the last value. Refuses (before anything) a count past the
/// buffers' 0x40 dwords.
pub unsafe fn split_colours(esi: u32, edi: u32, fill: u32) -> Result<Left, Refused> {
    // Converted: `rc2k_core::race::frame::{check_split_colours, split_colours}`.
    use rc2k_core::race::frame::{SPLIT_UNREACHED, check_split_colours, split_colours};
    unsafe {
        let n = get(esi + 0x90);
        check_split_colours(fill, n)?;
        for k in 0..fill {
            put(edi + 4 * k, SPLIT_UNREACHED);
        }
        let left = Left::nothing().with(ECX, n);
        if n as i32 <= 0 {
            return Ok(left);
        }
        let splits: Vec<u32> = (0..n).map(|k| get(esi + 0x140 + 4 * k)).collect();
        let colours = split_colours(&splits);
        for (k, &c) in colours.iter().enumerate() {
            put(edi + 4 * k as u32, c);
        }
        Ok(left
            .with(EAX, colours[n as usize - 1])
            .with(EDX, splits[n as usize - 1]))
    }
}

pub const PANEL_FIELDS: [usize; 4] = [rva(0x6D141A), rva(0x6D144A), rva(0x6D1426), rva(0x6D142E)];
const REPLAYING_RVA: usize = rva(0x62389C); // dword_62389C
const PLAYERS_RVA: usize = rva(0x63DB94); // dword_63DB94

/// `race::panel_players` (0x45ACDB): unless `dword_62389C` is set, `dword_6D141A` and
/// `dword_6D144A` -1; with more than one player also `dword_6D1426` and `dword_6D142E` -1,
/// and `dword_6D141A` / `dword_6D144A` the two view blocks' +0xFA4. Leaves eax as it does.
pub unsafe fn panel_players() -> Left {
    // Converted: `rc2k_core::race::frame::panel_players`.
    use rc2k_core::race::frame::{PanelPlayers, panel_players};
    unsafe {
        let step = panel_players(game_read(REPLAYING_RVA), || game_read(PLAYERS_RVA));
        if step == PanelPlayers::Kept {
            return Left::nothing();
        }
        game_write(PANEL_FIELDS[0], u32::MAX);
        game_write(PANEL_FIELDS[1], u32::MAX);
        if step == PanelPlayers::One {
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
    // Converted: `rc2k_core::race::frame::{car_lit, lamps_wanted}`.
    use rc2k_core::race::frame::{car_lit, lamps_wanted};
    unsafe {
        let cars: u32 = game_read(CARS_RVA);
        if cars > 8 {
            return Err(BadInput("more than eight race cars").into());
        }
        let mut any_lit = false;
        let mut left = Left::nothing().with(ECX, 0);
        for n in 0..cars {
            let b: u32 = game_read(CAR_BLOCKS_RVA + 4 * n as usize);
            any_lit |= car_lit(get(b + 0x25C), || get(b + 0x84));
            left = left.with(ECX, cars).with(ESI, b);
        }
        let eax = lamps_wanted(game_read(LAMP_COUNT_RVA), any_lit);
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
pub const SPOTS_MAX: u32 = rc2k_core::race::view::MOST_SHARED_SPOTS;

/// `race::share_sound_spots` (0x45C7B9): `dword_5165EC` = `dword_6D1DC4`, and that many
/// 0x14-byte spots copied from 0x6D1AF4 to 0x5165F0 (read by the sound side,
/// `sub_40E642`). Leaves ecx 0, esi past the spots (0x6D1AF4 with none), edi past the
/// copy. Refuses (before anything) more than 36 (past both tables).
pub unsafe fn share_sound_spots() -> Result<Left, Refused> {
    unsafe {
        // Converted: `rc2k_core::race::view::shared_spot_bytes`.
        let n: u32 = game_read(SPOT_COUNT_RVA);
        let bytes = rc2k_core::race::view::shared_spot_bytes(n)?;
        game_write(SHARED_COUNT_RVA, n);
        let from = crate::framework::game_addr(SPOTS_RVA) as u32;
        let to = crate::framework::game_addr(SHARED_SPOTS_RVA) as u32;
        std::ptr::copy(from as *const u8, to as *mut u8, bytes as usize);
        Ok(Left::nothing()
            .with(ECX, 0)
            .with(ESI, from + bytes)
            .with(EDI, to + bytes))
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
        // Converted: `rc2k_core::race::frame::object_bytes`.
        let kind = |n: u32| get(t[1].wrapping_add(n.wrapping_mul(4)));
        let mut eax = 0;
        for n in 0..0x100u32 {
            let record = t[0] + n * 0x40;
            let flags = ((record + 0xE) as *const u8).read();
            eax = kind(get(record));
            let record_kind = eax;
            let face_kind = (get(t[2] + 4 * n) != 0).then(|| {
                let face = get(t[4].wrapping_add(get(t[3] + 4 * n).wrapping_mul(4)));
                let word = ((face.wrapping_add(4)) as *const u16).read_unaligned();
                eax = kind(word as u32);
                eax
            });
            let bytes = rc2k_core::race::frame::object_bytes(flags, record_kind, face_kind);
            for (out, b) in t[5..9].iter().zip(bytes) {
                ((out + n) as *mut u8).write(b);
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
    use rc2k_core::race::panel::{distance_shown, shown_whole, travelled};
    unsafe {
        let f = |a: u32| crate::framework::game_addr(rva(a as usize)) as u32;
        let (base, distance) = if first {
            (0x6D140E, 0x6D146A)
        } else {
            (0x6D143E, 0x6D146E)
        };
        let whole = shown_whole(f32::from_bits(get(b + 0x718)));
        put(f(base), whole as u32);
        put(f(base + 4), get(b + 0xAB0));
        put(f(base + 8), get(b + 0x1D4));
        let at = |n: u32| game_read::<u32>(DISTANCES_RVA + 4 * n as usize);
        let from: u32 = game_read(START_CHECKPOINT_RVA);
        let d = travelled(at(get(b + 0x98)), at(from));
        game_write(SCRATCH_RVA, d);
        let (unit, miles): (u32, u32) =
            (game_read(DISTANCE_UNIT_RVA), game_read(METRES_IN_MILE_RVA));
        let shown = distance_shown(d as i32, f32::from_bits(unit), f32::from_bits(miles)).to_bits();
        put(f(distance), shown);
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

/// `race::panel_values` (0x45AB5A), converted (the hook's and `set_up_panel`'s; the
/// exact x87 version went once its one Rust caller converted): both view blocks' values
/// into the panel (see `panel_half`). Leaves esi the second block, ebx `dword_60102C`, eax
/// its +0x88 x 10.
// Converted: `rc2k_core::race::panel::{shown_whole, travelled, distance_shown}`.
pub unsafe fn panel_values_plain() -> Left {
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

/// The distance shown of each view (the float places `panel_values_plain` writes).
pub const PANEL_FLOATS: [usize; 2] = [rva(0x6D146A), rva(0x6D146E)];

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
                // Converted: `rc2k_core::race::lights::part_wake`.
                use rc2k_core::race::lights::{PartWake, part_wake};
                let count = get(b + 0x700 + 4 * slot);
                let part = part_wake(get(b + 0x48 + 4 * slot), count);
                if part == PartWake::Left {
                    continue;
                }
                plan.push(b + 0x48 + 4 * slot);
                if part != PartWake::WokenNow {
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
        // Converted: `rc2k_core::race::frame::neighbour_number` (and the labels'
        // `rc2k_core::race::start::car_label`).
        use rc2k_core::race::frame::neighbour_number;
        let number = get(behind + 0x24);
        let back = neighbour_number(place == game_read::<u32>(LAST_PLACE_RVA), number);
        put(g(0x6D1426), back);
        car_label(g(LABEL_BEHIND), table(0x605034), back);
        put(g(0x6D1422), get(a + 0x634));
        let front = neighbour_number(place == 1, get(ahead + 0x24));
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

/// `race::panel_motion` (0x45AE1D), converted (the hook's and `set_up_panel`'s): for
/// each view block, +0xAE8 and +0xAEC x 0.5 / 65536 (floats) and the heading,
/// (-(+0x9C0)) >> 14 x pi x 0.5 / 65536, into the panel (`flt_6D1432`.. for the first,
/// `flt_6D145A`.. for the second); then `dword_62400C` = `g_sceneClock` - `dword_62290C` -
/// 800, 0 when negative. Leaves esi the second block, eax that.
// Converted: `rc2k_core::race::panel::{motion, heading_turn, heading, shown_for}`.
pub unsafe fn panel_motion_plain() -> Left {
    use rc2k_core::race::panel::{heading, heading_turn, motion, shown_for};
    unsafe {
        let f = |a: usize| f32::from_bits(game_read(a));
        let (half, small, big) = (f(HALF_RVA), f(ONE_OVER_65536_RVA), f(SIXTY_FIVE_K_RVA));
        let mut b = 0;
        for (view, out) in PANEL_MOTION {
            b = game_read::<u32>(VIEW_BLOCKS[view]);
            put(g(out), motion(get(b + 0xAE8) as i32, half, small).to_bits());
            put(
                g(out + 4),
                motion(get(b + 0xAEC) as i32, half, small).to_bits(),
            );
            let turn = heading_turn(get(b + 0x9C0));
            game_write(SCRATCH_RVA, turn);
            put(g(out + 8), heading(turn as i32, half, big).to_bits());
        }
        let eax = shown_for(game_read(SCENE_CLOCK_RVA), game_read(SHOWN_SINCE_RVA));
        game_write(SCRATCH_RVA, eax);
        Left::nothing().with(ESI, b).with(EAX, eax)
    }
}

/// `panel_motion`'s views and their three floats' places (IDA addresses).
pub const PANEL_MOTION: [(usize, u32); 2] = [(0, 0x6D1432), (1, 0x6D145A)];

/// The float places `panel_motion_plain` writes, within `FLOAT`.
pub fn motion_float_places() -> Vec<(
    crate::service::harness::Region,
    crate::service::floats::Tolerance,
)> {
    PANEL_MOTION
        .iter()
        .flat_map(|&(_, out)| {
            (0..3).map(move |k| {
                (
                    crate::service::harness::at(rva(out as usize + 4 * k)),
                    crate::service::floats::FLOAT,
                )
            })
        })
        .collect()
}

/// `race::panel_marks` (0x45AEFE): the eight dwords at 0x6D14A2 .. 0x6D14C2 (but 0x6D14B2)
/// 0; for each view block, `dword_6D14B2` / `dword_6D14C6` = 400 + (+0xFA0), and its first
/// +0xF98 (at most 4) values at +0xF78, each + 1, at 0x6D14A2 / 0x6D14B6. Leaves esi the
/// second block, ebp its count (if any), ecx the last count when a loop ran, eax the last
/// value written.
pub unsafe fn panel_marks() -> Left {
    // Converted: `rc2k_core::race::frame::view_marks`.
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
            let offset = get(b + 0xFA0);
            let count = get(b + 0xF98);
            let m = rc2k_core::race::frame::view_marks(offset, count, |k| get(b + 0xF78 + 4 * k));
            put(g(base), m.base);
            let mut eax = m.base;
            left = left.with(ESI, b).with(EBP, count);
            if count != 0 {
                for k in 0..m.shown {
                    eax = m.marks[k as usize];
                    put(g(marks) + 4 * k, eax);
                }
                left = left.with(EBP, m.shown).with(ECX, m.shown);
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
        // Converted: `rc2k_core::race::start::lights`; the registers as the original's
        // divisions leave them.
        use rc2k_core::race::start::{lights, steps};
        r.eax = get(b + 0x14).wrapping_sub(game_read(LIGHTS_START_RVA));
        let Some(l) = lights(r.eax) else {
            return r;
        };
        let (_, rem) = steps(r.eax);
        r.ebx = 100;
        r.edx = rem;
        game_write(LIGHTS_SHOWN_RVA, l.shown);
        put(lit, l.lit);
        if l.whole {
            put(r.ebp, 1);
        }
        r.eax = if l.whole { 0 } else { rem * 2 };
        put(size, l.size);
        put(fade, l.fade);
        put(shift, l.shift);
        if l.shown == 5 {
            let v = (l.size as i32).clamp(0, 0x100) as i64 * 0x80;
            r.ecx = l.shift;
            r.eax = l.fade;
            r.edx = (v % 0x30) as u32;
            r.ebx = 0x30;
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
            rc2k_core::race::frame::check_split_colours(fill, n)?;
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
        // Converted: its float callees' `_plain` versions.
        panel_values_plain().apply(&mut r);
        panel_neighbours()?.apply(&mut r);
        panel_players().apply(&mut r);
        panel_motion_plain().apply(&mut r);
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
    // Converted: `rc2k_core::race::frame::hold_to_limit`.
    rc2k_core::race::frame::hold_to_limit(eax, unsafe { game_read(LIMIT_RVA) })
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
        // Converted: `rc2k_core::race::frame::sort_entry`.
        let entry = rc2k_core::race::frame::sort_entry(eax, ebx);
        let at = g(0x9DB730) + 8 * n;
        put(at, entry.first.unwrap_or(0));
        put(at + 4, ebx);
        let mut left = Left::carry(false).with(EDX, n).with(EBX, entry.key);
        if let Some(v) = entry.first {
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
    // Converted: `rc2k_core::race::frame::checkpoint_colours`.
    unsafe {
        let n: u32 = game_read(rva(0x71A070));
        let face = get(g(0x9E3738).wrapping_add(n.wrapping_mul(4)));
        let esi = (face << 6).wrapping_add(game_read(rva(0x5CCCBC)));
        let byte = ((esi + 0x10) as *const u8).read();
        let along = get(g(0x9ED808).wrapping_add(n.wrapping_mul(4)));
        let c = rc2k_core::race::frame::checkpoint_colours(byte, along, game_read(rva(0x71A998)));
        game_write(rva(0x614D08), c.near);
        game_write(rva(0x614D0C), c.far);
        game_write(rva(0x614D10), c.distance);
        game_write(rva(0x63F17C), c.level);
        Left::nothing()
            .with(EAX, c.level)
            .with(EBX, n)
            .with(ESI, esi)
    }
}

/// A scene point's record (`sub_44CF33`, inline in the callers): the record at
/// `g_pointRecords` + (point & 0xFFF) x 0x20, made anew (`scene::make_scene_record`) when
/// its +0 is not the point. Gives the record and, when made, ecx (the point's address).
pub unsafe fn point_record(point: u32) -> Result<(u32, Option<u32>), Refused> {
    unsafe {
        let record = ((point & 0xFFF) << 5).wrapping_add(game_read(POINT_RECORDS_RVA));
        if get(record) == point {
            return Ok((record, None));
        }
        let from = crate::scene::project_point::make_scene_record(point, record)?;
        Ok((record, Some(from)))
    }
}

/// What `point_record` needs to make a record: the records and the points.
pub unsafe fn points_there() -> Result<(), Refused> {
    unsafe {
        if game_read::<u32>(POINT_RECORDS_RVA) == 0
            || game_read::<u32>(crate::scene::project_point::POINTS_RVA) == 0
        {
            return Err(BadInput("no point records or no points").into());
        }
        Ok(())
    }
}

const VIEW_HIDES_RVA: usize = rva(0x606478); // dword_606478[]: by camera view
const SHOWN_VIEW_RVA: usize = rva(0x71A65C); // g_car +0x684, the camera view shown
const HIDING_RVA: usize = rva(0x621480); // dword_621480
const HIDE_MASK_RVA: usize = rva(0x64167C); // dword_64167C
const FACE_X_RVA: usize = rva(0x635004); // dword_635004
const FACE_DEPTH_RVA: usize = rva(0x635010); // dword_635010
const DEPTH_SHIFT_RVA: usize = rva(0x63F188); // flt_63F188

/// `race::face_hidden` (0x454D92), face record `esi`: carry (hidden) when the shown
/// view's `dword_606478` entry is 0, `dword_621480` is set and the face's +2 word has a
/// bit of `dword_64167C` (eax left that); else, inside `pusha`, its first point's record
/// (+4): `dword_635004` = |x| and `dword_635010` = depth + `flt_63F188` as whole numbers,
/// carry when the depth is below |x| (the sign of the difference). Gives eax and the carry.
/// Refuses (before anything) no point records or points.
pub unsafe fn face_hidden(esi: u32, eax: u32) -> Result<(u32, bool), Refused> {
    unsafe {
        // Converted: `rc2k_core::race::faces::{hidden_by_switches, hidden_by_depth}`; the
        // depths' roundings the x87's.
        use rc2k_core::race::faces::{hidden_by_depth, hidden_by_switches};
        let mut eax = eax;
        let view: u32 = game_read(SHOWN_VIEW_RVA);
        let shows: u32 = game_read(VIEW_HIDES_RVA + 4 * view as usize);
        let hiding = if shows == 0 { game_read(HIDING_RVA) } else { 0 };
        let (bits, hidden) = hidden_by_switches(shows, hiding, || {
            ((esi + 2) as *const u16).read_unaligned() as u32 & game_read::<u32>(HIDE_MASK_RVA)
        });
        if let Some(bits) = bits {
            eax = bits;
        }
        if hidden {
            return Ok((eax, true));
        }
        points_there()?;
        let (record, _) = point_record(get(esi + 4))?;
        let x = crate::math::x87_load::abs_to_int(get(record + 0x10));
        game_write(FACE_X_RVA, x);
        let depth =
            crate::math::x87_load::add_to_int(get(record + 0x18), game_read(DEPTH_SHIFT_RVA));
        game_write(FACE_DEPTH_RVA, depth);
        Ok((eax, hidden_by_depth(depth as u32, x as u32)))
    }
}

pub const FACE_KEY_RVA: usize = rva(0x63DC40); // dword_63DC40: the face's depth key
const FACE_SIDE_RVA: usize = rva(0x63DC44); // dword_63DC44

/// `race::face_depth` (0x4552D8): the depth key of face `ebx` (records of 0x40 at
/// `g_courseFaces`) into `dword_63DC40`: for a kind (+0) of 0x10 or more, its first point's
/// depth (`fistp`; and `dword_63DC44` its x); else, with its object's (+1) record +0xE bit
/// 1, the largest of its four points' depths (+4 .. +0x10, each `fistp`, through
/// `dword_62400C`), otherwise their float sum `fistp`'d and shifted right 2. Then + 0x2000000,
/// shifted right 10 (logical). Every register kept. Refuses (before anything) no point
/// records or points.
pub unsafe fn face_depth(ebx: u32) -> Result<(), Refused> {
    use crate::math::x87::{add, to_int};
    unsafe {
        points_there()?;
        let face = (ebx << 6).wrapping_add(game_read(rva(0x5CCCBC)));
        let depth = |k: u32| -> Result<u32, Refused> {
            let (record, _) = point_record(get(face + 4 + 4 * k))?;
            Ok(get(record + 0x18))
        };
        // Converted: `rc2k_core::race::faces::{depth_by, deeper, average_key, depth_key}`;
        // the roundings and the sum the x87's.
        use rc2k_core::race::faces::{DepthBy, average_key, deeper, depth_by};
        let kind = (face as *const u8).read();
        let by = depth_by(kind, || {
            let object = ((face + 1) as *const u8).read() as u32;
            ((g(0x979F5E) + (object << 6)) as *const u8).read()
        });
        let key = match by {
            DepthBy::First => {
                let (record, _) = point_record(get(face + 4))?;
                let key = to_int(get(record + 0x18)) as u32;
                game_write(FACE_KEY_RVA, key);
                game_write(FACE_SIDE_RVA, to_int(get(record + 0x10)) as u32);
                key
            }
            DepthBy::Deepest => {
                let mut key = to_int(depth(0)?) as u32;
                for k in 1..4 {
                    let d = to_int(depth(k)?) as u32;
                    game_write(SCRATCH_RVA, d);
                    key = deeper(key, d);
                }
                key
            }
            DepthBy::Average => {
                let mut sum = depth(0)?;
                for k in 1..3 {
                    sum = add(depth(k)?, sum);
                }
                average_key(crate::math::x87_load::add_to_int(depth(3)?, sum))
            }
        };
        game_write(FACE_KEY_RVA, rc2k_core::race::faces::depth_key(key));
        Ok(())
    }
}

pub const DEPTH_COUNT_RVA: usize = rva(0x63DC4C); // dword_63DC4C
pub const DEPTH_ENTRIES_RVA: usize = rva(0x9D3730); // dword_9D3730: 0x1000 of 8 bytes

/// `race::add_depth_entry` (0x454D2A): `face_depth` of face ebx, then entry
/// `dword_63DC4C` of `dword_9D3730`: +0 the key, +4 ebx; the count + 1. Leaves edx the
/// entry, ecx the key, ebx & 0x7FFFFFFF, CF clear. Refuses (before anything) an entry past
/// the 0x1000, or what `face_depth` refuses.
pub unsafe fn add_depth_entry(ebx: u32) -> Result<Left, Refused> {
    unsafe {
        // Converted: `rc2k_core::race::faces::depth_entry_room`.
        let n: u32 = game_read(DEPTH_COUNT_RVA);
        rc2k_core::race::faces::depth_entry_room(n)?;
        face_depth(ebx)?;
        let key: u32 = game_read(FACE_KEY_RVA);
        let at = g(0x9D3730) + 8 * n;
        put(at, key);
        put(at + 4, ebx);
        game_write(DEPTH_COUNT_RVA, n.wrapping_add(1));
        Ok(Left::carry(false)
            .with(EDX, n)
            .with(ECX, key)
            .with(EBX, ebx & 0x7FFF_FFFF))
    }
}

const GRID_REACH_RVA: usize = rva(0x60A87C); // dword_60A87C: how far the grid reaches
const GRID_HALF_RVA: usize = rva(0x63E15C); // dword_63E15C
const GRID_SPAN_RVA: usize = rva(0x63E160); // dword_63E160
const VIEW_X_RVA: usize = rva(0x6062B0); // dword_6062B0
const VIEW_Z_RVA: usize = rva(0x6062B4); // dword_6062B4
const GRID_X_RVA: usize = rva(0x63E174); // dword_63E174
const GRID_Z_RVA: usize = rva(0x63E178); // dword_63E178
pub const GRID_RVA: usize = rva(0x63E17C); // dword_63E17C[0x400]: 32 x 32 course records
const RECORD_X_RVA: usize = rva(0x5CCBF0); // dword_5CCBF0
const RECORD_Z_RVA: usize = rva(0x5CCBF4); // dword_5CCBF4
const COURSE_RECORDS_RVA: usize = rva(0x5CCCB8); // g_courseRecords (a pointer)
const COURSE_RECORD_COUNT_RVA: usize = rva(0x5CCD18); // g_courseRecordCount

/// `race::course_grid` (0x45497E): the course records around the view in a 32 x 32 grid.
/// The half size `dword_63E15C` = `hold_to_limit(dword_60A87C)`, the span `dword_63E160`
/// twice that; the corner is the view's place (`dword_6062B0` / `dword_6062B4` >> 21,
/// arithmetic) less the half. Unchanged corner: nothing more. Else the corner kept, the
/// grid cleared to 0x0FFFFFFF, and each record (0x20 bytes at `g_courseRecords`, +0 two
/// 11-bit cell numbers, -1 none; each into `dword_5CCBF0` / `dword_5CCBF4`) whose cell less
/// the corner (& 0x7FF) is within the span on both axes goes in at row z, column x. Gives
/// the registers it leaves. Refuses (before anything is written) no records when the grid
/// is to be made (the original's loop would run on: it tests `jnz` after the first).
pub unsafe fn course_grid(
    entry: &crate::framework::Regs,
) -> Result<crate::framework::Regs, Refused> {
    unsafe {
        // Converted: `rc2k_core::race::frame::{hold_to_limit, grid_corner, record_cell,
        // grid_cell}`.
        use rc2k_core::race::frame::{GRID_EMPTY, GRID_SIDE, grid_cell, grid_corner, record_cell};
        let mut r = *entry;
        let half = hold_to_limit(game_read(GRID_REACH_RVA));
        let span = half.wrapping_add(half);
        (r.eax, r.ecx) = grid_corner(game_read(VIEW_X_RVA), game_read(VIEW_Z_RVA), half);
        let unchanged =
            r.eax == game_read::<u32>(GRID_X_RVA) && r.ecx == game_read::<u32>(GRID_Z_RVA);
        let count: u32 = game_read(COURSE_RECORD_COUNT_RVA);
        if !unchanged && count == 0 {
            return Err(BadInput("no course records (the loop would not end)").into());
        }
        game_write(GRID_HALF_RVA, half);
        game_write(GRID_SPAN_RVA, span);
        if unchanged {
            return Ok(r);
        }
        let (gx, gz) = (r.eax, r.ecx);
        game_write(GRID_X_RVA, gx);
        game_write(GRID_Z_RVA, gz);
        let grid = g(0x63E17C);
        for k in 0..GRID_SIDE * GRID_SIDE {
            put(grid + 4 * k, GRID_EMPTY);
        }
        r.edi = grid + 0x1000;
        let records: u32 = game_read(COURSE_RECORDS_RVA);
        for n in 0..count {
            r.esi = (n << 5).wrapping_add(records);
            r.eax = get(r.esi);
            let Some((cx, cz)) = record_cell(r.eax) else {
                continue;
            };
            game_write(RECORD_X_RVA, cx);
            game_write(RECORD_Z_RVA, cz);
            // The registers as the original leaves them at each test.
            let column = cx.wrapping_sub(gx) & 0x7FF;
            let row = cz.wrapping_sub(gz) & 0x7FF;
            r.ecx = cz;
            r.eax = column;
            if column > span {
                continue;
            }
            r.ecx = row;
            if let Some(cell) = grid_cell(cx, cz, (gx, gz), span) {
                r.ecx = cell;
                put(grid.wrapping_add(4 * cell), n);
            }
        }
        r.ebx = count;
        Ok(r)
    }
}

const FACE_TABLES: [u32; 4] = [0x63DC5C, 0x63DD5C, 0x63DE5C, 0x63DF5C]; // `object_bytes`' outputs
pub const CURRENT_FACE_RVA: usize = rva(0x5CCCD0); // dword_5CCCD0

fn byte_at(table: u32, n: u32) -> u32 {
    unsafe { ((g(table) + n) as *const u8).read() as u32 }
}

/// A course record's faces (+8 first, +0xC last; -1 none), or None.
unsafe fn record_faces(record: u32) -> Option<(u32, u32)> {
    unsafe {
        let (first, last) = (get(record + 8), get(record + 0xC));
        (first != u32::MAX && first <= last).then_some((first, last))
    }
}

/// What `gather_record_faces` of these records could refuse, checked before anything:
/// a last face of -1 (the loop would not end), no points, or more faces than either
/// table has room left for.
pub unsafe fn check_gather(records: &[u32]) -> Result<(), Refused> {
    unsafe {
        let mut faces = 0u64;
        for &r in records {
            if let Some((first, last)) = record_faces(r) {
                if last == u32::MAX {
                    return Err(BadInput("a course record's last face is -1").into());
                }
                faces += (last - first) as u64 + 1;
            }
        }
        if faces == 0 {
            return Ok(());
        }
        points_there()?;
        let room =
            |count: usize| SORT_MAX as u64 - (game_read::<u32>(count) as u64).min(SORT_MAX as u64);
        if faces > room(SORT_COUNT_RVA) || faces > room(DEPTH_COUNT_RVA) {
            return Err(BadInput("more faces than the tables have room for").into());
        }
        Ok(())
    }
}

/// `race::gather_record_faces` (0x454C99): each face of course record esi (+8 .. +0xC),
/// `dword_5CCCD0` the face: a kind (+0) of 0xFF, or below 0x10, not `face_hidden`, goes to
/// `add_depth_entry` when its object's flag says so (`byte_63DD5C` / `byte_63DC5C`), else
/// to `add_sort_entry` with the object's kind byte (`byte_63DF5C` / `byte_63DE5C`). For
/// kinds below 0x10 the object is +1, or +0x21 when +1 is 0. Gives the registers it leaves.
/// Refuses (before anything) as `check_gather`.
pub unsafe fn gather_record_faces(
    entry: &crate::framework::Regs,
) -> Result<crate::framework::Regs, Refused> {
    unsafe {
        check_gather(&[entry.esi])?;
        let mut r = *entry;
        r.ebx = get(r.esi + 8);
        let Some((first, _)) = record_faces(r.esi) else {
            return Ok(r);
        };
        let record = r.esi;
        let mut face = first;
        loop {
            game_write(CURRENT_FACE_RVA, face);
            let at = (face << 6).wrapping_add(game_read(rva(0x5CCCBC)));
            r.esi = at;
            r.ebx = face;
            let kind = (at as *const u8).read();
            // `mov al, [esi]`: the kind in eax's low byte.
            r.eax = (r.eax & !0xFF) | kind as u32;
            // Converted: `rc2k_core::race::faces::{gathered, face_object}`.
            use rc2k_core::race::faces::{Gathered, face_object, gathered};
            let how = gathered(kind);
            if how != Gathered::No {
                let (eax, hidden) = face_hidden(at, r.eax)?;
                r.eax = eax;
                if !hidden {
                    let byte = |o: u32| ((at + o) as *const u8).read();
                    r.eax = face_object(how, byte(1), || byte(0x21));
                    let (depth_table, kind_table) = if how == Gathered::Lit {
                        (FACE_TABLES[1], FACE_TABLES[3])
                    } else {
                        (FACE_TABLES[0], FACE_TABLES[2])
                    };
                    let left = if byte_at(depth_table, r.eax) != 0 {
                        add_depth_entry(face)?
                    } else {
                        r.eax = byte_at(kind_table, r.eax);
                        add_sort_entry(r.eax, face)?
                    };
                    left.apply(&mut r);
                }
            }
            r.esi = record;
            face = face.wrapping_add(1);
            r.ebx = face;
            if face > get(record + 0xC) {
                break;
            }
        }
        Ok(r)
    }
}

/// `race::gather_visible_faces` (0x454B71): `gather_record_faces` of each record in
/// `g_courseGrid` (numbers below 0x0FFFFFFF), then of the last record once more (its
/// "already done" test sets ebp inside `pusha`, so it is lost: `notes/bugs.md`). Leaves ecx
/// 0x400, ebp the last record's number. Refuses (before anything) as `check_gather` would
/// for all of them.
pub unsafe fn gather_visible_faces(
    entry: &crate::framework::Regs,
) -> Result<crate::framework::Regs, Refused> {
    unsafe {
        let last = game_read::<u32>(COURSE_RECORD_COUNT_RVA).wrapping_sub(1);
        let todo = visible_records();
        check_gather(&todo)?;
        for &record in &todo {
            let regs = crate::framework::Regs {
                esi: record,
                ..*entry
            };
            gather_record_faces(&regs)?;
        }
        Ok(crate::framework::Regs {
            ecx: 0x400,
            ebp: last,
            ..*entry
        })
    }
}

const GROUND_FACE_RVA: usize = rva(0x71ADD8); // g_car +0xE00: the face under the car

/// `race::face_hide_mask` (0x456056): `physics::probe_ground`, then, unless the face under
/// the car (`g_car` +0xE00) is -1, `dword_64167C` = its +2 word's bits 1 and 3 flipped
/// when any is set (0xA), or'd with bits 4 and 5 likewise (0x30). Gives the registers it
/// leaves.
pub unsafe fn face_hide_mask(
    entry: &crate::framework::Regs,
) -> Result<crate::framework::Regs, Refused> {
    unsafe {
        let mut r = *entry;
        crate::physics::probe_ground::probe_ground(&mut r)?;
        r.ebx = game_read(GROUND_FACE_RVA);
        if r.ebx == u32::MAX {
            return Ok(r);
        }
        r.esi = (r.ebx << 6).wrapping_add(game_read(rva(0x5CCCBC)));
        // Converted: `rc2k_core::race::faces::hide_mask` (ebx the second flip).
        let word = ((r.esi + 2) as *const u16).read_unaligned();
        r.ebx = rc2k_core::race::faces::hide_mask(word & 0x30);
        r.eax = rc2k_core::race::faces::hide_mask(word);
        game_write(HIDE_MASK_RVA, r.eax);
        Ok(r)
    }
}

const TURNED_X_RVA: usize = rva(0x63F180); // flt_63F180
const TURNED_Z_RVA: usize = rva(0x63F184); // flt_63F184
const FAR_LIMIT_RVA: usize = rva(0x624020); // dword_624020: `dword_63F17C` as a float
pub const VIEW_TURN_RVA: usize = rva(0x5EA24C); // g_viewTurnMatrix

/// `race::mark_far_cells` (0x454A5B): every `g_courseGrid` cell's top bits cleared (&
/// 0x0FFFFFFF); then each cell (column x, row z, each less the half `dword_63E15C`) turned
/// by `g_viewTurnMatrix` (into `flt_63F180` / `flt_63F184`, the middle into
/// `dword_62400C`, then its length there): not marked when the length is below 2.0; marked
/// (bit 31) when the turned z is below 0, or when |x / 2| - 2 is not below z, or when the
/// length is not below `dword_63F17C` (as a float, `dword_624020`). Gives the registers it
/// leaves.
pub unsafe fn mark_far_cells(entry: &crate::framework::Regs) -> crate::framework::Regs {
    use crate::math::x87_load::{below_or_unordered, float_hypot, scaled_abs_less_below};
    unsafe {
        let mut r = *entry;
        let grid = g(0x63E17C);
        for k in 0..0x400 {
            put(grid + 4 * k, get(grid + 4 * k) & 0x0FFF_FFFF);
        }
        r.edi = grid + 0x1000;
        let half: u32 = game_read(GRID_HALF_RVA);
        let matrix = crate::framework::game_addr(VIEW_TURN_RVA) as u32;
        let two: u32 = game_read(rva(0x5C5D3C));
        let zero: u32 = game_read(rva(0x5C5D90));
        let point_five: u32 = game_read(HALF_RVA);
        let mut out = [0u32; 8];
        for cell in 0..0x400u32 {
            r.eax = (cell & 0x1F).wrapping_sub(half);
            r.ecx = (cell >> 5).wrapping_sub(half);
            let point = [r.eax as i32, 0, r.ecx as i32];
            crate::scene::project_point::rotate(
                point.as_ptr(),
                matrix as *const u32,
                out.as_mut_ptr(),
            );
            let (x, z) = (out[4], out[6]);
            game_write(TURNED_X_RVA, x);
            game_write(TURNED_Z_RVA, z);
            let length = float_hypot(x, z);
            game_write(SCRATCH_RVA, length);
            let limit = crate::math::x87::from_int(game_read::<u32>(rva(0x63F17C)) as i32);
            game_write(FAR_LIMIT_RVA, limit);
            if below_or_unordered(length, two) {
                continue;
            }
            let mark = below_or_unordered(z, zero)
                || !scaled_abs_less_below(x, point_five, two, z)
                || !below_or_unordered(length, limit);
            if mark {
                put(grid + 4 * cell, get(grid + 4 * cell) | 0x8000_0000);
            }
        }
        r.ebx = 0x400;
        r.esi = matrix;
        r
    }
}

/// `race::prepare_course_view` (0x4548A8): `dword_63F17C` 0x10, `face_hide_mask`,
/// `course_grid`; with `dword_621480` set, `dword_63F17C` 0x10 again, `checkpoint_colours`
/// with two checkpoints or more (`g_checkpointCount`), and `mark_far_cells`. Gives the
/// registers it leaves. Refuses (before anything) what `course_grid` would.
pub unsafe fn prepare_course_view(
    entry: &crate::framework::Regs,
) -> Result<crate::framework::Regs, Refused> {
    unsafe {
        check_course_grid()?;
        // `face_hide_mask` first: its `probe_ground` may refuse, and neither reads
        // `dword_63F17C`, so the 0x10 written before it in the original comes after here.
        let mut r = face_hide_mask(entry)?;
        game_write(rva(0x63F17C), 0x10u32);
        r = course_grid(&r)?;
        if game_read::<u32>(HIDING_RVA) == 0 {
            return Ok(r);
        }
        game_write(rva(0x63F17C), 0x10u32);
        if game_read::<u32>(rva(0x623640)) >= 2 {
            checkpoint_colours().apply(&mut r);
        }
        Ok(mark_far_cells(&r))
    }
}

/// `course_grid`'s refusal, before anything.
unsafe fn check_course_grid() -> Result<(), Refused> {
    unsafe {
        if game_read::<u32>(COURSE_RECORD_COUNT_RVA) == 0 {
            return Err(BadInput("no course records").into());
        }
        Ok(())
    }
}

/// What gathering these records' faces may write: the current face, both tables and
/// their counts, the face tests' dwords, and the scene records of every face's points.
pub fn gather_regions(records: &[u32]) -> Vec<crate::service::harness::Region> {
    use crate::service::harness::at;
    let mut v = vec![
        at(CURRENT_FACE_RVA),
        at(SORT_COUNT_RVA),
        (g(0x9DB730), 0x8000),
        at(DEPTH_COUNT_RVA),
        (g(0x9D3730), 0x8000),
        at(FACE_X_RVA),
        at(FACE_DEPTH_RVA),
        at(FACE_KEY_RVA),
        at(FACE_SIDE_RVA),
        at(SCRATCH_RVA),
    ];
    unsafe {
        let faces: u32 = game_read(rva(0x5CCCBC));
        if faces == 0 || game_read::<u32>(POINT_RECORDS_RVA) == 0 {
            return v;
        }
        let mut seen = std::collections::BTreeSet::new();
        for &r in records {
            let Some((first, last)) = record_faces(r) else {
                continue;
            };
            if last == u32::MAX || last - first > 0x1000 {
                continue;
            }
            for face in first..=last {
                let at = (face << 6).wrapping_add(faces);
                for k in 0..4 {
                    let point = get(at + 4 + 4 * k);
                    if seen.insert(point & 0xFFF | (point >> 31) << 12) {
                        v.extend(crate::scene::project_point::regions(point));
                    }
                }
            }
        }
    }
    v
}

/// The records `gather_visible_faces` gathers, as it would.
pub fn visible_records() -> Vec<u32> {
    unsafe {
        // Converted: `rc2k_core::race::view::visible_records`.
        let records: u32 = game_read(COURSE_RECORDS_RVA);
        let grid = g(0x63E17C);
        rc2k_core::race::view::visible_records(
            (0..0x400).map(|k| get(grid + 4 * k)),
            game_read(COURSE_RECORD_COUNT_RVA),
        )
        .into_iter()
        .map(|n| (n << 5).wrapping_add(records))
        .collect()
    }
}

/// `prepare_course_view`'s and `mark_far_cells`' other writes: the grid (and past it),
/// the turned cell, the limit, `dword_63F17C`'s neighbours.
pub fn far_regions() -> Vec<crate::service::harness::Region> {
    use crate::service::harness::at;
    vec![
        (g(0x63E17C), 0x1084),
        at(TURNED_X_RVA),
        at(TURNED_Z_RVA),
        at(FAR_LIMIT_RVA),
        at(SCRATCH_RVA),
    ]
}

pub const SHOWN_CAR_RVA: usize = rva(0x63DBA4); // dword_63DBA4: the drawn view's car

/// `race::car_light_modes` (0x455B70): `dword_63DBA4` = the drawn view's car
/// (`dword_63DBA8[g_drawnView]`); every car block's +0x6FC 0; then for each race car with
/// lights (+0x25C) that is drawn (any car in modes 0 and 2, else the shown car only; the
/// shown car not with +0x69C and +0x684 both 0, nor with `dword_63993C` set): +0x6FC's
/// nibbles set step by step: 0x21; for the shown car with `dword_606274`, 0x12, then 0x14
/// unless `dword_606270` is 1, then 0x11 unless it is 2 or 3; then 0x100 in the third
/// nibble, 0x200 with `dword_6238B4` (1: the shown car only). Gives the registers it
/// leaves. Refuses (before anything) a null block among the eight or more than eight cars.
pub unsafe fn car_light_modes(
    entry: &crate::framework::Regs,
) -> Result<crate::framework::Regs, Refused> {
    unsafe {
        let block = |n: u32| game_read::<u32>(CAR_BLOCKS_RVA + 4 * n as usize);
        let cars: u32 = game_read(CARS_RVA);
        if (0..8).any(|n| block(n) == 0) {
            return Err(BadInput("a null car block").into());
        }
        if cars > 8 {
            return Err(BadInput("more than eight race cars").into());
        }
        let mut r = *entry;
        let view: u32 = game_read(VIEW_RVA);
        r.ebx = get(g(0x63DBA8).wrapping_add(view.wrapping_mul(4)));
        let shown = r.ebx;
        game_write(SHOWN_CAR_RVA, shown);
        for n in 0..8 {
            r.esi = block(n);
            put(r.esi + 0x6FC, 0);
        }
        r.ebx = 8;
        if cars == 0 {
            return Ok(r);
        }
        // Converted: `rc2k_core::race::lights::{lights_drawn, light_word}`.
        use rc2k_core::race::lights::{light_word, lights_drawn};
        let mode: u32 = game_read(MODE_RVA);
        for n in 0..cars {
            r.ebx = n;
            r.esi = block(n);
            let b = r.esi;
            let at = b + 0x6FC;
            let me = n == shown;
            let drawn = lights_drawn(
                get(b + 0x25C) != 0,
                mode,
                me,
                || get(b + 0x69C) == 0 && get(b + 0x684) == 0,
                || game_read::<u32>(rva(0x63993C)) != 0,
            );
            if !drawn {
                put(at, 0);
                continue;
            }
            let word = light_word(
                get(at),
                me,
                || game_read::<u32>(rva(0x606274)) != 0,
                || game_read(rva(0x606270)),
                game_read(rva(0x6238B4)),
            );
            put(at, word);
        }
        r.ebx = cars;
        Ok(r)
    }
}

const LIT_FACE_COUNT_RVA: usize = rva(0x6D3620); // g_litFaceCount
const LIT_FACES: u32 = 0x6D3220; // g_litFaces: 0x100 face numbers
const LAMPS: u32 = 0x63E05C; // g_lamps: 16 of 0x10 bytes
const LAMP_TURNED_RVA: usize = rva(0x63501C); // dword_63501C .. dword_635024
const LAMP_ENTRIES_RVA: usize = rva(0x641590); // dword_641590

/// `race::add_lamp_entry` (0x4573FA): lamp ebx's point (`g_lamps` +0) through
/// `g_viewMatrix` (the point into `dword_63501C` .. `dword_635024`, then x and y turned
/// there, z rounded into `g_faceDepthKey`, + 0x2000000 and shifted right 10), and a depth
/// entry of that key with `g_lampCount` + 0x3000000; `dword_641590` + 1. Leaves ebx the
/// entry's second word, ecx the key, edx the entry, esi the point, ebp the lamp, CF clear.
/// The caller checks the room.
unsafe fn add_lamp_entry(lamp: u32) -> Left {
    use crate::framework::EBP;
    unsafe {
        let ebp = g(LAMPS) + (lamp << 4);
        let point = get(ebp);
        let esi = point
            .wrapping_mul(12)
            .wrapping_add(game_read(rva(0x5CCCC0)));
        let place = [get(esi), get(esi + 4), get(esi + 8)];
        let turned = crate::framework::game_addr(LAMP_TURNED_RVA) as u32;
        for (k, &v) in place.iter().enumerate() {
            put(turned + 4 * k as u32, v);
        }
        let mut out = [0u32; 3];
        crate::math::x87_load::rotate_rounding_z(
            place.as_ptr() as *const i32,
            crate::framework::game_addr(crate::scene::project_point::MATRIX_RVA) as *const u32,
            out.as_mut_ptr(),
        );
        put(turned, out[0]);
        put(turned + 4, out[1]);
        let key = out[2].wrapping_add(0x200_0000) >> 10;
        game_write(FACE_KEY_RVA, key);
        let second = game_read::<u32>(LAMP_COUNT_RVA).wrapping_add(0x300_0000);
        let n: u32 = game_read(DEPTH_COUNT_RVA);
        put(g(0x9D3730) + 8 * n, key);
        put(g(0x9D3730) + 8 * n + 4, second);
        game_write(DEPTH_COUNT_RVA, n.wrapping_add(1));
        game_write(
            LAMP_ENTRIES_RVA,
            game_read::<u32>(LAMP_ENTRIES_RVA).wrapping_add(1),
        );
        Left::carry(false)
            .with(EBX, second)
            .with(ECX, key)
            .with(EDX, n)
            .with(ESI, esi)
            .with(EBP, ebp)
    }
}

/// `race::gather_lamps` (0x454E08): `g_lampCount` 0; each lit face (`g_litFaces`, the
/// count `g_litFaceCount`) whose object (+1) is 0x3B becomes a lamp while there are fewer
/// than 16: `g_lamps` entry (+0 the face's first point, +4 its +0x11 byte, +8 0xFFFFFF) and
/// its depth entry (`add_lamp_entry`, inside `pusha`). Gives the registers it leaves.
/// Refuses (before anything) more than 0x100 lit faces, or a depth list without room for 16.
pub unsafe fn gather_lamps(
    entry: &crate::framework::Regs,
) -> Result<crate::framework::Regs, Refused> {
    unsafe {
        let count: u32 = game_read(LIT_FACE_COUNT_RVA);
        if count > 0x100 {
            return Err(BadInput("more than 0x100 lit faces").into());
        }
        if count != 0 && game_read::<u32>(DEPTH_COUNT_RVA) > SORT_MAX - 16 {
            return Err(BadInput("the depth list has no room for the lamps").into());
        }
        let mut r = *entry;
        game_write(LAMP_COUNT_RVA, 0u32);
        if count == 0 {
            return Ok(r);
        }
        for k in 0..count {
            r.ecx = k;
            r.ebx = get(g(LIT_FACES) + 4 * k);
            r.esi = (r.ebx << 6).wrapping_add(game_read(rva(0x5CCCBC)));
            // Converted: `rc2k_core::race::faces::takes_lamp`.
            r.eax = ((r.esi + 1) as *const u8).read() as u32;
            if r.eax != rc2k_core::race::faces::LAMP_OBJECT {
                continue;
            }
            r.ebx = game_read(LAMP_COUNT_RVA);
            if !rc2k_core::race::faces::takes_lamp(r.eax, r.ebx) {
                continue;
            }
            let lamp = r.ebx;
            // ebx stays shifted (`shl ebx, 4`; the call is inside `pusha` / `popa`).
            r.ebx = lamp << 4;
            r.edi = g(LAMPS) + r.ebx;
            put(r.edi, get(r.esi + 4));
            put(r.edi + 4, ((r.esi + 0x11) as *const u8).read() as u32);
            // +0x2C goes in first, then 0xFFFFFF over it; eax keeps +0x2C.
            r.eax = get(r.esi + 0x2C);
            put(r.edi + 8, rc2k_core::race::faces::LAMP_COLOUR);
            add_lamp_entry(lamp);
            game_write(LAMP_COUNT_RVA, lamp + 1);
        }
        r.ecx = count;
        Ok(r)
    }
}

/// `race::sort_sort_entries` (0x454F62) / `race::sort_depth_entries` (0x454ED4): with two
/// entries or more, the list's quicksort (`sub_454F85` / its twin `sub_454EF7`: 8-byte
/// entries by their first dword, the largest first) from its first entry to its last.
/// Gives the registers it leaves. Refuses (before anything) a sort that would go past the
/// list.
pub unsafe fn sort_entries(
    entry: &crate::framework::Regs,
    count: usize,
    entries: u32,
) -> Result<crate::framework::Regs, Refused> {
    // Converted: `rc2k_core::race::quick_sort::quick_sort` (through `sort_records`).
    use rc2k_core::race::quick_sort::Order;
    unsafe {
        let mut r = *entry;
        let n: u32 = game_read(count);
        if n < 2 {
            return Ok(r);
        }
        r.esi = g(entries);
        r.eax = n.wrapping_mul(8);
        r.edi = r.eax.wrapping_add(r.esi).wrapping_sub(8);
        crate::race::sorting::sort_records::<2>(&mut r, 0, Order::Down)?;
        Ok(r)
    }
}

pub const EAX_ENVIRONMENT_RVA: usize = rva(0x6D1DD4); // g_eaxEnvironment (a float)
pub const EAX_FACES_RVA: usize = rva(0x6D1AF0); // dword_6D1AF0: the faces summed
const ROOT_TABLE: u32 = 0x5C70F8; // g_rootTable: 256 floats by mantissa bits
const EXPONENT_ROOT_TABLE: u32 = 0x5C74F8; // g_exponentRootTable: 256 floats by exponent

/// The game's table root of a float (`sqrt` by `g_rootTable` x `g_exponentRootTable`),
/// times 1/65536 (`flt_5C5DA0`), as stored.
fn table_root(v: u32) -> u32 {
    let root = get_f(g(ROOT_TABLE) + 4 * ((v >> 15) & 0xFF));
    let scale = get_f(g(EXPONENT_ROOT_TABLE) + 4 * ((v & 0x7F80_0000) >> 23));
    crate::math::x87_load::product3(root, scale, unsafe { game_read(ONE_OVER_65536_RVA) })
}

fn get_f(a: u32) -> u32 {
    unsafe { get(a) }
}

/// A point's x and z less car 0's (`g_carBlockStore` +0x134, +0x13C), and its y.
unsafe fn point_from_car(point: u32) -> (u32, u32, u32) {
    unsafe {
        let p = point
            .wrapping_mul(12)
            .wrapping_add(game_read(rva(0x5CCCC0)));
        let x = get(p).wrapping_sub(game_read(rva(0x70F520)));
        let z = get(p + 8).wrapping_sub(game_read(rva(0x70F528)));
        game_write(rva(0x6D1DE4), x);
        game_write(rva(0x6D1DEC), z);
        (x, get(p + 4), z)
    }
}

/// The weight's end: `dword_62400C` = `v`, held to 16.0 (`flt_5C5D08`; kept when below or
/// not a number), added to `g_eaxEnvironment`; `dword_6D1AF0` + 1.
unsafe fn add_eax_weight(v: u32) {
    unsafe {
        let sixteen: u32 = game_read(rva(0x5C5D08));
        let w = if crate::math::x87_load::below_or_unordered(v, sixteen) {
            v
        } else {
            sixteen
        };
        game_write(SCRATCH_RVA, w);
        game_write(
            EAX_ENVIRONMENT_RVA,
            crate::math::x87::add(w, game_read(EAX_ENVIRONMENT_RVA)),
        );
        game_write(
            EAX_FACES_RVA,
            game_read::<u32>(EAX_FACES_RVA).wrapping_add(1),
        );
    }
}

/// `race::eax_face_weight` (0x45C881): face ebx's weight in the EAX sound environment
/// (`g_eaxEnvironment`), by its distance from car 0 (x and z): kind 0xFF with an object
/// (+1) below 0x38: its +0x18 byte over the distance to its first point; kinds below 0x10
/// with |+0x30| below 0.25: 3.0 over the distance to the middle of its second and third
/// points (`flt_6D1DD8`, `flt_6D1DE0`); each held to 16 (see `add_eax_weight`). Distances
/// by the game's root tables into `flt_6D1DF0`. Gives eax, ebx, ecx, esi as it leaves them.
pub unsafe fn eax_face_weight(face: u32, entry: &crate::framework::Regs) -> crate::framework::Regs {
    use crate::math::x87::{div, from_int};
    use crate::math::x87_load::{
        abs_below_or_unordered, float_square_sum, int_over_float, int_plus_times, int_square_sum,
    };
    unsafe {
        let mut r = *entry;
        r.esi = (face << 6).wrapping_add(game_read(rva(0x5CCCBC)));
        let kind = (r.esi as *const u8).read();
        if kind == 0xFF {
            if ((r.esi + 1) as *const u8).read() >= 0x38 {
                return r;
            }
            let (x, y, z) = point_from_car(get(r.esi + 4));
            r.ebx = y;
            r.ecx = z;
            let distance = table_root(int_square_sum(x as i32, z as i32));
            game_write(rva(0x6D1DF0), distance);
            r.eax = ((r.esi + 0x18) as *const u8).read() as u32;
            game_write(SCRATCH_RVA, r.eax);
            add_eax_weight(int_over_float(r.eax as i32, distance));
            return r;
        }
        if kind >= 0x10 {
            return r;
        }
        if !abs_below_or_unordered(get(r.esi + 0x30), game_read(rva(0x5C5DA4))) {
            return r;
        }
        let (x1, _, z1) = point_from_car(get(r.esi + 8));
        let (mx, mz) = (from_int(x1 as i32), from_int(z1 as i32));
        game_write(rva(0x6D1DD8), mx);
        game_write(rva(0x6D1DE0), mz);
        let (x2, y2, z2) = point_from_car(get(r.esi + 0xC));
        let half: u32 = game_read(HALF_RVA);
        let mx = int_plus_times(x2 as i32, mx, half);
        game_write(rva(0x6D1DD8), mx);
        let mz = int_plus_times(z2 as i32, mz, half);
        game_write(rva(0x6D1DE0), mz);
        r.ebx = y2;
        r.ecx = z2;
        // The root's `fstp [esp]` lands on the pushed eax, so `pop eax` gives the square
        // sum's bits: eax leaves with them.
        let sum = float_square_sum(mx, mz);
        r.eax = sum;
        let distance = table_root(sum);
        game_write(rva(0x6D1DF0), distance);
        add_eax_weight(div(game_read(rva(0x5C5D40)), distance));
        r
    }
}

/// `race::eax_environment` (0x45C7EE): `g_eaxEnvironment` 0.0 and `dword_6D1AF0` 0, then
/// the 3 x 3 grid cells around the middle (from `hold_to_limit(dword_60A87C)` - 1 on each
/// axis; `sub_45C84C` each): every face of each cell's course record (numbers below
/// 0x0FFFFFFF) weighed by `eax_face_weight` (inside `pusha`). Gives the registers it
/// leaves.
pub unsafe fn eax_environment(entry: &crate::framework::Regs) -> crate::framework::Regs {
    unsafe {
        let mut r = *entry;
        game_write(EAX_ENVIRONMENT_RVA, 0u32);
        game_write(EAX_FACES_RVA, 0u32);
        // Converted: `rc2k_core::race::view::{cells_from, record_faces}`.
        use rc2k_core::race::view::{NO_RECORD, cells_from, record_faces};
        r.eax = hold_to_limit(game_read(GRID_REACH_RVA));
        for cell in cells_from(r.eax.wrapping_sub(1), 3) {
            r.ecx = cell;
            // `sub_45C84C`.
            r.ebx = get(g(0x63E17C).wrapping_add(r.ecx.wrapping_mul(4)));
            if r.ebx >= NO_RECORD {
                continue;
            }
            r.esi = (r.ebx << 5).wrapping_add(game_read(COURSE_RECORDS_RVA));
            r.ebx = get(r.esi + 8);
            if record_faces(r.ebx, get(r.esi + 0xC)).is_none() {
                continue;
            }
            loop {
                eax_face_weight(r.ebx, &r);
                r.ebx = r.ebx.wrapping_add(1);
                if r.ebx > get(r.esi + 0xC) {
                    break;
                }
            }
        }
        r
    }
}

const HAS_MMX_RVA: usize = rva(0x5183A8); // g_hasMmx
pub const LIGHT_COLOUR_RVA: usize = rva(0x62DAC4); // g_lightColour
pub const LIT_PLACE_RVA: usize = rva(0x636588); // g_litPlace, dword_63658C, dword_636590
const CAR_LIGHT_TABLE: u32 = 0x635D18; // g_carLightTable: 256 colours
const TURNED_LIGHT_RVA: usize = rva(0x6365B8); // dword_6365B8 .. dword_6365C0

/// Whether the game uses MMX (`g_hasMmx`).
pub fn has_mmx() -> bool {
    unsafe { game_read::<u32>(HAS_MMX_RVA) != 0 }
}

/// Two colours added byte by byte (`rc2k_core::race::lights::add_colours`), MMX or not
/// as the game.
pub fn add_colours(a: u32, b: u32) -> u32 {
    rc2k_core::race::lights::add_colours(a, b, has_mmx())
}

/// `sub_44F691`: a car's (block `b`) headlights on the lit place, added to
/// `g_lightColour` (see `light_colour_at`). Called inside `pusha`.
unsafe fn car_light_on_point(b: u32) {
    use crate::math::x87_load::{int_square_sum_scaled_to_int, rotate_rounding};
    use rc2k_core::race::lights::headlight_index;
    unsafe {
        if get(b + 0x84) == 0 {
            return;
        }
        let place = g(0x636588);
        let d = [
            get(place).wrapping_sub(get(b + 0x134)),
            get(place + 4).wrapping_sub(get(b + 0x138)),
            get(place + 8).wrapping_sub(get(b + 0x13C)),
        ];
        let mut t = [0u32; 3];
        rotate_rounding(
            d.as_ptr() as *const i32,
            (b + 0x9CC) as *const u32,
            t.as_mut_ptr(),
        );
        let turned = crate::framework::game_addr(TURNED_LIGHT_RVA) as u32;
        for (k, &v) in t.iter().enumerate() {
            put(turned + 4 * k as u32, v);
        }
        let k: u32 = game_read(ONE_OVER_65536_RVA);
        let s1 = int_square_sum_scaled_to_int(
            t[2].wrapping_sub(0x20_0000) as i32,
            t[0].wrapping_add(t[0]) as i32,
            k,
            2,
        );
        // The second spread is worked out and then not used (eax is loaded over it).
        let s2 =
            int_square_sum_scaled_to_int(t[2].wrapping_sub(0x10_0000) as i32, t[0] as i32, k, 1);
        game_write(SCRATCH_RVA, s2);
        let index = headlight_index(s1, t[1], get(b + 0xABC));
        let colour = get(g(CAR_LIGHT_TABLE) + 4 * index);
        game_write(
            LIGHT_COLOUR_RVA,
            add_colours(colour, game_read(LIGHT_COLOUR_RVA)),
        );
    }
}

/// `sub_44F601`: the cars' headlights on the lit place: with `dword_52A910` the chosen car
/// (`dword_52A920`) or all with `dword_6238B8` 1; else the players' cars (all with
/// `dword_6238B8` 1), only the drawn view's car outside one player and modes 0 and 2.
unsafe fn car_lights_on_point(r: &mut crate::framework::Regs) {
    unsafe {
        let cars: u32 = game_read(CARS_RVA);
        let block = |n: u32| game_read::<u32>(CAR_BLOCKS_RVA + 4 * n as usize);
        let all = game_read::<u32>(rva(0x6238B8)) == 1;
        let one = game_read::<u32>(rva(0x52A910)) != 0;
        for n in 0..cars {
            r.ecx = n;
            r.esi = block(n);
            let lit = rc2k_core::race::lights::headlights_light(
                n,
                one,
                || game_read(rva(0x52A920)),
                all,
                || game_read(PLAYERS_RVA),
                || game_read(MODE_RVA),
                || {
                    r.eax = game_read(VIEW_RVA);
                    get(g(0x63DBA8).wrapping_add(r.eax.wrapping_mul(4)))
                },
            );
            if lit {
                car_light_on_point(r.esi);
            }
        }
        r.ecx = cars;
    }
}

/// `sub_44F863`: the lamps' light on the lit place: each lamp with a strength (+4,
/// `dword_639958`) gives ((0x200000 - its approximate distance) x strength) >> 17 (held to
/// 0 .. 0xFF) as a grey, times its colour (+8, `dword_62DA7C`) byte by byte, added to
/// `g_lightColour`.
unsafe fn lamp_lights_on_point(r: &mut crate::framework::Regs) {
    unsafe {
        let count: u32 = game_read(LAMP_COUNT_RVA);
        if count == 0 {
            return;
        }
        let place = g(0x636588);
        for n in 0..count {
            let lamp = g(LAMPS) + (n << 4);
            r.eax = get(lamp + 4);
            game_write(rva(0x639958), r.eax);
            if r.eax == 0 {
                continue;
            }
            let tint = get(lamp + 8);
            game_write(rva(0x62DA7C), tint);
            r.esi = get(lamp)
                .wrapping_mul(12)
                .wrapping_add(game_read(rva(0x5CCCC0)));
            let d = [
                get(r.esi).wrapping_sub(get(place)),
                get(r.esi + 4).wrapping_sub(get(place + 4)),
                get(r.esi + 8).wrapping_sub(get(place + 8)),
            ];
            let [distance, ebx, ecx] = crate::scene::approximate_distance::approximate_distance(d);
            r.ebx = ebx;
            r.ecx = ecx;
            game_write(SCRATCH_RVA, distance);
            let (grey, edx) =
                rc2k_core::race::lights::lamp_grey(distance, game_read(rva(0x639958)));
            r.edx = edx;
            r.eax = add_colours(
                rc2k_core::race::lights::multiply_colours(grey, tint),
                game_read(LIGHT_COLOUR_RVA),
            );
            game_write(LIGHT_COLOUR_RVA, r.eax);
        }
        r.ebx = count;
    }
}

/// What `light_colour_at` refuses, before anything: no cars (its loop tests at the end),
/// more than eight, a null block it would read, more than 16 lamps.
pub unsafe fn check_light_colour() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::lights::check_light_colour`.
    unsafe {
        Ok(rc2k_core::race::lights::check_light_colour(
            game_read(CARS_RVA),
            |n| game_read(CAR_BLOCKS_RVA + 4 * n as usize),
            || game_read(LAMP_COUNT_RVA),
        )?)
    }
}

/// `race::light_colour_at` (0x4537CE): `g_lightColour` 0, then the cars' headlights
/// (`sub_44F601`) and the lamps (`sub_44F863`) on the lit place (`g_litPlace`,
/// `dword_63658C`, `dword_636590`), added up. Gives the registers it leaves. Refuses as
/// `check_light_colour`.
pub unsafe fn light_colour_at(
    entry: &crate::framework::Regs,
) -> Result<crate::framework::Regs, Refused> {
    // Converted: `rc2k_core::race::lights::{check_light_colour, headlights_light,
    // headlight_index, lamp_grey, add_colours, multiply_colours}` (the turned place, the
    // spreads and the lamps' distances the x87's and the scene's).
    unsafe {
        check_light_colour()?;
        let mut r = *entry;
        game_write(LIGHT_COLOUR_RVA, 0u32);
        car_lights_on_point(&mut r);
        lamp_lights_on_point(&mut r);
        Ok(r)
    }
}

/// `race::view_light` (0x455896): the lit place the work car's (`g_car` +0x134 ..);
/// `light_colour_at`; `dword_6415E4` = `dword_62DAD4` plus the light (`add_colours`, through
/// `dword_62400C`). Gives the registers it leaves.
pub unsafe fn view_light(
    entry: &crate::framework::Regs,
) -> Result<crate::framework::Regs, Refused> {
    // Converted: `rc2k_core::race::lights::add_colours` (through `light_colour_at`'s).
    unsafe {
        check_light_colour()?;
        let mut r = *entry;
        let car = g(0x71A10C);
        r.eax = get(car);
        r.ebx = get(car + 4);
        r.ecx = get(car + 8);
        game_write(LIT_PLACE_RVA, r.eax);
        game_write(rva(0x63658C), r.ebx);
        game_write(rva(0x636590), r.ecx);
        r = light_colour_at(&r)?;
        let light: u32 = game_read(LIGHT_COLOUR_RVA);
        game_write(SCRATCH_RVA, light);
        r.eax = add_colours(game_read(rva(0x62DAD4)), light);
        game_write(rva(0x6415E4), r.eax);
        Ok(r)
    }
}

/// `race::draw_distance_settings` (0x43932A): the draw distance's level (`dword_60AC84`,
/// held to 0 ..= 4 by sign) picks from six tables: `dword_60A87C` (at least 1, by sign),
/// `dword_60930C`, `dword_609358`, `dword_609308`, `dword_609320`, `dword_60AC80`. Leaves
/// ebx the level, eax the last table's value.
pub unsafe fn draw_distance_settings() -> Left {
    // Converted: `rc2k_core::race::view::{draw_distance_level, grid_reach}`.
    use rc2k_core::race::view::{draw_distance_level, grid_reach};
    unsafe {
        let level = draw_distance_level(game_read(rva(0x60AC84)));
        let pick = |table: u32| get(g(table) + 4 * level);
        game_write(rva(0x60A87C), grid_reach(pick(0x60AC88)));
        let mut eax = 0;
        for (table, to) in [
            (0x60AC9C, 0x60930C),
            (0x60ACB0, 0x609358),
            (0x60ACC4, 0x609308),
            (0x60ACD8, 0x609320),
            (0x60ACEC, 0x60AC80),
        ] {
            eax = pick(table);
            put(g(to), eax);
        }
        Left::nothing().with(EBX, level).with(EAX, eax)
    }
}

const NOISE_SAMPLES_RVA: usize = rva(0x637620); // dword_637620 .. dword_637634

/// `race::sample_noise` (0x44FD38): the 64 x 64 byte map at esi sampled at the work car's
/// place (`g_car` +0x6A4 and +0x6AC: the top byte the cell, the next two the fraction):
/// the four corners into `dword_637620` .. `dword_63762C`, blended along x into
/// `dword_637630` / `dword_637634` and then along z, each step `imul` and `shrd` by 16;
/// the result shifted left 8. Leaves eax that, ebx and ecx the two fractions, edx the last
/// product's high half.
pub unsafe fn sample_noise(map: u32) -> Left {
    // Converted: `rc2k_core::race::view::sample_noise`.
    unsafe {
        let car = g(0x719FD8);
        let (px, pz) = (get(car + 0x6A4), get(car + 0x6AC));
        let s = rc2k_core::race::view::sample_noise(|n| ((map + n) as *const u8).read(), px, pz);
        let out = crate::framework::game_addr(NOISE_SAMPLES_RVA) as u32;
        for (k, &c) in s.corners.iter().enumerate() {
            put(out + 4 * k as u32, c);
        }
        put(out + 0x10, s.near);
        put(out + 0x14, s.far);
        Left::nothing()
            .with(EAX, s.value)
            .with(EBX, s.fx)
            .with(ECX, s.fz)
            .with(EDX, s.high)
    }
}

pub const FOG_TABLE_RVA: usize = rva(0x63661C); // g_fogTable: 0x1000 bytes
const FOG_LENGTH_RVA: usize = rva(0x63761C); // g_fogTableLength

/// `race::fog_table` (0x44FC33): `draw_distance_settings`; the noise at the car
/// (`sample_noise` of `g_noiseBytes`) spread between `dword_609308` and `dword_609320` into
/// the step `dword_62DAF0`; `dword_6365F0` / `dword_6365F4` = (`dword_6415DC` x
/// `dword_6415D8` >> 4) x `dword_609310` / `dword_609360` >> 8; `g_fogTableLength` =
/// `dword_63DC58` x 8 bytes of `g_fogTable`, each ((sum of steps) >> 16) + `dword_609358` +
/// `dword_6365F0` held to 0 ..= 0xFF; `dword_636610` the next one's value with
/// `dword_60930C` (held) plus `dword_6365F0` (held). Gives the registers it leaves.
/// Refuses (before anything) a table length of 0 or past 0x1000.
pub unsafe fn fog_table(entry: &crate::framework::Regs) -> Result<crate::framework::Regs, Refused> {
    unsafe {
        let length = game_read::<u32>(rva(0x63DC58)).wrapping_shl(3);
        if length == 0 || length > 0x1000 {
            return Err(BadInput("a fog table length of 0 or past 0x1000").into());
        }
        // Converted: `rc2k_core::race::view::fog_table`.
        let mut r = *entry;
        draw_distance_settings().apply(&mut r);
        r.esi = g(0xA2AAE4);
        sample_noise(r.esi).apply(&mut r);
        let k = |a: usize| game_read::<u32>(rva(a));
        let fog = rc2k_core::race::view::fog_table(
            r.eax,
            &rc2k_core::race::view::FogSettings {
                low: k(0x609308),
                high: k(0x609320),
                scale: [k(0x6415DC), k(0x6415D8)],
                near: k(0x609310),
                far: k(0x609360),
                start: k(0x609358),
                end: k(0x60930C),
                length,
            },
        );
        r.ebp = fog.step;
        game_write(rva(0x62DAF0), fog.step);
        game_write(rva(0x6365F0), fog.near);
        game_write(rva(0x6365F4), fog.far);
        r.edx = length;
        game_write(FOG_LENGTH_RVA, length);
        let table = g(0x63661C);
        std::ptr::copy_nonoverlapping(fog.table.as_ptr(), table as *mut u8, fog.table.len());
        r.ebx = fog.sum;
        r.edi = length;
        r.eax = fog.next;
        game_write(rva(0x636610), r.eax);
        Ok(r)
    }
}

/// `race::set_up_fog`'s calls by place: 1 `scene::restart_clock` (0x450381), 2 the fog
/// colour to the device (`sub_44E76B`: `dword_5AB964` = eax, then `sub_41AE25`, render
/// state 0x22 by `d3d::set_render_state`). Place 0 is not used.
pub const FOG_CALLEES: [(u32, usize); 2] = [(1, 0x450381), (2, 0x44E76B)];
pub const FOG_COLOUR_RVA: usize = rva(0x624000); // dword_624000

/// `race::set_up_fog` (0x44FBD9): `scene::restart_clock`; `dword_624000` = the fog colour
/// from `dword_6092CC` (red), `dword_6092D0`, `dword_6092D4` (each shifted into its byte
/// and added); `fog_table`; the colour with 0xFF000000 to the device (part 2). Gives the
/// registers it leaves. Refuses (before anything) what `fog_table` would.
pub unsafe fn set_up_fog(
    entry: &crate::framework::Regs,
    part: crate::race::frame::Part,
) -> Result<crate::framework::Regs, Refused> {
    unsafe {
        let length = game_read::<u32>(rva(0x63DC58)).wrapping_shl(3);
        if length == 0 || length > 0x1000 {
            return Err(BadInput("a fog table length of 0 or past 0x1000").into());
        }
        let mut r = *entry;
        part(1, &mut r)?;
        // Converted: `rc2k_core::race::view::{fog_colour, device_fog_colour}`.
        r.ebx = rc2k_core::race::view::fog_colour(
            game_read(rva(0x6092CC)),
            game_read(rva(0x6092D0)),
            game_read(rva(0x6092D4)),
        );
        game_write(FOG_COLOUR_RVA, r.ebx);
        let mut r = fog_table(&r)?;
        r.eax = rc2k_core::race::view::device_fog_colour(game_read(FOG_COLOUR_RVA));
        part(2, &mut r)?;
        Ok(r)
    }
}

/// `set_up_fog`'s parts in the game: the Rust clock and fog colour.
pub fn fog_part(place: u32, r: &mut crate::framework::Regs) -> Result<(), Refused> {
    match place {
        1 => {
            r.eax =
                crate::scene::restart_clock::restart_clock(crate::system::tick_count::tick_count())
        }
        _ => unsafe { crate::scene::set_fog_colour::set_fog_colour(r.eax) },
    }
    Ok(())
}

const VIEW_PLACE_RVA: usize = rva(0x606298); // g_viewPlace, dword_60629C, dword_6062A0
const VIEW_ANGLES_RVA: usize = rva(0x6062B8); // g_viewAngles

/// `race::place_view` (0x4389A8): the view's place from the camera (`dword_5CCDA0`,
/// `dword_5CCDC0`, `dword_5CCDE0`) and its angles. Flat (`dword_60932C` set): the place as
/// it is, `g_car` +0x6A8 += `dword_5CCD80`, `g_viewAngles` = `dword_5CCD60`,
/// `dword_6062BC` = 0x40000000. Else the place moved by (-`dword_5CCD80`, 0, 0) turned by
/// `dword_5CCD40` (`math::turn_point_xz_ecx`) and that turned again by `dword_5CCD60`
/// (`math::turn_point_xz`, its x 0, y the first turn's z, z its x), `g_viewAngles` and
/// `dword_6062BC` the two angles, and the place also as whole numbers and floats
/// (`dword_6062B0` / `dword_6062B4`, `flt_6062A4` ..). `dword_6062C0` .. `dword_6062CC` 0
/// both ways. Gives the registers it leaves.
pub unsafe fn place_view(entry: &crate::framework::Regs) -> crate::framework::Regs {
    use crate::math::x87::from_int;
    unsafe {
        let mut r = *entry;
        let place = crate::framework::game_addr(VIEW_PLACE_RVA) as u32;
        r.eax = game_read(rva(0x5CCDA0));
        r.ebx = game_read(rva(0x5CCDC0));
        r.ecx = game_read(rva(0x5CCDE0));
        put(place, r.eax);
        put(place + 4, r.ebx);
        put(place + 8, r.ecx);
        let zero = || {
            for a in [0x6062C0usize, 0x6062C8, 0x6062C4, 0x6062CC] {
                game_write(rva(a), 0u32);
            }
        };
        if game_read::<u32>(rva(0x60932C)) != 0 {
            r.ecx = game_read(rva(0x5CCD80));
            let at = g(0x71A680);
            put(at, get(at).wrapping_add(r.ecx));
            r.eax = game_read(rva(0x5CCD60));
            game_write(VIEW_ANGLES_RVA, r.eax);
            game_write(rva(0x6062BC), 0x4000_0000u32);
            zero();
            return r;
        }
        let first: u32 = game_read(rva(0x5CCD40));
        let second: u32 = game_read(rva(0x5CCD60));
        let back = (game_read::<u32>(rva(0x5CCD80))).wrapping_neg();
        let [x1, z1, _] =
            crate::math::turn_point_xz_ecx::turn_point_xz_ecx(back as i32, 0, 0, first);
        let [x2, _, z2, edx] =
            crate::math::turn_point_xz::turn_point_xz(0, z1 as i32, x1 as i32, second);
        r.ebx = z1;
        r.ecx = z2;
        r.edx = edx;
        put(place, get(place).wrapping_add(x2));
        put(place + 4, get(place + 4).wrapping_add(z1));
        put(place + 8, get(place + 8).wrapping_add(z2));
        game_write(VIEW_ANGLES_RVA, second);
        r.eax = first;
        game_write(rva(0x6062BC), first);
        zero();
        game_write(rva(0x6062B0), get(place));
        game_write(rva(0x6062A4), from_int(get(place) as i32));
        game_write(rva(0x6062A8), from_int(get(place + 4) as i32));
        game_write(rva(0x6062B4), get(place + 8));
        game_write(rva(0x6062AC), from_int(get(place + 8) as i32));
        r
    }
}

fn ga(a: u32) -> usize {
    crate::framework::game_addr(rva(a as usize))
}

/// `race::sky_matrix` (0x42725E): `g_madeObjectMatrix` for the sky: a turn about y by
/// -(`dword_609334` << 8), placed at (0, `dword_609318` << 8, 0) (into `dword_63501C` ..),
/// the copy with the place (`g_turnedMatrix`) scaled by (`dword_6241D0` << 4) / 16
/// (`dword_635038`, through `dword_62400C`), times `g_viewTurnMatrix`. Leaves eax
/// `dword_6241D0`, ebx `dword_635038`, ecx 0, edx 0, esi `g_turnedMatrix`, edi
/// `g_viewTurnMatrix`, ebp `g_madeObjectMatrix`.
pub unsafe fn sky_matrix() -> Left {
    use crate::framework::EBP;
    use crate::scene::matrix;
    unsafe {
        let angle = (game_read::<u32>(rva(0x609334)) << 8).wrapping_neg();
        matrix::set_turn_y(angle);
        let height = game_read::<u32>(rva(0x609318)) << 8;
        game_write(rva(0x63501C), 0u32);
        game_write(rva(0x635024), 0u32);
        game_write(rva(0x635020), height);
        matrix::set_place([0, height, 0]);
        matrix::copy_matrix_with_place(ga(0x5E9FCC), ga(0x5EA04C), ga(0x5EA10C));
        let size_in: u32 = game_read(rva(0x6241D0));
        let size = size_in << 4;
        game_write(rva(0x635038), size);
        let scale = crate::math::x87::int_over_16(size as i32);
        game_write(SCRATCH_RVA, scale);
        let turned = ga(0x5EA10C) as u32;
        for k in 0..16 {
            put(
                turned + 4 * k,
                crate::math::x87::mul(get(turned + 4 * k), scale),
            );
        }
        matrix::multiply_matrices(ga(0x5EA10C), ga(0x5EA24C), ga(0x5EA30C));
        Left::nothing()
            .with(EAX, size_in)
            .with(EBX, size)
            .with(ECX, 0)
            .with(EDX, 0)
            .with(ESI, turned)
            .with(EDI, ga(0x5EA24C) as u32)
            .with(EBP, ga(0x5EA30C) as u32)
    }
}

/// The end of `view_matrices` (0x4388E5 .. and 0x438950 ..): the place negated into
/// `g_placedMatrix` (kept also at 0x5EA1CC), times the view's turn into `g_viewMatrix`,
/// the turn undone into `g_viewBackMatrix`. Gives the registers it leaves.
unsafe fn view_place_tail(r: &mut crate::framework::Regs) {
    use crate::framework::Regs;
    use crate::scene::{matrix, view_matrix};
    unsafe {
        let place = g(0x606298);
        let p = [get(place), get(place + 4), get(place + 8)].map(u32::wrapping_neg);
        matrix::set_place(p);
        std::ptr::copy(ga(0x5EA04C) as *const u8, ga(0x5EA1CC) as *mut u8, 0x40);
        matrix::multiply_matrices(ga(0x5EA04C), ga(0x5EA24C), ga(0x5EA2CC));
        view_matrix::invert_turn(ga(0x5EA24C), ga(0x5EA20C));
        *r = Regs {
            eax: p[0],
            ebx: p[1],
            ecx: 0,
            edx: 0,
            esi: ga(0x5EA24C) as u32,
            edi: ga(0x5EA20C) as u32,
            ebp: ga(0x5EA2CC) as u32,
            ..*r
        };
    }
}

/// `race::view_matrices` (0x4386A4): the view's matrices for the frame. With
/// `dword_63993C` set: `g_viewTurnMatrix` the kept turn (`flt_5EA18C`). Else for the car's
/// views 0 .. 3 (`g_car` +0x684) the car's turn undone (`g_crashMatrix`); for the others
/// the identity turned by the view's angles (fine y, fine x, z of `g_viewAngles`,
/// `dword_6062BC`, `dword_6062C0`, each negated; then again by `dword_6062C4` ..
/// `dword_6062CC`), each step `turn y`, `turn x`, `turn z` multiplied in. Views 1 and 2
/// then roll with the car (`g_car` +0x648 by its speed, eased into `g_car` +0x6A0, turned
/// about z). Then the place (see `view_place_tail`). Gives the registers it leaves.
pub unsafe fn view_matrices(entry: &crate::framework::Regs) -> crate::framework::Regs {
    use crate::scene::matrix::multiply_matrices;
    use crate::scene::view_matrix::{Axis, invert_turn, set_fine_turn};
    unsafe {
        let mut r = *entry;
        let (view_turn, turn_y, turn_x, turn_z) =
            (ga(0x5EA24C), ga(0x5E9FCC), ga(0x5E9F8C), ga(0x5EA00C));
        let (turned, lit) = (ga(0x5EA10C), ga(0x5EA14C));
        if game_read::<u32>(rva(0x63993C)) != 0 {
            std::ptr::copy(ga(0x5EA18C) as *const u8, view_turn as *mut u8, 0x40);
            view_place_tail(&mut r);
            return r;
        }
        let view: u32 = game_read(rva(0x71A65C));
        if view <= 3 {
            invert_turn(ga(0x71A08C), view_turn);
        } else {
            std::ptr::copy(ga(0x5E9F4C) as *const u8, view_turn as *mut u8, 0x40);
            for angles in [
                [0x6062B8u32, 0x6062BC, 0x6062C0],
                [0x6062C4, 0x6062C8, 0x6062CC],
            ] {
                set_fine_turn(
                    Axis::Y,
                    game_read::<u32>(rva(angles[0] as usize)).wrapping_neg(),
                );
                set_fine_turn(
                    Axis::X,
                    game_read::<u32>(rva(angles[1] as usize)).wrapping_neg(),
                );
                set_fine_turn(
                    Axis::Z,
                    game_read::<u32>(rva(angles[2] as usize)).wrapping_neg(),
                );
                multiply_matrices(view_turn, turn_y, turned);
                multiply_matrices(turned, turn_x, lit);
                multiply_matrices(lit, turn_z, view_turn);
            }
        }
        if view == 1 || view == 2 {
            use crate::math::x87_load::{below_or_unordered, eased_roll, product4_to_int};
            let k = |a: usize| game_read::<u32>(rva(a));
            let mut v = crate::math::x87::div(k(0x71A184), k(0x5C5D54));
            if below_or_unordered(v, k(0x5C5D90)) {
                v = k(0x5C5D90);
            }
            if !below_or_unordered(v, k(0x5C5D38)) {
                v = k(0x5C5D38);
            }
            game_write(SCRATCH_RVA, v);
            let roll = eased_roll(
                k(0x71A620) as i32,
                k(0x5C5D98),
                k(0x5C5DD4),
                v,
                k(0x5C5DD0),
                k(0x71A678),
                k(0x5C5E40),
            );
            game_write(rva(0x71A678), roll);
            let angle = product4_to_int(roll, k(0x5C5D98), k(0x5C5E28), k(0x5C5DD4)) << 4;
            game_write(SCRATCH_RVA, angle);
            set_fine_turn(Axis::Z, angle);
            multiply_matrices(view_turn, turn_z, turned);
            std::ptr::copy(turned as *const u8, view_turn as *mut u8, 0x40);
        }
        view_place_tail(&mut r);
        r
    }
}

/// `race::camera_view` (0x438325): the view for the frame: `dword_606274`, `dword_60626C`,
/// `dword_606270` from `g_car` +0x6F0, +0x69C, +0x684; then without `dword_621480`
/// `race::place_view`; else the place from the car's camera (`g_car` +0x6A4 .., the angles
/// from +0x6B0 .., `dword_606294` from +0x6C8) or, with `dword_63993C`, from the crash
/// (the kept turn made from `flt_5E9ECC` and `g_crashMatrix`, the place the car's raised by
/// 0x8000), written as whole numbers and floats (also to `g_car` +0x734 ..), and
/// `scene::set_view_window`; then `view_matrices`. Gives the registers it leaves.
pub unsafe fn camera_view(entry: &crate::framework::Regs) -> crate::framework::Regs {
    use crate::math::x87::from_int;
    unsafe {
        let mut r = *entry;
        let car = |o: u32| get(ga(0x719FD8) as u32 + o);
        r.eax = car(0x6F0);
        game_write(rva(0x606274), r.eax);
        r.eax = car(0x69C);
        game_write(rva(0x60626C), r.eax);
        r.eax = car(0x684);
        game_write(rva(0x606270), r.eax);
        if game_read::<u32>(rva(0x621480)) == 0 {
            r = place_view(&r);
            return view_matrices(&r);
        }
        let place: [u32; 3] = if game_read::<u32>(rva(0x63993C)) == 0 {
            for (from, to) in [
                (0x6C8u32, 0x606294usize),
                (0x6B0, 0x6062B8),
                (0x6B8, 0x6062BC),
                (0x6B4, 0x6062C0),
                (0x6C0, 0x6062C4),
                (0x6BC, 0x6062C8),
                (0x6C4, 0x6062CC),
            ] {
                r.eax = car(from);
                game_write(rva(to), r.eax);
            }
            [car(0x6A4), car(0x6A8), car(0x6AC)]
        } else {
            crate::scene::matrix::multiply_matrices(ga(0x5E9ECC), ga(0x71A08C), ga(0x5EA10C));
            crate::scene::view_matrix::invert_turn(ga(0x5EA10C), ga(0x5EA18C));
            let base = ga(0x719FD8) as u32;
            put(base + 0x6CC, car(0x134));
            put(base + 0x6D0, car(0x138).wrapping_add(0x8000));
            put(base + 0x6D4, car(0x13C));
            r.eax = game_read(rva(0x606278));
            game_write(rva(0x606294), r.eax);
            [car(0x6CC), car(0x6D0), car(0x6D4)]
        };
        let base = ga(0x719FD8) as u32;
        let view = g(0x606298);
        for k in 0..3u32 {
            put(base + 0x734 + 4 * k, place[k as usize]);
            put(view + 4 * k, place[k as usize]);
        }
        game_write(rva(0x6062B0), place[0]);
        game_write(rva(0x6062B4), place[2]);
        game_write(rva(0x6062A4), from_int(place[0] as i32));
        game_write(rva(0x6062A8), from_int(place[1] as i32));
        game_write(rva(0x6062AC), from_int(place[2] as i32));
        crate::scene::set_view_window::set_view_window().apply(&mut r);
        view_matrices(&r)
    }
}

/// `race::draw_sky`'s calls by place: 1 `sub_44E44D` (three render states), 2
/// `scene::draw_scene_object`, 3 `sub_45FD1D` (the draw batch flushed), 4
/// `scene::depth_buffer_on`. Place 0 is not used.
pub const SKY_CALLEES: [(u32, usize); 4] =
    [(1, 0x44E44D), (2, 0x451E3E), (3, 0x45FD1D), (4, 0x44E41F)];

/// `race::draw_sky` (0x455540): `race::fetch_view_value`; `dword_5CCCD4` 0; unless the
/// scene's mode (`dword_6365C4`) is 1 or 2: the sky (object 1 through `sky_matrix`, kept
/// at 0x5EA34C, full light, `dword_6365C8` 4 and `dword_621510` 1 while it is drawn by
/// parts 1 .. 4); then with a sun (`dword_609310` + `dword_609360` not 0): its direction
/// from `dword_609334` + `dword_609338` and `dword_60933C` (`flt_6415AC` ..), turned by the
/// sky's matrix (`dword_6415B8` ..), and when in front (z not below `flt_5C5E18`) its place
/// on the screen (`flt_6415D0` / `dword_6415C8` ..), `dword_6415C4` 1 when on the screen,
/// `dword_6415DC` its size (256 less half its distance from the centre, at least 0).
/// Gives the registers it leaves.
pub unsafe fn draw_sky(
    entry: &crate::framework::Regs,
    part: crate::race::frame::Part,
) -> Result<crate::framework::Regs, Refused> {
    use crate::math::x87_load::{sun_direction, sun_on_screen, sun_size, tbyte_to_int, to_tbyte};
    unsafe {
        let mut r = *entry;
        fetch_view_value()?.apply(&mut r);
        game_write(rva(0x5CCCD4), 0u32);
        let mode: u32 = game_read(rva(0x6365C4));
        if mode == 1 || mode == 2 {
            return Ok(r);
        }
        sky_matrix().apply(&mut r);
        std::ptr::copy(ga(0x5EA30C) as *const u8, ga(0x5EA34C) as *mut u8, 0x40);
        r.esi = ga(0x5EA30C) as u32 + 0x40;
        r.edi = ga(0x5EA34C) as u32 + 0x40;
        r.ecx = 0;
        for (a, v) in [
            (0x6214FCusize, 1u32),
            (0x6365C8, 4),
            (0x635084, 0xFF),
            (0x635088, 0xFF),
            (0x63508C, 0xFF),
            (0x635094, 0),
            (0x635098, 0),
            (0x621510, 1),
        ] {
            game_write(rva(a), v);
        }
        part(1, &mut r)?;
        game_write(rva(0x62DAF4), 0u32);
        part(2, &mut r)?;
        part(3, &mut r)?;
        part(4, &mut r)?;
        for a in [0x6365C8usize, 0x621510, 0x6415C4, 0x6415DC] {
            game_write(rva(a), 0u32);
        }
        r.eax = game_read::<u32>(rva(0x609310)).wrapping_add(game_read(rva(0x609360)));
        if r.eax == 0 {
            return Ok(r);
        }
        let k = |a: usize| game_read::<u32>(rva(a));
        r.eax = k(0x609338).wrapping_add(k(0x609334)) & 0xFF;
        game_write(SCRATCH_RVA, r.eax);
        let dir = sun_direction(
            r.eax as i32,
            k(0x5C5D60),
            k(0x6415A8),
            k(0x60933C) as i32,
            k(0x5C5E4C),
        );
        for (n, &d) in dir.iter().enumerate() {
            game_write(rva(0x6415AC + 4 * n), d);
        }
        // `lea esi, g_madeObjectMatrix` for `scene::rotate_point`, which keeps it.
        r.esi = ga(0x5EA30C) as u32;
        let turned = crate::scene::rotate_point::rotate_point(
            ga(0x5EA30C) as u32,
            [to_tbyte(dir[0]), to_tbyte(dir[1]), to_tbyte(dir[2])],
        );
        let place = turned.map(tbyte_to_int);
        for (n, &p) in place.iter().enumerate() {
            game_write(rva(0x6415B8 + 4 * n), p);
        }
        let z = crate::math::x87::from_int(place[2] as i32);
        if crate::math::x87_load::below_or_unordered(z, k(0x5C5E18)) {
            return Ok(r);
        }
        let flat = k(0x60932C) != 0;
        let v = [
            place[0],
            place[1],
            place[2],
            k(0x5CCD80),
            k(0x624088),
            k(0x6240A0),
            k(0x62409C),
        ];
        let [fx, ix, fy, iy] = sun_on_screen(
            &v,
            flat,
            crate::framework::game_addr(SCRATCH_RVA) as *mut u32,
        );
        game_write(rva(0x6415D0), fx);
        game_write(rva(0x6415C8), ix);
        game_write(rva(0x6415D4), fy);
        game_write(rva(0x6415CC), iy);
        let on = ix < k(0x4F494C) && iy < k(0x4F4950);
        game_write(rva(0x6415C4), u32::from(on));
        r.eax = ix;
        r.ebx = iy;
        let size = sun_size(fx, k(0x62409C), fy, k(0x6240A0), k(0x5C5E40), k(0x5C5E2C));
        game_write(SCRATCH_RVA, size);
        r.ebp = if (size as i32) < 0 { 0 } else { size };
        game_write(rva(0x6415DC), r.ebp);
        Ok(r)
    }
}

/// `draw_sky`'s parts in the game: the Rust versions (the glow's render states,
/// `scene::draw_scene_object`, the batch's flush, the depth buffer).
pub fn sky_part(place: u32, r: &mut crate::framework::Regs) -> Result<(), Refused> {
    unsafe {
        match place {
            4 => crate::scene::depth_buffer_on::depth_buffer(r, true),
            1 => crate::scene::glow_render_states::glow_render_states(r),
            2 => return crate::scene::draw_scene_object::draw_scene_object_in_game(r),
            3 => return crate::race::flush_draw_batch::flush_draw_batch_in_game(r),
            _ => {
                let callee = SKY_CALLEES
                    .iter()
                    .find(|&&(p, _)| p == place)
                    .map_or(0, |&(_, a)| a);
                crate::framework::call_game(r, crate::framework::game_addr(rva(callee)));
            }
        }
    }
    Ok(())
}

pub const KEPT_NORMALS_RVA: usize = rva(0x6399B8); // g_keptReflectionNormals: 0x400 dwords
pub const REFLECTIONS_KEPT_RVA: usize = rva(0x63D9B8); // g_reflectionsKept

/// `race::start_kept_reflections` (0x451864): the 0x400 kept reflection normals cleared to
/// -999 (0xFFFFFC19) and `g_reflectionsKept` 1. Leaves eax -999, ecx 0, edi past them.
pub unsafe fn start_kept_reflections() -> Left {
    unsafe {
        let at = g(0x6399B8);
        for k in 0..0x400 {
            put(at + 4 * k, 0xFFFF_FC19);
        }
        game_write(REFLECTIONS_KEPT_RVA, 1u32);
        Left::nothing()
            .with(EAX, 0xFFFF_FC19)
            .with(ECX, 0)
            .with(EDI, at + 0x1000)
    }
}

/// `race::end_kept_reflections` (0x451881): `g_reflectionsKept` 0.
pub unsafe fn end_kept_reflections() {
    unsafe { game_write(REFLECTIONS_KEPT_RVA, 0u32) };
}

/// `race::draw_slot_object`'s draw by place: 1 `scene::draw_scene_object`. Place 0 is
/// not used.
pub const SLOT_CALLEES: [(u32, usize); 1] = [(1, 0x451E3E)];

/// `race::draw_slot_object` (0x4626D1): race slot esi's object (+0, into `g_objectNumber`)
/// drawn, unless outside one player and modes 0 and 2 its car (object 0x50 + 0x14 x car)
/// is not the drawn view's (`dword_641680`): the light (lowest 0, highest 0xFF, level
/// `dword_6092E4`, strength `dword_60935C`, turns -`dword_609338` / `dword_60933C`,
/// `dword_6365C8` 1, `g_lightKind` 2), its turn (+0x10, 16 dwords to `flt_5EA444`) placed
/// at +4, +8 + (+0x5C), +0xC (into `dword_63501C` ..) and moved by the view (0x5EA1CC,
/// into `flt_5EA484`), times the view's turn into `g_madeObjectMatrix`; the reflections
/// kept around the drawing (part 1). Gives the registers it leaves. Refuses (before
/// anything) as `check_slot_object`.
pub unsafe fn draw_slot_object(
    entry: &crate::framework::Regs,
    part: crate::race::frame::Part,
) -> Result<crate::framework::Regs, Refused> {
    use crate::scene::matrix;
    unsafe {
        check_slot_object(entry.esi)?;
        let mut r = *entry;
        let slot = r.esi;
        r.eax = get(slot);
        game_write(rva(0x6214FC), r.eax);
        let mode: u32 = game_read(MODE_RVA);
        if game_read::<u32>(PLAYERS_RVA) != 1 && mode != 0 && mode != 2 {
            let n = r.eax.wrapping_sub(0x50);
            r.edx = n % 0x14;
            r.ebx = 0x14;
            r.eax = get(g(0x641680) + 4 * ((n / 0x14) & 7));
            if r.eax != game_read::<u32>(VIEW_RVA) {
                return Ok(r);
            }
        }
        for (a, v) in [
            (0x635054usize, 0u32),
            (0x635084, 0),
            (0x635088, 0xFF),
            (0x63508C, game_read(rva(0x6092E4))),
            (0x635090, game_read(rva(0x60935C))),
            (0x635094, game_read::<u32>(rva(0x609338)).wrapping_neg()),
            (0x635098, game_read(rva(0x60933C))),
            (0x6365C8, 1),
            (0x62DAF4, 2),
        ] {
            game_write(rva(a), v);
        }
        std::ptr::copy((slot + 0x10) as *const u8, ga(0x5EA444) as *mut u8, 0x40);
        let place = [
            get(slot + 4),
            get(slot + 8).wrapping_add(get(slot + 0x5C)),
            get(slot + 0xC),
        ];
        for (k, &p) in place.iter().enumerate() {
            game_write(rva(0x63501C + 4 * k), p);
        }
        matrix::set_place(place);
        matrix::copy_matrix_with_place(ga(0x5EA04C), ga(0x5EA1CC), ga(0x5EA484));
        matrix::copy_matrix_with_place(ga(0x5EA444), ga(0x5EA484), ga(0x5EA10C));
        matrix::multiply_matrices(ga(0x5EA10C), ga(0x5EA24C), ga(0x5EA30C));
        r.eax = place[0];
        r.ebx = place[1];
        r.ecx = 0;
        r.edx = 0;
        r.esi = ga(0x5EA10C) as u32;
        r.edi = ga(0x5EA24C) as u32;
        r.ebp = ga(0x5EA30C) as u32;
        start_kept_reflections().apply(&mut r);
        part(1, &mut r)?;
        end_kept_reflections();
        Ok(r)
    }
}

/// What `scene::draw_scene_object` would refuse for slot `at`'s object, before anything:
/// a car's object (0x50 .. 0xEF) without its block.
pub unsafe fn check_slot_object(at: u32) -> Result<(), Refused> {
    unsafe {
        let n = get(at).wrapping_sub(0x50);
        if n < 0xA0 && game_read::<u32>(CAR_BLOCKS_RVA + 4 * (n / 0x14) as usize) == 0 {
            return Err(BadInput("a slot's car without its block").into());
        }
        Ok(())
    }
}

/// `draw_slot_object`'s draw in the game: `scene::draw_scene_object`'s Rust version.
pub fn slot_part(_: u32, r: &mut crate::framework::Regs) -> Result<(), Refused> {
    unsafe { crate::scene::draw_scene_object::draw_scene_object_in_game(r) }
}

/// `race::draw_race_slots` (0x4621E5): `slot` (`draw_slot_object` in the game) for each
/// of the eight race slots (0x78 bytes at `g_raceSlots`) whose +0 is not 0 (inside
/// `pusha`). Leaves ecx 8, esi past the slots. Refuses (before anything) as
/// `check_slot_object` for any.
pub unsafe fn draw_race_slots(
    entry: &crate::framework::Regs,
    slot: &mut dyn FnMut(&crate::framework::Regs) -> Result<(), Refused>,
) -> Result<crate::framework::Regs, Refused> {
    unsafe {
        let slots = g(0x71BAC8);
        for n in 0..8 {
            if get(slots + 0x78 * n) != 0 {
                check_slot_object(slots + 0x78 * n)?;
            }
        }
        for n in 0..8 {
            let at = slots + 0x78 * n;
            if get(at) != 0 {
                let regs = crate::framework::Regs {
                    esi: at,
                    ecx: n,
                    ..*entry
                };
                slot(&regs)?;
            }
        }
        Ok(crate::framework::Regs {
            ecx: 8,
            esi: slots + 0x78 * 8,
            ..*entry
        })
    }
}

/// `race::draw_screen_quad`'s calls by place: 1 `scene::depth_buffer_off`, 2
/// `scene::draw_polygon`, 3 `scene::depth_buffer_on`. Place 0 is not used.
pub const QUAD_CALLEES: [(u32, usize); 3] = [(1, 0x44E3F1), (2, 0x44BA2C), (3, 0x44E41F)];

/// `race::draw_screen_quad` (0x44B364): a flat rectangle on the screen from x
/// `dword_623E10` .. `dword_623E14`, y `dword_623E20` .. `dword_623E24` (the four vertices
/// at 0x623E40, 0x20 bytes each, depth `flt_5C5E20`) in `dword_623DE8`'s colour: the
/// polygon's four work entries set (0x975510, 0x44 bytes each), clipped
/// (`scene::clip_polygon_to_screen`), and with three vertices or more drawn with the depth
/// buffer off (`g_polygonMode` 0, switches -1, 1, 0; `g_polygonDrawn` 1); the colour back
/// to 0xFFFFFF. Every register kept (its callers return at once). Refuses what the clip
/// refuses (after the entries are set: never seen with four vertices).
pub unsafe fn draw_screen_quad(
    entry: &crate::framework::Regs,
    part: crate::race::frame::Part,
) -> Result<(), Refused> {
    unsafe { draw_screen_quad_regs(entry, part) }.map(|_| ())
}

/// `draw_screen_quad`, giving the registers its parts leave (eax, ecx, edx, esi as above).
pub unsafe fn draw_screen_quad_regs(
    entry: &crate::framework::Regs,
    part: crate::race::frame::Part,
) -> Result<crate::framework::Regs, Refused> {
    unsafe {
        let mut r = *entry;
        game_write(rva(0x624134), 0u32);
        let entries = g(0x975510);
        let vertices = g(0x623E40);
        for k in 0..4u32 {
            let e = entries + 0x44 * k;
            put(e + 4, vertices + 0x20 * k);
            put(e + 8, 0);
            put(e + 0xC, 0);
        }
        // Converted: `rc2k_core::race::loading::quad_corners`.
        let corners = rc2k_core::race::loading::quad_corners(
            game_read(rva(0x623E10)),
            game_read(rva(0x623E14)),
            game_read(rva(0x623E20)),
            game_read(rva(0x623E24)),
        );
        for (k, (x, y)) in corners.into_iter().enumerate() {
            let v = vertices + 0x20 * k as u32;
            put(v + 0xC, x.to_bits());
            put(v + 0x1C, y.to_bits());
            put(v + 0x18, game_read(rva(0x5C5E20)));
        }
        game_write(SCRATCH_RVA, corners[2].1.to_bits());
        for k in 0..4u32 {
            put(entries + 0x44 * k, u32::MAX);
            put(entries + 0x44 * k + 0x18, 0);
        }
        game_write(rva(0x6240CC), game_read::<u32>(rva(0x623DE8)));
        game_write(rva(0x6240F8), 4u32);
        game_write(rva(0x6240A4), g(0x977F10));
        for k in 0..4u32 {
            put(g(0x977710) + 4 * k, k);
        }
        put(g(0x977710) + 16, 0);
        game_write(rva(0x624114), 4u32);
        // The registers the callees get, as the original leaves them: eax 0 and ecx 4 from
        // the list's loop, esi the record pool; the clip of a polygon wholly on the screen
        // then leaves eax the last entry's offset (its number x 0x44), ecx the count, edx 0
        // and esi the last vertex's record. A clipping's passes leave more (not modelled:
        // no callee reads them; `scene::clip_polygon_to_screen` keeps the registers).
        r.eax = 0;
        r.ecx = 4;
        r.esi = g(0x977F10);
        let unclipped = crate::scene::clip_polygon_to_screen::wholly_on_screen();
        crate::scene::clip_polygon_to_screen::clip_polygon_to_screen()?;
        if unclipped {
            crate::scene::clip_polygon_to_screen::unclipped_regs(&mut r);
        }
        if game_read::<u32>(rva(0x6240F8)) >= 3 {
            part(1, &mut r)?;
            game_write(rva(0x6240BC), u32::MAX);
            game_write(rva(0x63DAB4), 0u32);
            game_write(rva(0x6240E0), 1u32);
            game_write(rva(0x6240E4), 0u32);
            part(2, &mut r)?;
            game_write(rva(0x63DAB4), 0u32);
            game_write(rva(0x624134), 1u32);
            part(3, &mut r)?;
        }
        game_write(rva(0x6240CC), 0x00FF_FFFFu32);
        Ok(r)
    }
}

/// `draw_screen_quad`'s parts in the game: the Rust versions.
pub fn quad_part(place: u32, r: &mut crate::framework::Regs) -> Result<(), Refused> {
    unsafe {
        match place {
            1 => crate::scene::depth_buffer_on::depth_buffer(r, false),
            2 => crate::scene::draw_polygon::draw_polygon_in_game(r)?,
            _ => crate::scene::depth_buffer_on::depth_buffer(r, true),
        }
    }
    Ok(())
}

/// `race::draw_view_tint` (0x456092): the whole view (0 .. `dword_62408C`, 0 ..
/// `dword_624090`) as a screen quad in 0x204080 (`dword_623DE8`), switches -1, 1, 0, by
/// `draw_screen_quad`. Leaves the registers `draw_screen_quad` leaves.
pub unsafe fn draw_view_tint(
    entry: &crate::framework::Regs,
    part: crate::race::frame::Part,
) -> Result<crate::framework::Regs, Refused> {
    unsafe {
        let mut r = *entry;
        game_write(rva(0x623E10), 0u32);
        game_write(rva(0x623E14), game_read::<u32>(rva(0x62408C)));
        game_write(rva(0x623E20), 0u32);
        game_write(rva(0x623E24), game_read::<u32>(rva(0x624090)));
        r.eax = 0x20_4080;
        game_write(rva(0x623DE8), r.eax);
        game_write(rva(0x6240BC), u32::MAX);
        game_write(rva(0x6240E0), 1u32);
        game_write(rva(0x6240E4), 0u32);
        draw_screen_quad_regs(&r, part)
    }
}

/// `race::add_car_depth_entry` (0x4574A0): car block esi's place (+0x134 .., into
/// `dword_63501C` ..) through `g_viewMatrix` (rounded into `dword_63502C` ..; x and y back
/// as floats, z the key: plus 0x2000000, shifted right 10), a depth entry of the key with
/// ebx plus 0x2000000. Leaves eax the place's z, ebx that, ecx the key, edx the entry, esi
/// `g_viewMatrix`. Refuses (before anything) a depth entry past the 0x1000.
pub unsafe fn add_car_depth_entry(esi: u32, ebx: u32) -> Result<Left, Refused> {
    use crate::math::x87::from_int;
    unsafe {
        let n: u32 = game_read(DEPTH_COUNT_RVA);
        if n >= SORT_MAX {
            return Err(BadInput("a depth entry past the 0x1000").into());
        }
        let place = [get(esi + 0x134), get(esi + 0x138), get(esi + 0x13C)];
        for (k, &p) in place.iter().enumerate() {
            game_write(rva(0x63501C + 4 * k), p);
        }
        let matrix = ga(0x5EA2CC) as u32;
        let mut t = [0u32; 3];
        crate::math::x87_load::rotate_rounding(
            place.as_ptr() as *const i32,
            matrix as *const u32,
            t.as_mut_ptr(),
        );
        for (k, &v) in t.iter().enumerate() {
            game_write(rva(0x63502C + 4 * k), v);
        }
        game_write(rva(0x63501C), from_int(t[0] as i32));
        game_write(rva(0x635020), from_int(t[1] as i32));
        let key = t[2].wrapping_add(0x200_0000) >> 10;
        game_write(FACE_KEY_RVA, key);
        let second = ebx.wrapping_add(0x200_0000);
        put(g(0x9D3730) + 8 * n, key);
        put(g(0x9D3730) + 8 * n + 4, second);
        game_write(DEPTH_COUNT_RVA, n + 1);
        Ok(Left::nothing()
            .with(EAX, place[2])
            .with(EBX, second)
            .with(ECX, key)
            .with(EDX, n)
            .with(ESI, matrix))
    }
}

pub const FLARE_COUNT_RVA: usize = rva(0x64158C); // dword_64158C: the flares this frame
pub const FLARES_MAX: u32 = 0x400;

/// `sub_457332`: a flare at `dword_63501C` .. (object `g_objectNumber`, size
/// `dword_635038`, colour `dword_635054`) into the six flare tables (0x400 each, from
/// `dword_63F18C`) and, turned by `g_viewMatrix` (x and y back as floats, z rounded into
/// the key), a depth entry with the flare's number + 0x1000000; none past 0x400. Gives
/// the registers it leaves (CF clear).
unsafe fn add_flare(r: &mut crate::framework::Regs) {
    unsafe {
        r.ebx = game_read(FLARE_COUNT_RVA);
        if r.ebx == FLARES_MAX {
            return;
        }
        let n = r.ebx;
        let place = [
            game_read::<u32>(rva(0x63501C)),
            game_read(rva(0x635020)),
            game_read(rva(0x635024)),
        ];
        for (table, v) in [
            (0x64058Cu32, game_read::<u32>(rva(0x6214FC))),
            (0x63F18C, place[0]),
            (0x63F58C, place[1]),
            (0x63F98C, place[2]),
            (0x64018C, game_read(rva(0x635038))),
            (0x63FD8C, game_read(rva(0x635054))),
        ] {
            r.eax = v;
            put(g(table) + 4 * n, v);
        }
        let mut out = [0u32; 3];
        r.esi = ga(0x5EA2CC) as u32;
        crate::math::x87_load::rotate_rounding_z(
            place.as_ptr() as *const i32,
            r.esi as *const u32,
            out.as_mut_ptr(),
        );
        game_write(rva(0x63501C), out[0]);
        game_write(rva(0x635020), out[1]);
        let key = out[2].wrapping_add(0x200_0000) >> 10;
        game_write(FACE_KEY_RVA, key);
        r.ebx = n.wrapping_add(0x100_0000);
        r.edx = game_read(DEPTH_COUNT_RVA);
        r.ecx = key;
        put(g(0x9D3730) + 8 * r.edx, key);
        put(g(0x9D3730) + 8 * r.edx + 4, r.ebx);
        game_write(DEPTH_COUNT_RVA, r.edx + 1);
        game_write(FLARE_COUNT_RVA, n + 1);
    }
}

/// `sub_4572B0`: a flare at (eax, ebx, ecx) in car block esi's frame (its matrix +0xB4,
/// each part rounded, plus its place +0x134 ..), object 0x39, by `add_flare`.
unsafe fn place_flare(r: &mut crate::framework::Regs) {
    unsafe {
        let b = r.esi;
        let point = [r.eax, r.ebx, r.ecx];
        let mut t = [0u32; 3];
        crate::math::x87_load::rotate_rounding(
            point.as_ptr() as *const i32,
            (b + 0xB4) as *const u32,
            t.as_mut_ptr(),
        );
        game_write(SCRATCH_RVA, t[2]);
        r.eax = t[2].wrapping_add(get(b + 0x13C));
        game_write(rva(0x63501C), t[0].wrapping_add(get(b + 0x134)));
        game_write(rva(0x635020), t[1].wrapping_add(get(b + 0x138)));
        game_write(rva(0x635024), r.eax);
        game_write(rva(0x6214FC), 0x39u32);
        add_flare(r);
    }
}

/// `sub_457241`: a pair of lights at the light record's (edi) +ecx (three dwords: none
/// when all are 0): the right one (x as it is) when the block's byte +edx is set, the
/// left (x negated) when +ebx is (-1: no byte), each by `place_flare` inside `pusha`;
/// `dword_641758` / `dword_64175C` the two bytes.
unsafe fn light_pair(r: &mut crate::framework::Regs, at: u32, left: u32, right: u32) {
    unsafe {
        let byte = |o: u32| ((r.esi.wrapping_add(o)) as *const u8).read() as u32;
        game_write(rva(0x641758), 0u32);
        game_write(rva(0x64175C), 0u32);
        if left != u32::MAX {
            r.eax = byte(left);
            game_write(rva(0x641758), r.eax);
        }
        if right != u32::MAX {
            r.eax = byte(right);
            game_write(rva(0x64175C), r.eax);
        }
        r.eax = get(r.edi + at);
        r.ebx = get(r.edi + at + 4);
        r.ecx = get(r.edi + at + 8);
        r.edx = r.eax | r.ebx | r.ecx;
        if r.edx == 0 {
            return;
        }
        r.ebx = r.ebx.wrapping_sub(game_read(rva(0x613FC4)));
        if game_read::<u32>(rva(0x64175C)) != 0 {
            let mut inner = *r;
            place_flare(&mut inner);
        }
        r.eax = r.eax.wrapping_neg();
        if game_read::<u32>(rva(0x641758)) != 0 {
            let mut inner = *r;
            place_flare(&mut inner);
        }
    }
}

/// `race::car_light_flares` (0x457036): car block esi's lights as flares (see the hook's
/// header). Gives the registers it leaves. Refuses (before anything) a flare list or depth
/// list without room for the 24 flares a car can give.
pub unsafe fn car_light_flares(
    entry: &crate::framework::Regs,
) -> Result<crate::framework::Regs, Refused> {
    unsafe {
        if game_read::<u32>(DEPTH_COUNT_RVA) > SORT_MAX - 24 {
            return Err(BadInput("the depth list has no room for a car's flares").into());
        }
        let mut r = *entry;
        let b = r.esi;
        if game_read::<u32>(rva(0x621870)) == 8 {
            game_write(rva(0x635038), 2u32);
            r.eax = game_read(rva(0x641744));
            r.ebx = game_read::<u32>(rva(0x641748)).wrapping_sub(game_read(rva(0x613FC4)));
            r.ecx = game_read(rva(0x64174C));
            game_write(rva(0x635054), 0x00F0_F0F0u32);
            place_flare(&mut r);
            return Ok(r);
        }
        if game_read::<u32>(HIDING_RVA) == 0 {
            return Ok(r);
        }
        let kind = get(b + 0x24);
        r.edi = kind;
        game_write(rva(0x641750), kind);
        let valid = kind < 0x22 && game_read::<u32>(rva(0x5C5C58 + 4 * kind as usize)) != 0;
        if valid {
            if !crate::math::x87_load::below_or_unordered(get(b + 0x310), game_read(rva(0x5C5D14)))
            {
                return Ok(r);
            }
            r.eax = get(b + 0x298);
            game_write(rva(0x641754), r.eax);
            // A second test of the same kind follows; its error branch (a log line) is
            // never reached.
        }
        // `sub_41F4E1`: the kind held to 0 ..= 0x21 (by sign), through `dword_5C59B0`.
        let index = game_read::<u32>(rva(0x5C59B0 + 4 * (kind as i32).clamp(0, 0x21) as usize));
        r.edi = index.wrapping_mul(0x134).wrapping_add(g(0x5C906C));
        game_write(rva(0x6365C8), 1u32);
        game_write(rva(0x62DAF4), 3u32);
        game_write(rva(0x635038), 2u32);
        game_write(rva(0x635054), 0x0060_6060u32);
        let pair = |r: &mut crate::framework::Regs, at: u32, left: u32, right: u32| {
            r.ecx = at;
            r.ebx = left;
            r.edx = right;
            light_pair(r, at, left, right);
        };
        pair(&mut r, 0xA4, 0xAC0, 0xAC1);
        if game_read::<u32>(rva(0x62DAD0)) != 0 && game_read::<u32>(rva(0x641754)) >> 9 & 1 != 0 {
            pair(&mut r, 0xB0, 0xACA, 0xACB);
            pair(&mut r, 0xBC, 0xACC, 0xACD);
            pair(&mut r, 0xC8, 0xACE, 0xACF);
        }
        // `sub_45721A`: no left light for kind 0xD.
        let left = if kind == 0xD { u32::MAX } else { 0xAC6 };
        pair(&mut r, 0xF8, left, 0xAC7);
        game_write(rva(0x635054), 0x00C0_0000u32);
        // `sub_457229`: none for kinds 0xD and 0xE.
        let left = if kind == 0xD || kind == 0xE {
            u32::MAX
        } else {
            0xAC8
        };
        pair(&mut r, 0x104, left, 0xAC9);
        game_write(rva(0x635054), 0x0080_0000u32);
        pair(&mut r, 0xD4, 0xAC4, 0xAC5);
        pair(&mut r, 0xE0, 0xAD0, u32::MAX);
        pair(&mut r, 0xEC, 0xAC2, 0xAC3);
        Ok(r)
    }
}

/// `race::car_lights` (0x455CFF): with `dword_621480` set, for each race car: with +0x6FC
/// bit 5 (lights on, `car_light_modes`) its depth entry (`add_car_depth_entry`, the car
/// in ebx) and then `car_light_flares` (inside `pusha`). Leaves ebx the cars. Refuses (before
/// anything) no cars (the loop tests at its end) or more than eight, a null block, or a
/// depth list without room for every car's entries.
pub unsafe fn car_lights(
    entry: &crate::framework::Regs,
) -> Result<crate::framework::Regs, Refused> {
    unsafe {
        let mut r = *entry;
        if game_read::<u32>(HIDING_RVA) == 0 {
            return Ok(r);
        }
        // Converted: `rc2k_core::race::lights::{check_car_lights, lights_on}` (the null
        // block's refusal between its two, as before).
        use rc2k_core::race::lights::{check_car_lights, lights_on};
        let cars: u32 = game_read(CARS_RVA);
        check_car_lights(cars, 0, SORT_MAX)?;
        let block = |n: u32| game_read::<u32>(CAR_BLOCKS_RVA + 4 * n as usize);
        if (0..cars).any(|n| block(n) == 0) {
            return Err(BadInput("a null car block").into());
        }
        check_car_lights(cars, game_read(DEPTH_COUNT_RVA), SORT_MAX)?;
        for n in 0..cars {
            let inner = crate::framework::Regs {
                ebx: n,
                esi: block(n),
                ..*entry
            };
            // Both only with the lights on: the `jz` skips to the `popa`.
            if lights_on(get(inner.esi + 0x6FC)) {
                add_car_depth_entry(inner.esi, n)?;
                car_light_flares(&inner)?;
            }
        }
        r.ebx = cars;
        Ok(r)
    }
}

/// The end of `place_object_matrix`'s ways: the place `dword_63501C` .. into
/// `g_placedMatrix`, moved by the view (0x5EA1CC into `flt_5EA484`), `flt_5EA444` with it
/// (`g_turnedMatrix`), times the view's turn into `g_madeObjectMatrix`.
unsafe fn placed_object_tail(r: &mut crate::framework::Regs) {
    use crate::scene::matrix;
    unsafe {
        let place = [
            game_read::<u32>(rva(0x63501C)),
            game_read(rva(0x635020)),
            game_read(rva(0x635024)),
        ];
        matrix::set_place(place);
        matrix::copy_matrix_with_place(ga(0x5EA04C), ga(0x5EA1CC), ga(0x5EA484));
        matrix::copy_matrix_with_place(ga(0x5EA444), ga(0x5EA484), ga(0x5EA10C));
        matrix::multiply_matrices(ga(0x5EA10C), ga(0x5EA24C), ga(0x5EA30C));
        r.eax = place[0];
        r.ebx = place[1];
        r.ecx = 0;
        r.edx = 0;
        r.esi = ga(0x5EA10C) as u32;
        r.edi = ga(0x5EA24C) as u32;
        r.ebp = ga(0x5EA30C) as u32;
    }
}

/// `flt_5EA444`'s sixteen floats times `dword_635038` / 16 (through `dword_62400C`).
unsafe fn scale_object_turn() {
    unsafe {
        let scale = crate::math::x87::int_over_16(game_read::<u32>(rva(0x635038)) as i32);
        game_write(SCRATCH_RVA, scale);
        let at = ga(0x5EA444) as u32;
        for k in 0..16 {
            put(at + 4 * k, crate::math::x87::mul(get(at + 4 * k), scale));
        }
    }
}

/// `race::place_object_matrix` (0x42739A): `g_madeObjectMatrix` for an object placed at
/// `dword_63501C` .. (after `scene::set_light_direction`). With `dword_6365D0` 0: turned
/// about y, x and z (16 bits a turn) by `dword_635040`, `dword_63503C`, `dword_635044`
/// negated, unscaled. Else (nothing more with `dword_6365C8` 4) by the object's
/// material (+0x20 of its kind's 0x80-byte record at `g_objectMaterials`): 1 and 2 facing
/// the view (`g_viewBackMatrix`); 5 turned about y by `dword_6062C4` and the view's y
/// angle (fine turns); else about y by -`dword_63503C`, then with `dword_635040` tilted
/// about x round the y of `dword_635044`, and with `dword_635060` / `dword_635064` again
/// round `dword_63505C`; then scaled by `dword_635038` / 16. Gives the registers it leaves.
pub unsafe fn place_object_matrix(entry: &crate::framework::Regs) -> crate::framework::Regs {
    use crate::scene::matrix::{
        multiply_matrices, set_light_direction, set_turn_x, set_turn_y, set_turn_z_16,
    };
    use crate::scene::view_matrix::{Axis, set_fine_turn};
    unsafe {
        let mut r = *entry;
        let k = |a: usize| game_read::<u32>(rva(a));
        let neg = |a: usize| game_read::<u32>(rva(a)).wrapping_neg();
        let (turn_y, turn_x, turn_z) = (ga(0x5E9FCC), ga(0x5E9F8C), ga(0x5EA00C));
        let (turned, lit, object) = (ga(0x5EA10C), ga(0x5EA14C), ga(0x5EA444));
        set_light_direction();
        if k(0x6365D0) == 0 {
            set_turn_y(neg(0x635040));
            set_turn_x(neg(0x63503C));
            multiply_matrices(turn_y, turn_x, turned);
            set_turn_z_16(neg(0x635044));
            multiply_matrices(turned, turn_z, object);
            placed_object_tail(&mut r);
            return r;
        }
        if k(0x6365C8) == 4 {
            return r;
        }
        let kind = k(0x5CD27C + 4 * k(0x6214FC) as usize);
        let record = g(0x97DF30).wrapping_add(kind.wrapping_mul(0x80));
        let material = ((record + 0x20) as *const u8).read();
        r.eax = (record & !0xFF) | material as u32;
        match material {
            1 | 2 => {
                std::ptr::copy(ga(0x5EA20C) as *const u8, object as *mut u8, 0x40);
            }
            5 => {
                std::ptr::copy(ga(0x5E9F4C) as *const u8, object as *mut u8, 0x40);
                set_fine_turn(Axis::Y, k(0x6062C4));
                multiply_matrices(object, turn_y, turned);
                set_fine_turn(Axis::Y, k(0x6062B8));
                multiply_matrices(turned, turn_y, object);
            }
            _ => {
                set_turn_y(neg(0x63503C));
                std::ptr::copy(turn_y as *const u8, object as *mut u8, 0x40);
                if k(0x635040) != 0 {
                    set_turn_y(neg(0x635044));
                    multiply_matrices(object, turn_y, lit);
                    set_turn_x(neg(0x635040));
                    multiply_matrices(lit, turn_x, turned);
                    set_turn_y(k(0x635044));
                    multiply_matrices(turned, turn_y, object);
                }
                if k(0x635064) != 0 || k(0x635060) != 0 {
                    set_turn_y(neg(0x635064));
                    multiply_matrices(object, turn_y, lit);
                    set_turn_y(neg(0x63505C));
                    multiply_matrices(lit, turn_y, turned);
                    set_turn_x(neg(0x635060));
                    multiply_matrices(turned, turn_x, lit);
                    set_turn_y(k(0x63505C));
                    multiply_matrices(lit, turn_y, object);
                }
            }
        }
        scale_object_turn();
        placed_object_tail(&mut r);
        r
    }
}

/// `race::draw_flare` (0x455189): flare ebx of the six flare tables back into
/// `dword_63501C` .., `dword_635038`, `dword_635054`, `g_objectNumber`; `dword_6365C8` 2,
/// `place_object_matrix`, `g_lightKind` 0 and the drawing (part 6,
/// `scene::draw_scene_object`). Every register kept (`pusha` .. `popa`).
pub unsafe fn draw_flare(
    entry: &crate::framework::Regs,
    part: crate::race::frame::Part,
) -> Result<(), Refused> {
    unsafe {
        let flare = entry.ebx;
        for (table, to) in [
            (0x63F18Cu32, 0x63501Cusize),
            (0x63F58C, 0x635020),
            (0x63F98C, 0x635024),
            (0x64018C, 0x635038),
            (0x63FD8C, 0x635054),
            (0x64058C, 0x6214FC),
        ] {
            game_write(rva(to), get(g(table).wrapping_add(flare.wrapping_mul(4))));
        }
        game_write(rva(0x6365C8), 2u32);
        place_object_matrix(entry);
        game_write(rva(0x62DAF4), 0u32);
        // The `popa` and `pusha` between: the drawing gets the registers at entry.
        let mut r = *entry;
        part(6, &mut r)
    }
}

/// `race::draw_lamp_flare` (0x4551F4): lamp ebx (`g_lamps`): its point (`g_scenePoints`)
/// into `dword_63501C` .., its +4 x 8 the size, +8 the colour, object `dword_621508`;
/// `dword_6365C8` 2, `place_object_matrix`, `g_lightKind` 0 and the drawing (part 6).
/// Every register kept.
pub unsafe fn draw_lamp_flare(
    entry: &crate::framework::Regs,
    part: crate::race::frame::Part,
) -> Result<(), Refused> {
    unsafe {
        let lamp = entry.ebx;
        let edi = g(LAMPS).wrapping_add(lamp << 4);
        let p = get(edi)
            .wrapping_mul(12)
            .wrapping_add(game_read(rva(0x5CCCC0)));
        game_write(rva(0x63501C), get(p));
        game_write(rva(0x635020), get(p + 4));
        game_write(rva(0x635024), get(p + 8));
        game_write(rva(0x635038), get(edi + 4) << 3);
        game_write(rva(0x635054), get(edi + 8));
        game_write(rva(0x6214FC), game_read::<u32>(rva(0x621508)));
        game_write(rva(0x6365C8), 2u32);
        let mut r = place_object_matrix(entry);
        game_write(rva(0x62DAF4), 0u32);
        part(6, &mut r)
    }
}

/// `race::draw_list_entry`'s calls by place: 1 `scene::draw_described_object` (0x451B61),
/// 2 `race::draw_tinted_face` (0x44F9F3, a course face), 3 `race::draw_flare`, 4
/// `race::draw_lamp_flare`, 5 `race::draw_car_entry` (0x455105, a car), 6 `scene::draw_scene_object` (within 3
/// and 4). Place 0 is not used.
pub const ENTRY_CALLEES: [(u32, usize); 6] = [
    (1, 0x451B61),
    (2, 0x44F9F3),
    (3, 0x455189),
    (4, 0x4551F4),
    (5, 0x455105),
    (6, 0x451E3E),
];

/// `race::draw_list_entry` (0x454FF0): one entry's second word ebx (into `dword_5CCCD4`;
/// the top bit dropped): below 0x1000000 a course face (`g_currentCourseFace`): kind 0xFF
/// part 1, below 0x10 part 2, each with `dword_63E164` .. `dword_63E16C` 1 around it;
/// 0x1000000 .. a flare (part 3), 0x2000000 .. a car (part 5), 0x3000000 .. a lamp (part
/// 4), each by its low 24 bits; then `dword_6365C8` 0. Every register kept. Refuses (before
/// anything) a word of 0x4000000 or more (the original would take it for a face far past
/// the table).
pub unsafe fn draw_list_entry(
    entry: &crate::framework::Regs,
    part: crate::race::frame::Part,
) -> Result<(), Refused> {
    unsafe {
        let ebx = entry.ebx;
        let word = ebx & 0x7FFF_FFFF;
        if word >= 0x400_0000 {
            return Err(BadInput("a list entry of 0x4000000 or more").into());
        }
        game_write(rva(0x5CCCD4), ebx);
        let mut r = crate::framework::Regs {
            ebx: word & 0xFF_FFFF,
            ..*entry
        };
        match word >> 24 {
            0 => {
                game_write(CURRENT_FACE_RVA, word);
                let face = (word << 6).wrapping_add(game_read(rva(0x5CCCBC)));
                let kind = (face as *const u8).read();
                let place = if kind == 0xFF {
                    Some(1)
                } else if kind < 0x10 {
                    Some(2)
                } else {
                    None
                };
                if let Some(place) = place {
                    for a in [0x63E164usize, 0x63E168, 0x63E16C] {
                        game_write(rva(a), 1u32);
                    }
                    r.ebx = word;
                    r.esi = face;
                    r.eax = kind as u32;
                    part(place, &mut r)?;
                    for a in [0x63E164usize, 0x63E168, 0x63E16C] {
                        game_write(rva(a), 0u32);
                    }
                }
            }
            1 => part(3, &mut r)?,
            2 => part(5, &mut r)?,
            _ => part(4, &mut r)?,
        }
        game_write(rva(0x6365C8), 0u32);
        Ok(())
    }
}

/// `draw_list_entry`'s parts in the game.
pub fn entry_part(place: u32, r: &mut crate::framework::Regs) -> Result<(), Refused> {
    unsafe {
        match place {
            1 => crate::scene::draw_described_object::draw_described_object(
                r,
                &mut crate::scene::draw_described_object::described_part,
            ),
            2 => crate::race::course_faces::draw_tinted_face(
                r,
                &mut crate::race::course_faces::tinted_part,
            ),
            3 => draw_flare(r, &mut entry_part),
            5 => {
                crate::race::car_draw::draw_car_entry(r, &mut crate::race::car_draw::entry_car_part)
            }
            4 => draw_lamp_flare(r, &mut entry_part),
            6 => crate::scene::draw_scene_object::draw_scene_object_in_game(r),
            // Place 0 is not used.
            _ => Ok(()),
        }
    }
}

/// `race::draw_sorted_entries` (0x454EAB) / `race::draw_depth_entries` (0x454E84): each
/// entry of the list (`count` its count, `entries` its 8-byte entries) by `each`
/// (`draw_list_entry` in the game) with ebx its second word. Leaves ecx the count, esi the last entry, ebx
/// its word (none: kept). Refuses (before anything) what `draw_list_entry` would for any.
pub unsafe fn draw_entries(
    entry: &crate::framework::Regs,
    count: usize,
    entries: u32,
    each: &mut dyn FnMut(&crate::framework::Regs) -> Result<(), Refused>,
) -> Result<crate::framework::Regs, Refused> {
    unsafe {
        let mut r = *entry;
        let n: u32 = game_read(count);
        let list = g(entries);
        if (0..n).any(|k| get(list + 8 * k + 4) & 0x7FFF_FFFF >= 0x400_0000) {
            return Err(BadInput("a list entry of 0x4000000 or more").into());
        }
        for k in 0..n {
            r.ecx = k;
            r.esi = list + 8 * k;
            r.ebx = get(r.esi + 4);
            each(&r)?;
        }
        if n != 0 {
            r.ecx = n;
        }
        Ok(r)
    }
}
