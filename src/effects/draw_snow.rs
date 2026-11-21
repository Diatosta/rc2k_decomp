//! `effects::draw_snow`: the snowflakes, `flakes` of them (also into `count_copy`; the walk's
//! `first` is 0, then plus the count; `drawn` counts the drawn). Each flake k: its place from
//! `randoms`[k], y plus the fall, x and z moved by `curve` at (the fall's bits 11 .. 19) plus
//! `remainders`[k], plus the view's place less the point (`view`), wrapped to 0x100000 and centred;
//! turned by `matrix_copy` plus `point` into `turned` (the depth also whole, in `depth`). Unless
//! the depth is at or below 0x10000 the flake is drawn as a square (side `side` over the depth,
//! depth times `inverse`, texture u `spreads`[k] plus `u_offset`, v `v`) through
//! `effects::clip_quad` and `scene::draw_fan`; with the light on its colour is `fade_colours`[the
//! square of the light's dot with the way from `light_place`, times the fade step, 0 when the dot
//! is below 0].
//!
//! The x87 blocks are plain Rust on the x87 model (`math::x87_float`). Refuses, before anything (a
//! dry pass, its scratch put back), more than 0x800 flakes, null marks settings with a flake, a
//! curve index past the 4095 entries and a fade index past the 17.

use crate::effects::clip_quad::{clip_quad, draw_fan};
use crate::effects::rain_store::Weather;
use crate::effects::snow_store::{SnowLight, SnowState, SnowTables};
use crate::effects::tuning_store::MarksFloats;
use crate::errors::{BadInput, Refused};

const MOST: i32 = 0x800;
const CURVE: u32 = 4095;
const FADES: u32 = 17;
/// The flake's whole-number place turned by the view.
fn turn(weather: &Weather, v: &[u32; 3]) {
    turn_x87(&v[..], &mut weather.snow.borrow_mut());
}

/// The light's dot and its fade index.
fn fade(weather: &Weather, effects: &crate::effects::owner::Effects) {
    fade_x87(
        &weather.snow.borrow(),
        &mut weather.snow_light.borrow_mut(),
        &effects.setup.floats,
    );
}

/// The projected square, its depth and u.
fn square(
    front: &crate::scene::polygon_work_store::PolygonWork,
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    weather: &Weather,
    i: u32,
) {
    let mut w = [0u32; 32];
    square_x87(
        front,
        crate::math::x87_mem::cells(&mut w),
        i as usize,
        &weather.snow_tables.borrow(),
        &mut weather.snow.borrow_mut(),
        &weather.snow_light.borrow(),
    );
    // The scratch holds four vertices' records, eight dwords each.
    for (vertex, w) in buffer.vertices.iter().zip(w.as_chunks::<8>().0) {
        let w = w.map(f32::from_bits);
        vertex.x.set(w[0]);
        vertex.y.set(w[1]);
        vertex.z.set(w[2]);
        vertex.u.set(w[6]);
    }
}

/// Flake k's whole-number place (x, y, z), the two curve indices (x, z) and the three
/// shifted values the indices are made from.
fn place(weather: &Weather, k: u32) -> ([u32; 3], [u32; 2], [u32; 3]) {
    {
        let t = weather.snow_tables.borrow();
        let mut x = t.randoms[0][k as usize];
        let mut y = t.randoms[1][k as usize].wrapping_add(weather.snow.borrow().fall);
        let mut z = t.randoms[2][k as usize];
        let f = (y & 0xF_FFFF) >> 11;
        let ix = f.wrapping_add(t.remainders[0][k as usize]);
        let iz = f.wrapping_add(t.remainders[1][k as usize]);
        let shift: [u32; 3] = weather.snow.borrow().view;
        let curve_table = weather.snow_curve.borrow();
        let curve = |i: u32| {
            if i < CURVE {
                curve_table.curve[i as usize]
            } else {
                0
            }
        };
        x = x.wrapping_add(curve(ix)).wrapping_add(shift[0]);
        z = z.wrapping_add(curve(iz)).wrapping_add(shift[2]);
        y = y.wrapping_add(shift[1]);
        let wrap = |v: u32| (v & 0xF_FFFF).wrapping_sub(0x7_FFFF);
        ([wrap(x), wrap(y), wrap(z)], [ix, iz], shift)
    }
}

/// Why it refuses before anything is drawn, if it does.
fn refusal(weather: &Weather, effects: &crate::effects::owner::Effects) -> Option<&'static str> {
    let n = weather.snow.borrow().flakes as i32;
    if n > MOST {
        return Some("more than 0x800 snowflakes");
    }
    if n > 0 && effects.setup.settings.is_set.get() == 0 {
        return Some("no marks settings");
    }
    None
}

pub(super) fn draw_snow(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    weather: &Weather,
    scene: &crate::scene::owner::Scene,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    let (snow, snow_light) = (&weather.snow, &weather.snow_light);
    if let Some(why) = refusal(weather, effects) {
        return Err(BadInput(why).into());
    }
    {
        let n = snow.borrow().flakes as i32;
        snow.borrow_mut().count_copy = n as u32;
        weather.snow_vars.borrow_mut().first = 0;
        snow.borrow_mut().drawn = 0;
        weather.snow_vars.borrow_mut().first = n as u32;
        for k in 0..n.max(0) as u32 {
            let ([eax, ebx, ecx], curve_at, _) = place(weather, k);
            if curve_at.iter().any(|&i| i >= CURVE) {
                return Err(BadInput("a snow curve index past the table").into());
            }
            turn(weather, &[ebx, eax, ecx]);
            if snow.borrow().depth as i32 > 0x1_0000 {
                if effects.setup.kind_floats.borrow().fading != 0 {
                    fade(weather, effects);
                    let mut index = snow_light.borrow().fade_index;
                    if snow_light.borrow().dot.is_sign_negative() {
                        index = 0;
                    }
                    if index >= FADES {
                        return Err(BadInput("a snow fade index past the table").into());
                    }
                    let colour = snow_light.borrow().fade_colours[index as usize];
                    for item in buffer.vertices.iter().take(4) {
                        item.colour.set(colour);
                    }
                }
                square(scene.scratch.front_end_polygon(), buffer, weather, 4 * k);
                let [top, bottom] = snow_light.borrow().v;
                for (n, vertex) in buffer.vertices.iter().enumerate().take(4) {
                    vertex.v.set(if n < 2 { top } else { bottom });
                }
                if !clip_quad(buffer, scene, effects)? {
                    draw_fan(buffer, scene);
                    let drawn = snow.borrow().drawn;
                    snow.borrow_mut().drawn = drawn.wrapping_add(1);
                    scene
                        .scratch
                        .front_end_polygon()
                        .polygon_vertex_count
                        .set(4);
                }
            }
        }
    }
    Ok(())
}

use crate::math::x87_float as bh;
use crate::math::x87_mem::{Base, fild as ldi, fld as ld, fstp as st};

/// `turn`'s arithmetic as plain Rust on the x87 model, in the same order.
fn turn_x87(v: impl Base, s: &mut SnowState) {
    let t0 = ldi(v, 0);
    let t1 = ldi(v, 0x4);
    let t2 = ldi(v, 0x8);
    let t3 = t1 * bh::ld(s.matrix_copy[0]);
    let t4 = t0 * bh::ld(s.matrix_copy[3]);
    let t5 = t2 * bh::ld(s.matrix_copy[6]);
    let t6 = t4 + t5;
    let t7 = t3 + t6;
    let t8 = t7 + bh::int(s.point[0] as i32);
    s.turned[0] = f32::from_bits(bh::st(t8));
    let t9 = t1 * bh::ld(s.matrix_copy[1]);
    let t10 = t0 * bh::ld(s.matrix_copy[4]);
    let t11 = t2 * bh::ld(s.matrix_copy[7]);
    let t12 = t10 + t11;
    let t13 = t9 + t12;
    let t14 = t13 + bh::int(s.point[1] as i32);
    s.turned[1] = f32::from_bits(bh::st(t14));
    let t15 = t1 * bh::ld(s.matrix_copy[2]);
    let t16 = t0 * bh::ld(s.matrix_copy[5]);
    let t17 = t2 * bh::ld(s.matrix_copy[8]);
    let t18 = t16 + t17;
    let t19 = t15 + t18;
    let t20 = t19 + bh::int(s.point[2] as i32);
    s.depth = bh::to_int(t20) as u32;
    s.turned[2] = f32::from_bits(bh::st(t20));
}

/// `fade`'s arithmetic as plain Rust on the x87 model, in the same order.
fn fade_x87(s: &SnowState, l: &mut SnowLight, f: &MarksFloats) {
    {
        let t0 = bh::ld(s.turned[0]);
        let t1 = t0 - bh::ld(l.light_place[0]);
        l.fade_delta[0] = f32::from_bits(bh::st(t1));
        let t2 = t1 * t1;
        let t3 = bh::ld(s.turned[1]);
        let t4 = t3 - bh::ld(l.light_place[1]);
        l.fade_delta[1] = f32::from_bits(bh::st(t4));
        let t5 = t4 * t4;
        let t6 = bh::ld(s.turned[2]);
        let t7 = t6 - bh::ld(l.light_place[2]);
        l.fade_delta[2] = f32::from_bits(bh::st(t7));
        let t8 = t7 * t7;
        let t9 = t5 + t8;
        let t10 = t2 + t9;
        let t11 = t10.sqrt();
        let t12 = bh::ld(f.one) / t11;
        let t13 = t12 * bh::ld(l.fade_delta[0]);
        l.fade_unit[0] = f32::from_bits(bh::st(t13));
        let t14 = t13 * bh::ld(l.light_way[0]);
        let t15 = t12 * bh::ld(l.fade_delta[1]);
        l.fade_unit[1] = f32::from_bits(bh::st(t15));
        let t16 = t15 * bh::ld(l.light_way[1]);
        let t17 = t12 * bh::ld(l.fade_delta[2]);
        l.fade_unit[2] = f32::from_bits(bh::st(t17));
        let t18 = t17 * bh::ld(l.light_way[2]);
        let t19 = t14 + t18;
        let t20 = t16 + t19;
        l.dot = f32::from_bits(bh::st(t20));
        let t21 = t20 * t20;
        let t22 = t21 * bh::ld(f.sixteen);
        l.fade_index = bh::to_int(t22) as u32;
    }
}

/// `square`'s arithmetic as plain Rust on the x87 model, in the same order.
fn square_x87(
    front: &crate::scene::polygon_work_store::PolygonWork,
    vt: impl Base,
    i: usize,
    t: &SnowTables,
    s: &mut SnowState,
    l: &SnowLight,
) {
    let t0 = bh::ld(s.turned[2]);
    let t1 = crate::math::x87_float::ld(front.window.borrow().view_scale) / t0;
    s.depth = bh::st(t1);
    let t2 = bh::ld(s.turned[1]);
    let t3 = t2 * t1;
    let t4 = t1 * bh::ld(s.turned[0]);
    let t5 = -t3;
    let t6 = t5 + crate::math::x87_float::ld(front.window.borrow().view_centre[1]);
    st(vt, 0x4, t6);
    let t7 = t4 + crate::math::x87_float::ld(front.window.borrow().view_centre[0]);
    st(vt, 0x0, t7);
    let t8 = bh::ld(l.side);
    let t9 = t8 * bh::ld(s.depth);
    let t10 = ld(vt, 0x0);
    let t11 = t10 + t9;
    st(vt, 0x20, t11);
    st(vt, 0x40, t11);
    let t12 = t11 - t9;
    st(vt, 0x60, t12);
    let t13 = ld(vt, 0x4);
    st(vt, 0x24, t13);
    let t14 = t9 + t13;
    st(vt, 0x44, t14);
    st(vt, 0x64, t14);
    let t15 = bh::ld(s.turned[2]);
    let t16 = t15 * bh::ld(s.inverse);
    st(vt, 0x8, t16);
    st(vt, 0x28, t16);
    st(vt, 0x48, t16);
    st(vt, 0x68, t16);
    let t17 = bh::ld(t.spreads[i / 4]);
    st(vt, 0x18, t17);
    st(vt, 0x78, t17);
    let t18 = t17 + bh::ld(l.u_offset);
    st(vt, 0x38, t18);
    st(vt, 0x58, t18);
}

#[cfg(test)]
mod tests;
