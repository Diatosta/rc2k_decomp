use super::*;
use crate::sound::asm_layout::{Laid, with_laid};
use crate::test_support::own::race;
use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse};

fn blocks() -> Vec<Block> {
    vec![
        Block {
            name: "car_sound_state line 46",
            operands: vec![Operand::RaceBlock {
                reads: &[(0x213EE0, In::F), (0x31A260, In::F)],
                writes: &[(0x31A264, Out::F), (0x31A268, Out::F)],
            }],
            ulps: 0,
            plain: |_| x87_1(&race().shared),
        },
        Block {
            name: "car_sound_state line 64",
            operands: vec![
                Operand::RaceBlock {
                    reads: &[
                        (0x1C5E2C, In::F),
                        (0x31A264, In::F),
                        (0x31A268, In::F),
                        (0x31A26C, In::F),
                    ],
                    writes: &[],
                },
                Operand::Sized {
                    len: crate::sound::store::SoundObject::EXTENT,
                    reads: &[],
                    writes: &[
                        (0x44, Out::F),
                        (0x4C, Out::F),
                        (0x50, Out::F),
                        (0x58, Out::F),
                        (0x60, Out::F),
                        (0x64, Out::F),
                        (0x6C, Out::F),
                        (0x74, Out::F),
                        (0x78, Out::F),
                        (0x80, Out::F),
                        (0x88, Out::F),
                        (0x8C, Out::F),
                    ],
                },
            ],
            ulps: 0,
            plain: |a| {
                with_laid::<crate::sound::store::SoundObject, _>(a[1], |so| {
                    x87_2(race().course.numbers.get(), race().shared.car(), so);
                });
            },
        },
    ]
}

#[test]
fn x87_blocks_matches_the_assembly() {
    sparse("physics.car_sound_state", &blocks(), 1200, 100);
}
