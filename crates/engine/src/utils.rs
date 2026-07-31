use crate::{MELEERANGE, MobjFlags, MobjNum, MobjType, MonsterRotation, Position, Random, point_to_angle};

pub fn aprox_xz_distance(src: (f32, f32), dst: (f32, f32)) -> f32 {
	let dx = (src.0 - dst.0).abs();
	let dz = (src.1 - dst.1).abs();

	dx + dz - dx.min(dz)*0.5
}

pub fn aprox_xyz_distance(src: (f32, f32, f32), dst: (f32, f32, f32)) -> f32 {
	aprox_xz_distance((src.0, src.2), (dst.0, dst.2)) + (src.1 - dst.1).abs()*0.5
}

pub fn in_fov(pos: &Position, rot: &MonsterRotation, player_pos: &Position) -> bool {
	let angle_to_player = point_to_angle(pos.x - player_pos.x, pos.z - player_pos.z) >> 29;

    return angle_to_player == (rot.move_dir.wrapping_sub(2) & 0b111) 
        || angle_to_player == (rot.move_dir.wrapping_sub(1) & 0b111) 
        || angle_to_player == rot.move_dir 
        || angle_to_player == ((rot.move_dir + 1) & 0b111) 
        || angle_to_player == ((rot.move_dir + 2) & 0b111);
}

pub fn p_check_melee_range(pos: &Position, target_pos: &Position, target_radius: f32) -> bool {
    // TODO: if !p_check_sight(...) { return false; }

    let dist = aprox_xz_distance((target_pos.x, target_pos.z), (pos.x, pos.z));

    dist < MELEERANGE - 20.0 + target_radius
}

pub fn p_check_missile_range(
    pos: &Position,
    mobj_type: &mut MobjType,
    target_pos: &Position, 
    random: &mut Random,
    is_melee_state_none: bool,
) -> bool {
    // TODO: if !p_check_sight(...) { return false; }
	
    if mobj_type.flags.contains(MobjFlags::JUST_HIT) {
        mobj_type.flags.remove(MobjFlags::JUST_HIT);
        return true;
    }

    let mut dist = aprox_xz_distance((target_pos.x, target_pos.z), (pos.x, pos.z)) - 64.0;

    if is_melee_state_none {
        dist -= 128.0;
    }

    match mobj_type.type_ {
        MobjNum::Vile => {
            if dist > 14.0 * 64.0 {
                return false;
            }
        },

        MobjNum::Undead => {
            if dist < 196.0 {
                return false;
            } else {
                dist /= 2.0;
            }
        },

        MobjNum::Cyborg | MobjNum::Spider | MobjNum::Skull => {
            dist /= 2.0;
        }

        _ => {}
    }

    if dist > 200.0 { dist = 200.0; }

    if mobj_type.type_ == MobjNum::Cyborg && dist > 160.0 {
        dist = 160.0;
    }

    return (random.p() as f32) >= dist;
}

pub fn p_new_chase_dir(pos: &Position, rot: &mut MonsterRotation, target_pos: &Position, random: &mut Random) {
    let delta_x = target_pos.x - pos.x;
    let delta_y = target_pos.z - pos.z;

    let d1 = if delta_x > 10.0 {
        if delta_y > 10.0 { 1 } else if delta_y < -10.0 { 7 } else { 0 }
    } else if delta_x < -10.0 {
        if delta_y > 10.0 { 3 } else if delta_y < -10.0 { 5 } else { 4 } 
    } else {
        if delta_y > 10.0 { 2 } else if delta_y < -10.0 { 6 } else { 2 } 
    };

    if d1 != 8 && random.p() & 0b1 != 0 {
        rot.move_dir = d1;
    } else {
        rot.move_dir = (random.p() & 0b111) as u32;
    }
}
