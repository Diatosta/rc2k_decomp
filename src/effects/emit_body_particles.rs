//! `effects::emit_body_particles`: the particles of the car's fifth record (the body's, the last of
//! `records`). First the object's `rate` times `MarksFloats.three` (rounded), the record's `pace`
//! plus 0x386A0, its `flip` bit 0 flipped (`EmitterVars.record` 2, or 3 when the bit is now clear),
//! its `driven_speed` 0; then `effects::emit_wheel_particles`' chance (its own random
//! `EmitterVars.body_random`) of copying the surface's kind and flags, and its switch, but for kind
//! 1 (`effects::emit_body_contact_particles`) and kind 3 (the bursts of kind 2: a slip kept as is).
//!
//! Refuses a null record or object and the emitters' refusals; on a refusal nothing stays written
//! (all it may write is put back).

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
    object: &CarEffects,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    {
        let size = x87::int_times_to_int(object.rate.get() as i32, effects.setup.floats.three);
        object.rate.set(size as u32);
        record.pace.set(record.pace.get().wrapping_add(PACE));
        effects.kinds.emitter.record.set(2);
        let flip = record.flip.get() ^ 1;
        record.flip.set(flip);
        if flip & 1 == 0 {
            effects.kinds.emitter.record.set(3);
        }
        record.driven_speed.set(0.0);
        let chance = x87::mul_to_int(&record.chance, effects.setup.floats.fixed_16_scale);
        ring.work.borrow_mut().whole = chance as u32;
        let r = next_c5(&effects.kinds.emitter.body_random);
        if ((r & 0xFFFF) as i32) < chance {
            record.take_surface();
        }
    }
    Ok(())
}

/// `record` is the body's record, `object` the car's object.
pub(super) fn emit_body_particles(
    effects: &Effects,
    record: &MarkRecord,
    object: &CarEffects,
) -> Result<(), Refused> {
    {
        prepare(&effects.ring, record, object, effects)?;
        emit(effects, Table::Body, record.kind.get(), record, object)
    }
}
