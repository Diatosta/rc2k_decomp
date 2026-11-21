use super::*;

fn key() -> PipelineKey {
    PipelineKey {
        cull: 1,
        depth_function: 4,
        depth_write: true,
        blend: None,
        colour: true,
    }
}

#[test]
fn first_key_sets_everything() {
    let c = key().changes_from(None);
    assert!(c.cull && c.depth_function && c.depth_write && c.blend && c.colour);
}

#[test]
fn same_key_sets_nothing() {
    let c = key().changes_from(Some(&key()));
    assert!(!(c.cull || c.depth_function || c.depth_write || c.blend || c.colour));
}

#[test]
fn only_the_differing_state_is_set() {
    let mut k = key();
    k.blend = Some((5, 6));
    let c = k.changes_from(Some(&key()));
    assert_eq!(
        c,
        PipelineChange {
            cull: false,
            depth_function: false,
            depth_write: false,
            blend: true,
            colour: false
        }
    );
    k.depth_write = false;
    k.colour = false;
    let c = k.changes_from(Some(&key()));
    assert!(c.depth_write && c.colour && c.blend && !c.cull && !c.depth_function);
}

#[test]
fn numbers_with_one_gl_value_do_not_count() {
    let mut k = key();
    k.cull = 7; // off, like 1
    k.depth_function = 9; // always, like 8
    let mut was = key();
    was.depth_function = 8;
    let c = k.changes_from(Some(&was));
    assert!(!c.cull && !c.depth_function);
    k.cull = 2;
    assert!(k.changes_from(Some(&was)).cull);
    assert_eq!(k.cull_face(), Some(0x0404));
    assert_eq!(key().depth_func(), 0x0203);
}
