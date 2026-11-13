//! The finish routine's camera from the eased time (`eased`) and the share (`share`): view
//! record 6's yaw (`angles[1]`), its pitch and its distance and height offsets (each a line in
//! them, `camera_yaw_rate` .. `camera_height_per_share`), its `offset[0]` 0.

use crate::math::x87_float::ld;
use crate::math::x87_float::st;
use crate::race::groups_store::ViewAndRecords;
use crate::race::owner_replay::Replay;
use crate::race::view_vars_store::{ViewRecord, ViewVars};

/// View record 6, the finish camera's (`ViewAndRecords.records[6].kind` ..).
fn record(view_records: &ViewAndRecords) -> &ViewRecord {
    ViewVars::record(view_records, 6)
}

/// The finish camera's lines (`ViewAndRecords.records[6]` angles and two more values) from the
/// eased time, the share and the lines' constants.
fn lines(view_records: &ViewAndRecords, replay: &Replay) {
    let finish = replay.finish.get();
    record(view_records).angles[1]
        .set(st(ld(finish.eased.get()) * ld(finish.camera_yaw_rate.get())));
    let a = ld(finish.eased.get()) * ld(finish.camera_height_rate.get())
        + ld(finish.camera_height_start.get());
    let b = ld(finish.share.get()) * ld(finish.camera_height_per_share.get());
    record(view_records).offset[1].set(st(a + b));
    record(view_records).offset[2].set(st(ld(finish.eased.get())
        * ld(finish.camera_distance_rate.get())
        + ld(finish.camera_distance_start.get())));
}

/// `ViewAndRecords.records[6].angles[0]` from the share and its constant.
fn last_line(view_records: &ViewAndRecords, replay: &Replay) {
    let finish = replay.finish.get();
    record(view_records).angles[0].set(st(
        ld(finish.share.get()) * ld(finish.camera_pitch_per_share.get())
    ));
}

/// (the exact one, for `drive::finish_run`).
pub fn finish_camera(view_records: &ViewAndRecords, replay: &Replay) {
    lines(view_records, replay);
    record(view_records).offset[0].set(0);
    last_line(view_records, replay);
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::race::owner::race;
    use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse};

    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "finish_camera lines",
                operands: vec![Operand::RaceBlock {
                    reads: &[
                        (0x24BB68, In::F),
                        (0x24BB4C, In::F),
                        (0x24BB5C, In::F),
                        (0x24BB54, In::F),
                        (0x24BB6C, In::F),
                        (0x24BB7C, In::F),
                        (0x24BB58, In::F),
                        (0x24BB50, In::F),
                    ],
                    writes: &[(0x206748, Out::F), (0x206730, Out::F), (0x206734, Out::F)],
                }],
                ulps: 0,
                plain: |_| lines(race().frame.view_and_records.get(), &race().replay),
            },
            Block {
                name: "finish_camera last line",
                operands: vec![Operand::RaceBlock {
                    reads: &[(0x24BB6C, In::F), (0x24BB78, In::F)],
                    writes: &[(0x206744, Out::F)],
                }],
                ulps: 0,
                plain: |_| last_line(race().frame.view_and_records.get(), &race().replay),
            },
        ]
    }

    #[test]
    fn finish_camera_matches_the_assembly() {
        sparse("drive.finish_camera", &blocks(), 3000, 201);
    }
}
