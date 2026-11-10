//! `effects::emit_body_particles` (0x4CA13B): the particles of the car's fifth record
//! (+0x1F4, under the body). First the object's +0x90 times `flt_765174` (rounded), the
//! record's +4 plus 0x386A0, its +0 bit 0 flipped (`dword_935416` 2, or 3 when the bit is
//! now clear), its +0xC 0; then `effects::emit_wheel_particles`' chance (its own random
//! `dword_935526`) of copying the surface's kind and flags, and its switch, but for kind 1
//! (`effects::emit_particle_18`) and kind 3 (the bursts of kind 2: section 16).
//!
//! Takes: the record, the car object
//!
//! Caller: `effects::emit_car_particles`'s end. Refuses a null record or object and the emitters'
//! refusals; on a refusal nothing stays written (all it may write is put back).

use crate::effects::emit_wheel_particles::{Table, emit};
use crate::effects::emitters::*;
use crate::effects::owner::Effects;
use crate::errors::Refused;
use crate::math::x87;
use crate::race::effects_settings_store::{CarEffects, MarkRecord};
const PACE: u32 = 0x386A0;

/// The first part (see the header), written.
fn prepare(
    ring: &Ring,
    record: &MarkRecord,
    object: &'static CarEffects,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    {
        let size =
            x87::int_times_to_int(object.rate.get() as i32, effects.setup.floats.three.bits());
        object.rate.set(size as u32);
        record.pace.set(record.pace.get().wrapping_add(PACE));
        effects.kinds.emitter.record.set(2);
        let flip = record.flip.get() ^ 1;
        record.flip.set(flip);
        if flip & 1 == 0 {
            effects.kinds.emitter.record.set(3);
        }
        record.driven_speed.set_bits(0);
        let chance = x87::mul_to_int(
            record.chance.bits(),
            effects.setup.floats.fixed_16_scale.bits(),
        );
        ring.work.whole.set(chance as u32);
        let r = next_c5(&effects.kinds.emitter.body_random);
        if ((r & 0xFFFF) as i32) < chance {
            record.take_surface();
        }
    }
    Ok(())
}

/// `record` the body's record (the original's edi),
/// `object` the car's object (esi).
pub fn emit_body_particles(
    effects: &Effects,
    record: &MarkRecord,
    object: &'static CarEffects,
) -> Result<(), Refused> {
    {
        prepare(&effects.ring, record, object, effects)?;
        emit(effects, Table::Body, record.kind.get(), record, object)
    }
}
