pub const ANG45: u32 = 0x20000000;
pub const ANG90: u32 = 0x40000000;
pub const ANG180: u32 = 0x80000000;

const ATANSCOEFFICIENTS: [f32; 3] = [
	0.31805385, -0.10129062, 0.04505315,
];

// from Kaze Emmanuar's video
pub fn fast_atan2(mut dx: f32, mut dy: f32) -> u32 {
    let mut arctan_approx = ANG45;

    if dy + dy < -dx {
        (dx, dy) = (-dx, -dy);
        arctan_approx += ANG180;
    }

    if dy > dx + dx {
        (dx, dy) = (dy, -dx);
        arctan_approx += ANG90;
    }

    let updated_y = dx - dy;
    if dy <= updated_y {
        arctan_approx -= ANG45;
    } else {
        (dx, dy) = (dx + dy, -updated_y);
    }

    if dx == 0.0 {
        return arctan_approx;
    }

    let z = dy / dx;
    let z2 = z * z;
    let [c0, c1, c2] = ATANSCOEFFICIENTS;
    let result = z * (c0 + z2 * (c1 + z2 * c2));

	arctan_approx.wrapping_add_signed((result * ANG180 as f32) as i32)
}
