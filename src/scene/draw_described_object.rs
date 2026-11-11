//! `scene::draw_described_object` (0x451B61): an object drawn from its description
//! (a course face record of kind 0xFF, part R1 of stream A; also `scene::draw_scene_object`'s
//! modes 1 and 2).
//!
//! Nothing for an object above 0x64 or a point past the scene's point count, nor in a race for
//! object 0x3F. Objects 0x38 and 0x3A .. 0x3F are signs: upright (0x2000), full light, 0x3D /
//! 0x3E half bright when any of +0x19 .. +0x1B, +0x20, +0x28 .. +0x2B is set. Then
//! the view kind 2, `race::place_object_matrix`, the light kind 2 (1 when the object's
//! material record has +0x1F), `scene::draw_scene_object` (given the material record, the
//! object and its +0x1F); after it (and in the cases of nothing) `only_object` 0 and the
//! object place -1.
//! Refuses (before anything) what `draw_scene_object` refuses for the object (a race car
//! without its block).

use crate::errors::{BadInput, Refused};

/// What `scene::draw_scene_object` would refuse for `object`: in a race (`g_inRace`) a car
/// (0x50 ..) without its block.
fn check_object(race: &crate::race::owner::Race, object: u32) -> Result<(), Refused> {
    let car = object.wrapping_sub(0x50);
    if race.course.numbers.in_race.is_set()
        && car < 0xA0
        && !race.shared.work.has_car_block((car / 0x14) as usize)
    {
        return Err(BadInput("a car without its block").into());
    }
    Ok(())
}

/// Where an object's description is read (passed in): a place in game
/// memory, or a view's sort list from a flare's entry on (`race::view_scratch`; the
/// original's entries lay in the buffers block, read on past the entry).
#[derive(Clone, Copy)]
pub enum Description<'a> {
    /// The material record of an object kind (and what follows it in the tables).
    Material(u32),
    Cells(&'a [crate::session::Dword]),
    /// A matrix's floats, read as the bytes they are.
    Floats(&'a [crate::session::Float]),
}

impl Description<'_> {
    /// The first `len` bytes of the words, or the refusal `past` when they are fewer.
    fn from_words(
        words: impl Iterator<Item = u32>,
        len: usize,
        past: &'static str,
    ) -> Result<Vec<u8>, BadInput> {
        let bytes: Vec<u8> = words
            .take(len.div_ceil(4))
            .flat_map(u32::to_le_bytes)
            .collect();
        bytes.get(..len).map(<[u8]>::to_vec).ok_or(BadInput(past))
    }

    /// The description's first `len` bytes.
    pub fn read_bytes(&self, len: usize) -> Result<Vec<u8>, BadInput> {
        match self {
            Description::Material(kind) => {
                let mut bytes = vec![0; len];
                crate::scene::polygon::material_run(*kind, 0, &mut bytes)?;
                Ok(bytes)
            }
            Description::Cells(cells) => Self::from_words(
                cells.iter().map(crate::session::Dword::get),
                len,
                "a described object past the sort lists",
            ),
            Description::Floats(cells) => Self::from_words(
                cells.iter().map(crate::session::Float::bits),
                len,
                "a described object past its matrix",
            ),
        }
    }
}

/// Draws the object of the description at `d`. Its calls: `race::place_object_matrix` and
/// `scene::draw_scene_object`.
pub fn draw_described_object(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    d: Option<Description<'_>>,
) -> Result<(), Refused> {
    let d = d.ok_or(BadInput("a described object without its description"))?;
    draw_description(ctx, &|len| d.read_bytes(len))
}

/// The same for course face `face` (kind 0xFF), its own description.
pub fn draw_described_face(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    face: u32,
) -> Result<(), Refused> {
    draw_description(ctx, &|len| {
        let bytes: [u8; 0x40] = crate::race::course_places::face_field(&ctx.race.course, face, 0)?;
        Ok(bytes[..len].to_vec())
    })
}

/// The drawing from a description's first `len` bytes as `read` gives them.
fn draw_description(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    read: &dyn Fn(usize) -> Result<Vec<u8>, BadInput>,
) -> Result<(), Refused> {
    let race = ctx.race;
    let course = &race.course;
    let shared = &race.shared;
    let scene = ctx.scene;
    let head = read(8)?;
    let object = head[1] as u32;
    let place = u32::from_le_bytes([head[4], head[5], head[6], head[7]]);
    let shown = object <= 0x64
        && place < course.course.scene_point_count.get()
        && !(shared.switches.scene_set_up.get() != 0 && object == 0x3F);
    if shown {
        check_object(race, object)?;
    }
    ctx.view.cars.object_number.set(object);
    let end = || {
        ctx.view.object.only_object.set(0u32);
        ctx.view.object.object_place.set(u32::MAX);
    };
    if object > 0x64 {
        end();
        return Ok(());
    }
    if place >= course.course.scene_point_count.get() {
        end();
        return Ok(());
    }
    ctx.view.object.object_place.set(place);
    if !shown {
        end();
        return Ok(());
    }
    // Everything the description gives from here on lies in its first 0x3C bytes.
    let desc = read(0x3C)?;
    let byte = |o: u32| desc[o as usize] as u32;
    let get = |o: usize| u32::from_le_bytes(desc[o..o + 4].try_into().unwrap());
    let at: [u32; 3] = crate::race::scene_points::point_number(&race.course, place)?;
    ctx.view.object.place[0].set(at[0].wrapping_add(get(0x30)));
    ctx.view.object.place[1].set(at[1].wrapping_add(get(0x34)));
    ctx.view.object.place[2].set(at[2].wrapping_add(get(0x38)));
    ctx.view.object.angles[0].set((byte(8) << 8) & 0xFFFF);
    ctx.view.object.angles[1].set(byte(0xC) << 8);
    ctx.view.object.angles[2].set(byte(0x10) << 8);
    ctx.view.object.described_turns[1].set(0u32);
    ctx.view.object.described_turns[0].set(0u32);
    ctx.view.object.described_turns[2].set(0u32);
    if byte(0x11) != 0 {
        ctx.view.object.described_turns[2].set(byte(9) << 8);
        ctx.view.object.described_turns[0].set(byte(0x12) << 8);
        ctx.view.object.described_turns[1].set(byte(0x13) << 8);
    }
    ctx.view.light.other_light.set(get(0x2C));
    ctx.view.object.only_object.set((byte(0x25) >> 1) & 3);
    let light = |lowest: u32, highest: u32, level: u32, strength: u32| {
        ctx.view.light.light_lowest.set(lowest);
        ctx.view.light.light_highest.set(highest);
        ctx.view.light.light_level.set(level);
        ctx.view.object.light_strength.set(strength);
    };
    if object == 0x38 || (0x3A..0x40).contains(&object) {
        ctx.view.object.angles[0].set(0x2000u32);
        ctx.view.object.angles[1].set(0u32);
        ctx.view.object.angles[2].set(0u32);
        ctx.view.object.size.set(byte(0x18));
        light(0xFF, 0xFF, 0xFF, 0);
        ctx.view.light.light_turns[0].set(0u32);
        ctx.view.light.light_turns[1].set(0u32);
        if (object == 0x3E || object == 0x3D)
            && [0x28u32, 0x29, 0x2A, 0x2B, 0x19, 0x1A, 0x1B, 0x20]
                .iter()
                .any(|&o| byte(o) != 0)
        {
            ctx.view.light.light_highest.set(0x80u32);
        }
    } else {
        ctx.view.object.size.set(byte(0x18));
        light(byte(0x14), byte(0x15), byte(0x16), byte(0x17));
        ctx.view.light.light_turns[0].set(byte(0x22));
        ctx.view.light.light_turns[1].set(byte(0x23));
        if byte(0x24) != 0 {
            ctx.view.light.light_turns[0].set(course.stage_prefs.sun_turn.get().wrapping_neg());
            ctx.view.light.light_turns[1].set(course.stage_prefs.sun_height.get());
            ctx.view
                .light
                .light_level
                .set(course.stage_prefs.light_level.get());
            ctx.view
                .object
                .light_strength
                .set(course.stage_prefs.light_strength_kept.get());
            ctx.view.light.light_lowest.set(0u32);
            ctx.view.light.light_highest.set(0xFFu32);
        }
    }
    ctx.view.light.view_kind.set(2u32);
    crate::race::frame_draw::place_object_matrix(ctx)?;
    ctx.view.light.light_kind.set(2u32);
    let object: u32 = ctx.view.cars.object_number.get();
    let kind: u32 = scene.objects.tables.object_kind(object);
    if crate::scene::polygon::material_run_byte(kind, 0x1F)? != 0 {
        ctx.view.light.light_kind.set(1u32);
    }
    crate::scene::draw_scene_object::draw_scene_object_in_game(
        ctx,
        Some(Description::Material(kind)),
    )?;
    end();
    Ok(())
}
