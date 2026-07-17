use crate::{
	constants::{TANTABLE, ANG90, ANG180}
};

pub fn point_to_angle(dx: f32, dy: f32) -> u32 {
    if dx == 0.0 && dy == 0.0 {
        return 0;
    }

    let ax = dx.abs();
    let ay = dy.abs();
    let mut angle: u32;

    if ax > ay {
        // 0..45 degrees
        let index = ((ay / ax) * 2048.0) as usize;
        angle = TANTABLE[index.min(2048)];
    } else {
        // 45..90 degrees
        let index = ((ax / ay) * 2048.0) as usize;
        angle = ANG90.wrapping_sub(TANTABLE[index.min(2048)]);
    }

    if dx < 0.0 {
        if dy < 0.0 {
            // 180..270 degrees
            angle = ANG180.wrapping_add(angle);
        } else {
            // 90..180 degrees
            angle = ANG180.wrapping_sub(angle);
        }
    } else {
        if dy < 0.0 {
			// 270..360 degrees
			angle = angle.wrapping_neg();
        }
    }

    angle
}