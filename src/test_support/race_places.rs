//! Where the race block's places the recorded blocks read and write live in the registry's
//! stores: the block's offset (the original instruction's displacement), the store's
//! registry name and the field in the store, by its path (`Cells::field_path`). The blocks'
//! tests (`vectors::block_vectors`) reach the places through the stores by name.

use std::collections::HashMap;
use std::sync::OnceLock;

/// `(offset, store, field path)`, sorted by offset.
#[rustfmt::skip]
const FIELDS: &[(usize, &str, &str)] = &[
    (0x1C2C64, "race.owner.RACE.replay.viewer_and_kinds", ".viewer_panel.texel_scale"),
    (0x1C2C68, "race.owner.RACE.replay.viewer_and_kinds", ".viewer_panel.reciprocal_width_800"),
    (0x1C2C6C, "race.owner.RACE.replay.viewer_and_kinds", ".viewer_panel.reciprocal_height_600"),
    (0x1C5CFC, "race.owner.RACE.course.numbers", ".hundredth_a"),
    (0x1C5D00, "race.owner.RACE.course.numbers", ".two_and_a_quarter"),
    (0x1C5D14, "race.owner.RACE.course.numbers", ".hundred"),
    (0x1C5D18, "race.owner.RACE.course.numbers", ".thousand"),
    (0x1C5D38, "race.owner.RACE.course.numbers", ".one"),
    (0x1C5D3C, "race.owner.RACE.course.numbers", ".two"),
    (0x1C5D40, "race.owner.RACE.course.numbers", ".three"),
    (0x1C5D48, "race.owner.RACE.course.numbers", ".five"),
    (0x1C5D60, "race.owner.RACE.course.numbers", ".sixty_four"),
    (0x1C5D98, "race.owner.RACE.course.numbers", ".k65536"),
    (0x1C5DA0, "race.owner.RACE.course.numbers", ".one_over_65536"),
    (0x1C5DA4, "race.owner.RACE.course.numbers", ".quarter"),
    (0x1C5DB4, "race.owner.RACE.course.numbers", ".one_and_a_half"),
    (0x1C5DD4, "race.owner.RACE.course.numbers", ".one_over_two_pi"),
    (0x1C5DDC, "race.owner.RACE.course.numbers", ".tenths[0]"),
    (0x1C5E08, "race.owner.RACE.course.numbers", ".kilometres_in_mile"),
    (0x1C5E28, "race.owner.RACE.course.numbers", ".k4096"),
    (0x1C5E2C, "race.owner.RACE.course.numbers", ".k256"),
    (0x1C5E34, "race.owner.RACE.course.numbers", ".k10000"),
    (0x1C5E40, "race.owner.RACE.course.numbers", ".half"),
    (0x1C5E48, "race.owner.RACE.course.numbers", ".grip_change"),
    (0x1CB958, "race.owner.RACE.frame.lights_and_gear", ".gear.shift_up_point"),
    (0x1CB95C, "race.owner.RACE.frame.lights_and_gear", ".gear.shift_down_point"),
    (0x1CB960, "race.owner.RACE.frame.lights_and_gear", ".gear.shift_down_clutch_point"),
    (0x1CB964, "race.owner.RACE.frame.lights_and_gear", ".gear.shift_ratio_figure"),
    (0x1CB968, "race.owner.RACE.frame.lights_and_gear", ".gear.drive_torque_figure"),
    (0x1CB96C, "race.owner.RACE.frame.lights_and_gear", ".gear.figures[0]"),
    (0x1CB970, "race.owner.RACE.frame.lights_and_gear", ".gear.figures[1]"),
    (0x1CB974, "race.owner.RACE.frame.lights_and_gear", ".gear.figures[2]"),
    (0x1CB978, "race.owner.RACE.frame.lights_and_gear", ".gear.figures[3]"),
    (0x1CB97C, "race.owner.RACE.frame.lights_and_gear", ".gear.test_marks[0]"),
    (0x1CB980, "race.owner.RACE.frame.lights_and_gear", ".gear.test_marks[1]"),
    (0x1CB984, "race.owner.RACE.frame.lights_and_gear", ".gear.test_marks[2]"),
    (0x1CCD80, "race.owner.RACE.course.course", ".flat_depth"),
    (0x205260, "race.owner.RACE.shared.tables", ".view_share"),
    (0x20628C, "race.owner.RACE.frame.view_and_records", ".view_vars.camera_bias_scale"),
    (0x206290, "race.owner.RACE.frame.view_and_records", ".view_vars.camera_bias_limit"),
    (0x2062D8, "race.owner.RACE.frame.view_and_records", ".view_vars.camera_vector[0]"),
    (0x2062DC, "race.owner.RACE.frame.view_and_records", ".view_vars.camera_vector[1]"),
    (0x2062E0, "race.owner.RACE.frame.view_and_records", ".view_vars.camera_vector[2]"),
    (0x20631C, "race.owner.RACE.frame.view_and_records", ".view_vars.camera_distance"),
    (0x206570, "race.owner.RACE.frame.view_and_records", ".records[1].focus[1]"),
    (0x206574, "race.owner.RACE.frame.view_and_records", ".records[1].focus[2]"),
    (0x2065C8, "race.owner.RACE.frame.view_and_records", ".records[2].focus[0]"),
    (0x2065CC, "race.owner.RACE.frame.view_and_records", ".records[2].focus[1]"),
    (0x2065D0, "race.owner.RACE.frame.view_and_records", ".records[2].focus[2]"),
    (0x206624, "race.owner.RACE.frame.view_and_records", ".records[3].focus[0]"),
    (0x206628, "race.owner.RACE.frame.view_and_records", ".records[3].focus[1]"),
    (0x20662C, "race.owner.RACE.frame.view_and_records", ".records[3].focus[2]"),
    (0x206730, "race.owner.RACE.frame.view_and_records", ".records[6].offset[1]"),
    (0x206734, "race.owner.RACE.frame.view_and_records", ".records[6].offset[2]"),
    (0x206744, "race.owner.RACE.frame.view_and_records", ".records[6].angles[0]"),
    (0x206748, "race.owner.RACE.frame.view_and_records", ".records[6].angles[1]"),
    (0x208B20, "race.owner.RACE.physics.camera", ".view_shift_a"),
    (0x208B24, "race.owner.RACE.physics.camera", ".view_shift_b"),
    (0x208B28, "race.owner.RACE.physics.camera", ".view_shift_c"),
    (0x208B2C, "race.owner.RACE.physics.camera", ".view_shift_d"),
    (0x208B30, "race.owner.RACE.physics.camera", ".camera_offset_x"),
    (0x208B34, "race.owner.RACE.physics.camera", ".camera_offset_y"),
    (0x208B38, "race.owner.RACE.physics.camera", ".camera_offset_z"),
    (0x208B50, "race.owner.RACE.physics.camera", ".start_heading"),
    (0x208B54, "race.owner.RACE.physics.camera", ".start_roll"),
    (0x208B58, "race.owner.RACE.physics.camera", ".start_pitch"),
    (0x208B5C, "race.owner.RACE.physics.camera", ".view_a_heading"),
    (0x208B60, "race.owner.RACE.physics.camera", ".view_a_roll"),
    (0x208B64, "race.owner.RACE.physics.camera", ".view_a_pitch"),
    (0x208B68, "race.owner.RACE.physics.camera", ".view_b_heading"),
    (0x208B6C, "race.owner.RACE.physics.camera", ".view_b_roll"),
    (0x208B70, "race.owner.RACE.physics.camera", ".view_b_pitch"),
    (0x208B98, "race.owner.RACE.physics.camera", ".view_a_offset[0]"),
    (0x208B9C, "race.owner.RACE.physics.camera", ".view_a_offset[1]"),
    (0x208BA0, "race.owner.RACE.physics.camera", ".view_a_offset[2]"),
    (0x208BB0, "race.owner.RACE.physics.camera", ".view_b_offset[0]"),
    (0x208BB4, "race.owner.RACE.physics.camera", ".view_b_offset[1]"),
    (0x208BB8, "race.owner.RACE.physics.camera", ".view_b_offset[2]"),
    (0x208BF0, "race.owner.RACE.physics.camera", ".view_blend_aux_2"),
    (0x21388C, "race.owner.RACE.shared.physics_tables", ".step_time"),
    (0x213E78, "race.owner.RACE.shared.physics_tables", ".grip_slide_scale"),
    (0x213E7C, "race.owner.RACE.shared.physics_tables", ".grip_slip_limit"),
    (0x213E80, "race.owner.RACE.shared.physics_tables", ".grip_force_limit"),
    (0x213E88, "race.owner.RACE.shared.physics_tables", ".spec_scale_a4"),
    (0x213EB8, "race.owner.RACE.shared.physics_tables", ".rev_limit_scale"),
    (0x213EC4, "race.owner.RACE.shared.physics_tables", ".drive_speed_scale"),
    (0x213ECC, "race.owner.RACE.shared.physics_tables", ".side_force_scale"),
    (0x213ED0, "race.owner.RACE.shared.physics_tables", ".slip_blend"),
    (0x213ED4, "race.owner.RACE.shared.physics_tables", ".slip_share"),
    (0x213ED8, "race.owner.RACE.shared.physics_tables", ".wheel_speed_margin"),
    (0x213EDC, "race.owner.RACE.shared.physics_tables", ".wheel_speed_gap"),
    (0x213EE0, "race.owner.RACE.shared.physics_tables", ".sound_speed_scale"),
    (0x213F04, "race.owner.RACE.shared.physics_tables", ".spec_default_bc0[0]"),
    (0x213F08, "race.owner.RACE.shared.physics_tables", ".spec_default_bc0[1]"),
    (0x213F0C, "race.owner.RACE.shared.physics_tables", ".spec_default_bc0[2]"),
    (0x213F10, "race.owner.RACE.shared.physics_tables", ".spec_scale_1b8"),
    (0x213F34, "race.owner.RACE.shared.physics_tables", ".grip_ramp"),
    (0x213F38, "race.owner.RACE.shared.physics_tables", ".grip_ramp_share"),
    (0x213F5C, "race.owner.RACE.shared.physics_tables", ".surface_blend[0]"),
    (0x213F60, "race.owner.RACE.shared.physics_tables", ".surface_blend[1]"),
    (0x213F64, "race.owner.RACE.shared.physics_tables", ".surface_blend[2]"),
    (0x213F68, "race.owner.RACE.shared.physics_tables", ".surface_blend[3]"),
    (0x213FAC, "race.owner.RACE.shared.physics_tables", ".inertia[1]"),
    (0x213FB8, "race.owner.RACE.shared.physics_tables", ".rev_limits[0]"),
    (0x213FBC, "race.owner.RACE.shared.physics_tables", ".rev_limits[1]"),
    (0x213FC0, "race.owner.RACE.shared.physics_tables", ".camera_drop"),
    (0x213FC8, "race.owner.RACE.shared.physics_tables", ".gravity"),
    (0x213FCC, "race.owner.RACE.shared.physics_tables", ".air_drag"),
    (0x213FD0, "race.owner.RACE.shared.physics_tables", ".rolling_drag"),
    (0x213FD4, "race.owner.RACE.shared.physics_tables", ".free_rev_add"),
    (0x213FD8, "race.owner.RACE.shared.physics_tables", ".free_rev_scale"),
    (0x213FE4, "race.owner.RACE.shared.physics_tables", ".rev_top_scale"),
    (0x213FEC, "race.owner.RACE.shared.physics_tables", ".wheel_load_scale"),
    (0x214098, "race.owner.RACE.shared.physics_tables", ".camera_frame_scale"),
    (0x2140B0, "race.owner.RACE.shared.physics_tables", ".drive_shares[0]"),
    (0x2140B4, "race.owner.RACE.shared.physics_tables", ".drive_shares[1]"),
    (0x2140B8, "race.owner.RACE.shared.physics_tables", ".drive_shares[2]"),
    (0x2140C0, "race.owner.RACE.shared.physics_tables", ".gear_ratios[1]"),
    (0x2140F8, "race.owner.RACE.shared.physics_tables", ".idle_engine_speed"),
    (0x2140FC, "race.owner.RACE.shared.physics_tables", ".stall_engine_speed"),
    (0x214104, "race.owner.RACE.shared.physics_tables", ".rev_limit"),
    (0x21411C, "race.owner.RACE.shared.physics_tables", ".surface_scales[0]"),
    (0x214120, "race.owner.RACE.shared.physics_tables", ".surface_scales[1]"),
    (0x214138, "race.owner.RACE.shared.physics_tables", ".vertical_speed"),
    (0x214148, "race.owner.RACE.shared.physics_tables", ".contact_step"),
    (0x21414C, "race.owner.RACE.shared.physics_tables", ".contact_steps"),
    (0x214164, "race.owner.RACE.shared.physics_tables", ".grip_drive_word"),
    (0x214170, "race.owner.RACE.shared.physics_tables", ".weight"),
    (0x2141D4, "race.owner.RACE.shared.physics_tables", ".wheel_spin_push[0]"),
    (0x2141D8, "race.owner.RACE.shared.physics_tables", ".wheel_spin_push[1]"),
    (0x21422C, "race.owner.RACE.shared.physics_tables", ".wheel_speed_step"),
    (0x214230, "race.owner.RACE.shared.physics_tables", ".surface_softness"),
    (0x214234, "race.owner.RACE.shared.physics_tables", ".wheel_slips[0]"),
    (0x214238, "race.owner.RACE.shared.physics_tables", ".wheel_slips[1]"),
    (0x21423C, "race.owner.RACE.shared.physics_tables", ".drive_limits[0]"),
    (0x214240, "race.owner.RACE.shared.physics_tables", ".drive_limits[1]"),
    (0x214244, "race.owner.RACE.shared.physics_tables", ".drive_limits[2]"),
    (0x214248, "race.owner.RACE.shared.physics_tables", ".drive_forces[0]"),
    (0x21424C, "race.owner.RACE.shared.physics_tables", ".drive_forces[1]"),
    (0x214250, "race.owner.RACE.shared.physics_tables", ".drive_slip[0]"),
    (0x214254, "race.owner.RACE.shared.physics_tables", ".drive_slip[1]"),
    (0x214260, "race.owner.RACE.shared.physics_tables", ".grip_limit"),
    (0x214264, "race.owner.RACE.shared.physics_tables", ".slip_limit"),
    (0x214268, "race.owner.RACE.shared.physics_tables", ".wheel_load"),
    (0x214284, "race.owner.RACE.shared.physics_tables", ".wheel_loads[0]"),
    (0x214288, "race.owner.RACE.shared.physics_tables", ".wheel_loads[1]"),
    (0x21428C, "race.owner.RACE.shared.physics_tables", ".wheel_loads[2]"),
    (0x214290, "race.owner.RACE.shared.physics_tables", ".wheel_loads[3]"),
    (0x214294, "race.owner.RACE.shared.physics_tables", ".wheel_loads[4]"),
    (0x214298, "race.owner.RACE.shared.physics_tables", ".wheel_grips[0]"),
    (0x21429C, "race.owner.RACE.shared.physics_tables", ".wheel_grips[1]"),
    (0x2142A0, "race.owner.RACE.shared.physics_tables", ".wheel_grips[2]"),
    (0x2142A4, "race.owner.RACE.shared.physics_tables", ".wheel_grips[3]"),
    (0x2142A8, "race.owner.RACE.shared.physics_tables", ".wheel_grips[4]"),
    (0x2142AC, "race.owner.RACE.shared.physics_tables", ".wheel_grips_b[0]"),
    (0x2142B0, "race.owner.RACE.shared.physics_tables", ".wheel_grips_b[1]"),
    (0x2142B4, "race.owner.RACE.shared.physics_tables", ".wheel_grips_b[2]"),
    (0x2142B8, "race.owner.RACE.shared.physics_tables", ".wheel_grips_b[3]"),
    (0x2142BC, "race.owner.RACE.shared.physics_tables", ".wheel_grips_b[4]"),
    (0x2142C0, "race.owner.RACE.shared.physics_tables", ".point_grips[0]"),
    (0x2142C4, "race.owner.RACE.shared.physics_tables", ".point_grips[1]"),
    (0x2142D0, "race.owner.RACE.shared.physics_tables", ".point_grips[4]"),
    (0x2146C0, "race.owner.RACE.shared.physics_tables", ".surface_blends[0]"),
    (0x2146C4, "race.owner.RACE.shared.physics_tables", ".surface_blends[1]"),
    (0x2146D0, "race.owner.RACE.shared.physics_tables", ".surface_blends[4]"),
    (0x2146D4, "race.owner.RACE.shared.physics_tables", ".surface_blends[5]"),
    (0x2146D8, "race.owner.RACE.shared.physics_tables", ".surface_blends[6]"),
    (0x2146E4, "race.owner.RACE.shared.physics_tables", ".surface_blends[9]"),
    (0x2146E8, "race.owner.RACE.shared.physics_tables", ".surface_blends[10]"),
    (0x2146EC, "race.owner.RACE.shared.physics_tables", ".surface_blends[11]"),
    (0x2146F8, "race.owner.RACE.shared.physics_tables", ".surface_blends[14]"),
    (0x2146FC, "race.owner.RACE.shared.physics_tables", ".surface_blends[15]"),
    (0x214700, "race.owner.RACE.shared.physics_tables", ".surface_blends[16]"),
    (0x21470C, "race.owner.RACE.shared.physics_tables", ".surface_blends[19]"),
    (0x214740, "race.owner.RACE.shared.physics_tables", ".drive_ratio"),
    (0x214748, "race.owner.RACE.shared.physics_tables", ".engine_force"),
    (0x214750, "race.owner.RACE.shared.physics_tables", ".drive_torque"),
    (0x214760, "race.owner.RACE.shared.physics_tables", ".wheel_drag[0]"),
    (0x214764, "race.owner.RACE.shared.physics_tables", ".wheel_drag[1]"),
    (0x214768, "race.owner.RACE.shared.physics_tables", ".lever_share"),
    (0x214774, "race.owner.RACE.shared.physics_tables", ".forces[0].force[1]"),
    (0x214794, "race.owner.RACE.shared.physics_tables", ".forces[1].force[1]"),
    (0x2147B4, "race.owner.RACE.shared.physics_tables", ".forces[2].force[1]"),
    (0x2147D4, "race.owner.RACE.shared.physics_tables", ".forces[3].force[1]"),
    (0x2148B0, "race.owner.RACE.shared.physics_tables", ".forces[10].force[0]"),
    (0x2148B8, "race.owner.RACE.shared.physics_tables", ".forces[10].force[2]"),
    (0x2148D0, "race.owner.RACE.shared.physics_tables", ".forces[11].force[0]"),
    (0x2148F0, "race.owner.RACE.shared.physics_tables", ".forces[12].force[0]"),
    (0x214910, "race.owner.RACE.shared.physics_tables", ".forces[13].force[0]"),
    (0x214930, "race.owner.RACE.shared.physics_tables", ".forces[14].force[0]"),
    (0x2149D0, "race.owner.RACE.shared.physics_tables", ".forces[19].force[0]"),
    (0x2149E4, "race.owner.RACE.shared.physics_tables", ".forces[19].point[2]"),
    (0x2149F0, "race.owner.RACE.shared.physics_tables", ".forces[20].force[0]"),
    (0x214A04, "race.owner.RACE.shared.physics_tables", ".forces[20].point[2]"),
    (0x214B74, "race.owner.RACE.shared.physics_tables", ".grip_scale"),
    (0x214B78, "race.owner.RACE.shared.physics_tables", ".side_grip_scale"),
    (0x214BA0, "race.owner.RACE.shared.physics_tables", ".spec_grip"),
    (0x214BA8, "race.owner.RACE.shared.physics_tables", ".spec_wheel"),
    (0x214BAC, "race.owner.RACE.shared.physics_tables", ".spec_drive[0]"),
    (0x214BB0, "race.owner.RACE.shared.physics_tables", ".spec_drive[1]"),
    (0x214BB4, "race.owner.RACE.shared.physics_tables", ".spec_surface[0]"),
    (0x214BB8, "race.owner.RACE.shared.physics_tables", ".spec_surface[1]"),
    (0x214BBC, "race.owner.RACE.shared.physics_tables", ".spec_surface[2]"),
    (0x214BC0, "race.owner.RACE.shared.physics_tables", ".spec_ints[0]"),
    (0x214BC4, "race.owner.RACE.shared.physics_tables", ".spec_ints[1]"),
    (0x214BC8, "race.owner.RACE.shared.physics_tables", ".spec_ints[2]"),
    (0x214BF0, "race.owner.RACE.shared.physics_tables", ".handling[1]"),
    (0x214BFC, "race.owner.RACE.shared.physics_tables", ".handling[4]"),
    (0x214C04, "race.owner.RACE.shared.physics_tables", ".handling[6]"),
    (0x214C08, "race.owner.RACE.shared.physics_tables", ".handling[7]"),
    (0x214C0C, "race.owner.RACE.shared.physics_tables", ".handling[8]"),
    (0x214D04, "race.owner.RACE.shared.physics_tables", ".split_point_distance"),
    (0x223AE4, "race.owner.RACE.shared.buffers.reads", ".engine_tables.sines[0]"),
    (0x223AE8, "race.owner.RACE.shared.buffers.reads", ".engine_tables.sines[1]"),
    (0x223AF4, "race.owner.RACE.shared.buffers.reads", ".engine_tables.sines[4]"),
    (0x223DA8, "race.owner.RACE.shared.vars", ".mud_scale_a[0]"),
    (0x223DAC, "race.owner.RACE.shared.vars", ".mud_scale_a[1]"),
    (0x22400C, "scene.owner.SCENE.drawing.scene", ".scratch"),
    (0x22401C, "scene.owner.SCENE.drawing.scene", ".work_float_a"),
    (0x224020, "scene.owner.SCENE.drawing.scene", ".work_float_b"),
    (0x224040, "scene.owner.SCENE.drawing.scene", ".surface_point[0]"),
    (0x224044, "scene.owner.SCENE.drawing.scene", ".surface_point[1]"),
    (0x224048, "scene.owner.SCENE.drawing.scene", ".surface_point[2]"),
    (0x237638, "scene.owner.SCENE.lights.state", ".camera_shake_x"),
    (0x23763C, "scene.owner.SCENE.lights.state", ".camera_shake_y"),
    (0x237640, "scene.owner.SCENE.lights.state", ".camera_shake_z"),
    (0x237644, "scene.owner.SCENE.lights.state", ".camera_shake_listener"),
    (0x239824, "race.owner.RACE.physics.ground", ".ground_x"),
    (0x23982C, "race.owner.RACE.physics.ground", ".ground_z"),
    (0x24BB38, "race.owner.RACE.replay.finish", ".car_lift"),
    (0x24BB44, "race.owner.RACE.replay.finish", ".limit_a"),
    (0x24BB48, "race.owner.RACE.replay.finish", ".limit_b"),
    (0x24BB4C, "race.owner.RACE.replay.finish", ".camera_yaw_rate"),
    (0x24BB50, "race.owner.RACE.replay.finish", ".camera_distance_start"),
    (0x24BB54, "race.owner.RACE.replay.finish", ".camera_height_start"),
    (0x24BB58, "race.owner.RACE.replay.finish", ".camera_distance_rate"),
    (0x24BB5C, "race.owner.RACE.replay.finish", ".camera_height_rate"),
    (0x24BB64, "race.owner.RACE.replay.finish", ".time"),
    (0x24BB68, "race.owner.RACE.replay.finish", ".eased"),
    (0x24BB6C, "race.owner.RACE.replay.finish", ".share"),
    (0x24BB70, "race.owner.RACE.replay.finish", ".limit_c"),
    (0x24BB74, "race.owner.RACE.replay.finish", ".limit_d"),
    (0x24BB78, "race.owner.RACE.replay.finish", ".camera_pitch_per_share"),
    (0x24BB7C, "race.owner.RACE.replay.finish", ".camera_height_per_share"),
    (0x2D1DF4, "race.owner.RACE.physics.contact", ".contact_share_numerator"),
    (0x2D1DF8, "race.owner.RACE.physics.contact", ".contact_normal_factor"),
    (0x2D2A14, "race.owner.RACE.physics.contact", ".probe_share_u[0]"),
    (0x2D2A18, "race.owner.RACE.physics.contact", ".probe_share_u[1]"),
    (0x2D2A24, "race.owner.RACE.physics.contact", ".probe_share_u[4]"),
    (0x2D2B14, "race.owner.RACE.physics.contact", ".probe_share_v[0]"),
    (0x2D2B18, "race.owner.RACE.physics.contact", ".probe_share_v[1]"),
    (0x2D2B24, "race.owner.RACE.physics.contact", ".probe_share_v[4]"),
    (0x2D3014, "race.owner.RACE.physics.contact", ".crash_ground_scale"),
    (0x2D307C, "race.owner.RACE.physics.contact", ".contact_normal[1]"),
    (0x2D3090, "race.owner.RACE.physics.contact", ".contact_point[0]"),
    (0x2D3094, "race.owner.RACE.physics.contact", ".contact_point[1]"),
    (0x2D3098, "race.owner.RACE.physics.contact", ".contact_point[2]"),
    (0x2D30A8, "race.owner.RACE.physics.contact", ".contact_car_speed[0]"),
    (0x2D30AC, "race.owner.RACE.physics.contact", ".contact_car_speed[1]"),
    (0x2D30B0, "race.owner.RACE.physics.contact", ".contact_car_speed[2]"),
    (0x2D30C0, "race.owner.RACE.physics.contact", ".contact_turn_cross[0]"),
    (0x2D30C4, "race.owner.RACE.physics.contact", ".contact_turn_cross[1]"),
    (0x2D30C8, "race.owner.RACE.physics.contact", ".contact_turn_cross[2]"),
    (0x2D30CC, "race.owner.RACE.physics.contact", ".contact_point_speed[0]"),
    (0x2D30D0, "race.owner.RACE.physics.contact", ".contact_point_speed[1]"),
    (0x2D30D4, "race.owner.RACE.physics.contact", ".contact_point_speed[2]"),
    (0x2D30E4, "race.owner.RACE.physics.contact", ".contact_normal_speed[0]"),
    (0x2D30E8, "race.owner.RACE.physics.contact", ".contact_normal_speed[1]"),
    (0x2D30EC, "race.owner.RACE.physics.contact", ".contact_normal_speed[2]"),
    (0x2D30F0, "race.owner.RACE.physics.contact", ".contact_side_speed[0]"),
    (0x2D30F4, "race.owner.RACE.physics.contact", ".contact_side_speed[1]"),
    (0x2D30F8, "race.owner.RACE.physics.contact", ".contact_side_speed[2]"),
    (0x2D30FC, "race.owner.RACE.physics.contact", ".contact_slide_limit_held"),
    (0x2D3100, "race.owner.RACE.physics.contact", ".contact_slide_limit"),
    (0x2D3130, "race.owner.RACE.physics.contact", ".contact_normal_push[0]"),
    (0x2D3134, "race.owner.RACE.physics.contact", ".contact_normal_push[1]"),
    (0x2D3138, "race.owner.RACE.physics.contact", ".contact_normal_push[2]"),
    (0x2D3140, "race.owner.RACE.physics.contact", ".contact_slide[0]"),
    (0x2D3144, "race.owner.RACE.physics.contact", ".contact_slide[1]"),
    (0x2D3148, "race.owner.RACE.physics.contact", ".contact_slide[2]"),
    (0x2D3150, "race.owner.RACE.physics.contact", ".contact_total[0]"),
    (0x2D3154, "race.owner.RACE.physics.contact", ".contact_total[1]"),
    (0x2D3158, "race.owner.RACE.physics.contact", ".contact_total[2]"),
    (0x2D3160, "race.owner.RACE.physics.contact", ".contact_extra[0]"),
    (0x2D3164, "race.owner.RACE.physics.contact", ".contact_extra[1]"),
    (0x2D3168, "race.owner.RACE.physics.contact", ".contact_extra[2]"),
    (0x2D31D0, "race.owner.RACE.physics.contact", ".crash_up_dot"),
    (0x2D31D8, "race.owner.RACE.physics.contact", ".contact_turn_push"),
    (0x2D31DC, "race.owner.RACE.physics.contact", ".contact_depth"),
    (0x2D31E0, "race.owner.RACE.physics.contact", ".contact_share"),
    (0x2D3204, "race.owner.RACE.physics.contact", ".contact_factor"),
    (0x2D3B4C, "race.owner.RACE.physics.contact", ".object_point_offset[0]"),
    (0x2D3B50, "race.owner.RACE.physics.contact", ".object_point_offset[1]"),
    (0x2D3B54, "race.owner.RACE.physics.contact", ".object_point_offset[2]"),
    (0x2D3B58, "race.owner.RACE.physics.contact", ".object_point_turned[0]"),
    (0x2D3B5C, "race.owner.RACE.physics.contact", ".object_point_turned[1]"),
    (0x2D3B60, "race.owner.RACE.physics.contact", ".object_point_turned[2]"),
    (0x2D3B70, "race.owner.RACE.physics.contact", ".face_angle"),
    (0x2D3B74, "race.owner.RACE.physics.contact", ".face_speed"),
    (0x2D3B78, "race.owner.RACE.physics.contact", ".object_point_scaled[0]"),
    (0x2D3B7C, "race.owner.RACE.physics.contact", ".object_point_scaled[1]"),
    (0x2D3B80, "race.owner.RACE.physics.contact", ".object_point_scaled[2]"),
    (0x2D3B88, "race.owner.RACE.physics.contact", ".bump_push_scale"),
    (0x2D3B9C, "race.owner.RACE.physics.contact", ".bump_push[0]"),
    (0x2D3BA0, "race.owner.RACE.physics.contact", ".bump_push[1]"),
    (0x2D3BA4, "race.owner.RACE.physics.contact", ".bump_push[2]"),
    (0x2D3BAC, "race.owner.RACE.physics.contact", ".bump_share_a"),
    (0x2D3BB0, "race.owner.RACE.physics.contact", ".bump_share_b"),
    (0x2D3CD8, "race.owner.RACE.physics.contact", ".bump_relative_speed[0]"),
    (0x2D3CDC, "race.owner.RACE.physics.contact", ".bump_relative_speed[1]"),
    (0x2D3CE0, "race.owner.RACE.physics.contact", ".bump_relative_speed[2]"),
];

/// The places in the work car, the race work and the replay's car blocks, by their byte in the
/// store.
#[rustfmt::skip]
const BYTES: &[(usize, &str, usize)] = &[
    (0x30F524, "race.owner.RACE.replay.car_blocks", 0x138),
    (0x319FE0, "race.owner.RACE.shared.car", 0x8),
    (0x319FF8, "race.owner.RACE.shared.car", 0x20),
    (0x31A008, "race.owner.RACE.shared.car", 0x30),
    (0x31A00C, "race.owner.RACE.shared.car", 0x34),
    (0x31A018, "race.owner.RACE.shared.car", 0x40),
    (0x31A038, "race.owner.RACE.shared.car", 0x60),
    (0x31A0A0, "race.owner.RACE.shared.car", 0xC8),
    (0x31A10C, "race.owner.RACE.shared.car", 0x134),
    (0x31A110, "race.owner.RACE.shared.car", 0x138),
    (0x31A114, "race.owner.RACE.shared.car", 0x13C),
    (0x31A158, "race.owner.RACE.shared.car", 0x180),
    (0x31A170, "race.owner.RACE.shared.car", 0x198),
    (0x31A174, "race.owner.RACE.shared.car", 0x19C),
    (0x31A178, "race.owner.RACE.shared.car", 0x1A0),
    (0x31A17C, "race.owner.RACE.shared.car", 0x1A4),
    (0x31A180, "race.owner.RACE.shared.car", 0x1A8),
    (0x31A184, "race.owner.RACE.shared.car", 0x1AC),
    (0x31A190, "race.owner.RACE.shared.car", 0x1B8),
    (0x31A194, "race.owner.RACE.shared.car", 0x1BC),
    (0x31A198, "race.owner.RACE.shared.car", 0x1C0),
    (0x31A1A0, "race.owner.RACE.shared.car", 0x1C8),
    (0x31A1B0, "race.owner.RACE.shared.car", 0x1D8),
    (0x31A1B4, "race.owner.RACE.shared.car", 0x1DC),
    (0x31A1B8, "race.owner.RACE.shared.car", 0x1E0),
    (0x31A1CC, "race.owner.RACE.shared.car", 0x1F4),
    (0x31A1D8, "race.owner.RACE.shared.car", 0x200),
    (0x31A1DC, "race.owner.RACE.shared.car", 0x204),
    (0x31A1E0, "race.owner.RACE.shared.car", 0x208),
    (0x31A1E8, "race.owner.RACE.shared.car", 0x210),
    (0x31A1EC, "race.owner.RACE.shared.car", 0x214),
    (0x31A1F0, "race.owner.RACE.shared.car", 0x218),
    (0x31A1F4, "race.owner.RACE.shared.car", 0x21C),
    (0x31A1F8, "race.owner.RACE.shared.car", 0x220),
    (0x31A1FC, "race.owner.RACE.shared.car", 0x224),
    (0x31A200, "race.owner.RACE.shared.car", 0x228),
    (0x31A204, "race.owner.RACE.shared.car", 0x22C),
    (0x31A208, "race.owner.RACE.shared.car", 0x230),
    (0x31A258, "race.owner.RACE.shared.car", 0x280),
    (0x31A260, "race.owner.RACE.shared.car", 0x288),
    (0x31A264, "race.owner.RACE.shared.car", 0x28C),
    (0x31A268, "race.owner.RACE.shared.car", 0x290),
    (0x31A26C, "race.owner.RACE.shared.car", 0x294),
    (0x31A278, "race.owner.RACE.shared.car", 0x2A0),
    (0x31A27C, "race.owner.RACE.shared.car", 0x2A4),
    (0x31A280, "race.owner.RACE.shared.car", 0x2A8),
    (0x31A284, "race.owner.RACE.shared.car", 0x2AC),
    (0x31A288, "race.owner.RACE.shared.car", 0x2B0),
    (0x31A28C, "race.owner.RACE.shared.car", 0x2B4),
    (0x31A290, "race.owner.RACE.shared.car", 0x2B8),
    (0x31A294, "race.owner.RACE.shared.car", 0x2BC),
    (0x31A298, "race.owner.RACE.shared.car", 0x2C0),
    (0x31A29C, "race.owner.RACE.shared.car", 0x2C4),
    (0x31A2A0, "race.owner.RACE.shared.car", 0x2C8),
    (0x31A2A4, "race.owner.RACE.shared.car", 0x2CC),
    (0x31A2A8, "race.owner.RACE.shared.car", 0x2D0),
    (0x31A2AC, "race.owner.RACE.shared.car", 0x2D4),
    (0x31A2B0, "race.owner.RACE.shared.car", 0x2D8),
    (0x31A2B4, "race.owner.RACE.shared.car", 0x2DC),
    (0x31A2B8, "race.owner.RACE.shared.car", 0x2E0),
    (0x31A2BC, "race.owner.RACE.shared.car", 0x2E4),
    (0x31A2C0, "race.owner.RACE.shared.car", 0x2E8),
    (0x31A2C4, "race.owner.RACE.shared.car", 0x2EC),
    (0x31A2C8, "race.owner.RACE.shared.car", 0x2F0),
    (0x31A2CC, "race.owner.RACE.shared.car", 0x2F4),
    (0x31A2D0, "race.owner.RACE.shared.car", 0x2F8),
    (0x31A2D4, "race.owner.RACE.shared.car", 0x2FC),
    (0x31A2D8, "race.owner.RACE.shared.car", 0x300),
    (0x31A2DC, "race.owner.RACE.shared.car", 0x304),
    (0x31A2E0, "race.owner.RACE.shared.car", 0x308),
    (0x31A2E4, "race.owner.RACE.shared.car", 0x30C),
    (0x31A2E8, "race.owner.RACE.shared.car", 0x310),
    (0x31A2EC, "race.owner.RACE.shared.car", 0x314),
    (0x31A2F0, "race.owner.RACE.shared.car", 0x318),
    (0x31A2F4, "race.owner.RACE.shared.car", 0x31C),
    (0x31A2F8, "race.owner.RACE.shared.car", 0x320),
    (0x31A2FC, "race.owner.RACE.shared.car", 0x324),
    (0x31A300, "race.owner.RACE.shared.car", 0x328),
    (0x31A304, "race.owner.RACE.shared.car", 0x32C),
    (0x31A308, "race.owner.RACE.shared.car", 0x330),
    (0x31A30C, "race.owner.RACE.shared.car", 0x334),
    (0x31A310, "race.owner.RACE.shared.car", 0x338),
    (0x31A314, "race.owner.RACE.shared.car", 0x33C),
    (0x31A318, "race.owner.RACE.shared.car", 0x340),
    (0x31A31C, "race.owner.RACE.shared.car", 0x344),
    (0x31A320, "race.owner.RACE.shared.car", 0x348),
    (0x31A324, "race.owner.RACE.shared.car", 0x34C),
    (0x31A328, "race.owner.RACE.shared.car", 0x350),
    (0x31A32C, "race.owner.RACE.shared.car", 0x354),
    (0x31A330, "race.owner.RACE.shared.car", 0x358),
    (0x31A334, "race.owner.RACE.shared.car", 0x35C),
    (0x31A338, "race.owner.RACE.shared.car", 0x360),
    (0x31A33C, "race.owner.RACE.shared.car", 0x364),
    (0x31A3D0, "race.owner.RACE.shared.car", 0x3F8),
    (0x31A3D4, "race.owner.RACE.shared.car", 0x3FC),
    (0x31A3D8, "race.owner.RACE.shared.car", 0x400),
    (0x31A3DC, "race.owner.RACE.shared.car", 0x404),
    (0x31A3E0, "race.owner.RACE.shared.car", 0x408),
    (0x31A410, "race.owner.RACE.shared.car", 0x438),
    (0x31A414, "race.owner.RACE.shared.car", 0x43C),
    (0x31A418, "race.owner.RACE.shared.car", 0x440),
    (0x31A41C, "race.owner.RACE.shared.car", 0x444),
    (0x31A420, "race.owner.RACE.shared.car", 0x448),
    (0x31A424, "race.owner.RACE.shared.car", 0x44C),
    (0x31A428, "race.owner.RACE.shared.car", 0x450),
    (0x31A42C, "race.owner.RACE.shared.car", 0x454),
    (0x31A430, "race.owner.RACE.shared.car", 0x458),
    (0x31A434, "race.owner.RACE.shared.car", 0x45C),
    (0x31A610, "race.owner.RACE.shared.car", 0x638),
    (0x31A614, "race.owner.RACE.shared.car", 0x63C),
    (0x31A618, "race.owner.RACE.shared.car", 0x640),
    (0x31A61C, "race.owner.RACE.shared.car", 0x644),
    (0x31A668, "race.owner.RACE.shared.car", 0x690),
    (0x31A670, "race.owner.RACE.shared.car", 0x698),
    (0x31A674, "race.owner.RACE.shared.car", 0x69C),
    (0x31A688, "race.owner.RACE.shared.car", 0x6B0),
    (0x31A690, "race.owner.RACE.shared.car", 0x6B8),
    (0x31A6A0, "race.owner.RACE.shared.car", 0x6C8),
    (0x31A8E4, "race.owner.RACE.shared.car", 0x90C),
    (0x31A920, "race.owner.RACE.shared.car", 0x948),
    (0x31A924, "race.owner.RACE.shared.car", 0x94C),
    (0x31A930, "race.owner.RACE.shared.car", 0x958),
    (0x31A984, "race.owner.RACE.shared.car", 0x9AC),
    (0x31A988, "race.owner.RACE.shared.car", 0x9B0),
    (0x31A994, "race.owner.RACE.shared.car", 0x9BC),
    (0x31AA88, "race.owner.RACE.shared.car", 0xAB0),
    (0x31ACC8, "race.owner.RACE.shared.car", 0xCF0),
    (0x31ACD4, "race.owner.RACE.shared.car", 0xCFC),
    (0x31AEE0, "race.owner.RACE.shared.car", 0xF08),
    (0x31AF18, "race.owner.RACE.shared.car", 0xF40),
    (0x31AF1C, "race.owner.RACE.shared.car", 0xF44),
    (0x31C708, "race.owner.RACE.shared.work", 0x12F8),
];

/// The path of a field with its array indices taken out: `.forces[3].force[1]` is
/// `.forces.force`. A run of like cells is named by its first cell, so a cell is looked for
/// in the runs whose path matches in this form.
fn without_indices(path: &str) -> String {
    let mut out = String::new();
    let mut inside = false;
    for c in path.chars() {
        match c {
            '[' => inside = true,
            ']' => inside = false,
            _ if !inside => out.push(c),
            _ => {}
        }
    }
    out
}

/// A stretch of a store's bytes under one field path.
struct Run {
    offset: usize,
    len: usize,
    path: String,
}

/// The runs a store is cut into, in order.
fn runs_of(store: &dyn super::registry::Walk) -> Vec<Run> {
    let size = store.size();
    let mut runs = Vec::new();
    let mut at = 0;
    while at < size {
        let (start, run) = store.cell_run_at(at);
        let end = (start + run.len()).min(size);
        assert!(end > at, "a run that does not advance, at byte {at}");
        let mut path = String::new();
        store.field_path_at(at, &mut path);
        if path.is_empty() {
            path = format!("@{at}");
        }
        runs.push(Run {
            offset: at,
            len: end - at,
            path,
        });
        at = end;
    }
    runs
}

/// The offset in `store` of the cell named `path`.
fn offset_in_store(store: &dyn super::registry::Walk, runs: &[Run], path: &str) -> usize {
    let wanted = without_indices(path);
    for run in runs {
        if without_indices(&run.path) != wanted {
            continue;
        }
        for at in run.offset..run.offset + run.len {
            let mut here = String::new();
            store.field_path_at(at, &mut here);
            if here == path {
                return at;
            }
        }
    }
    panic!("no field {path} in a store of the registry");
}

/// Every place of the race block: its offset, the store and the offset in the store.
fn table() -> &'static [(usize, &'static str, usize)] {
    static TABLE: OnceLock<Vec<(usize, &'static str, usize)>> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut runs: HashMap<&str, Vec<Run>> = HashMap::new();
        let mut table: Vec<(usize, &'static str, usize)> = FIELDS
            .iter()
            .map(|&(block, name, path)| {
                let store = super::state::walk(name);
                let runs = runs.entry(name).or_insert_with(|| runs_of(store));
                (block, name, offset_in_store(store, runs, path))
            })
            .collect();
        // The byte rows were recorded in the 1999 layout of the car and of the race work.
        table.extend(
            BYTES
                .iter()
                .map(|&(block, name, at)| (block, name, in_current_layout(name, at))),
        );
        table.sort_by_key(|p| p.0);
        table
    })
}

/// The store and the offset in it of the place at `off`.
pub fn place(off: usize) -> (&'static str, usize) {
    let table = table();
    match table.binary_search_by_key(&off, |p| p.0) {
        Ok(i) => (table[i].1, table[i].2),
        Err(_) => panic!("race block place {off:#X} is not in test_support::race_places"),
    }
}

/// The offset in the store as the records lie now: the places were recorded in the 1999
/// layout of the car and of the race work.
pub fn in_current_layout(store: &str, at: usize) -> usize {
    use crate::physics::car_store::{Car, from_1999};
    match store {
        "race.owner.RACE.shared.car" => from_1999(at),
        "race.owner.RACE.replay.car_blocks" => {
            (at / 0xFE0) * size_of::<Car>() + from_1999(at % 0xFE0)
        }
        "race.owner.RACE.shared.work" => crate::race::work_store::from_1999(at),
        _ => at,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_places_are_found_once_each() {
        let _g = crate::test_support::test_lock();
        let table = super::table();
        assert_eq!(table.len(), super::FIELDS.len() + super::BYTES.len());
        for pair in table.windows(2) {
            assert!(
                pair[0].0 < pair[1].0,
                "place {:#X} twice or out of order",
                pair[1].0
            );
        }
        for &(_, name, at) in table {
            assert!(at < crate::test_support::state::store(name).size());
        }
    }
}
