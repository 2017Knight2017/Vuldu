use wad_parser::{AABB, Level, Line, LineId, SlopeType};

pub(crate) fn p_generic_line_side(x: f32, y: f32, x0: f32, y0: f32, dx: f32, dy: f32) -> i32 {
    if dx == 0.0 {
        if x == x0 {
            return 2;
        }
        if x <= x0 {
            return if dy > 0.0 { 1 } else { 0 };
        }
        return if dy < 0.0 { 1 } else { 0 };
    }

    if dy == 0.0 {
        if y == y0 {
            return 2;
        }
        if y <= y0 {
            return if dx < 0.0 { 1 } else { 0 };
        }
        return if dx > 0.0 { 1 } else { 0 };
    }

    let px = x - x0;
    let py = y - y0;

    let left = dy * px;
    let right = py * dx;

    if right < left {
        0
    } else if (left - right).abs() < f32::EPSILON {
        2
    } else {
        1
    }
}

pub(crate) fn p_point_on_line_side(x: f32, y: f32, line: &Line, level: &Level) -> i32 {
    let v1 = level.geom.vertices[line.v1.0];
    p_generic_line_side(x, y, v1.0, v1.1, line.delta.0, line.delta.1)
}

pub(crate) fn p_box_on_line_side(bbox: &AABB, line: &Line, level: &Level) -> i32 {
	let v1 = level.geom.vertices[line.v1.0];

	let p1: i32;
	let p2: i32;

	match line.slope {
		SlopeType::Horizontal => {
			p1 = if bbox.max_z > v1.1 { 1 } else { 0 };
			p2 = if bbox.min_z > v1.1 { 1 } else { 0 };

			if line.delta.0 < 0.0 {
				let p1_flipped = p1 ^ 1;
				let p2_flipped = p2 ^ 1;
				if p1_flipped == p2_flipped {
					return p1_flipped;
				}
				return -1;
			}
		}
		SlopeType::Vertical => {
			p1 = if bbox.max_x < v1.0 { 1 } else { 0 };
			p2 = if bbox.min_x < v1.0 { 1 } else { 0 };

			if line.delta.1 < 0.0 {
				let p1_flipped = p1 ^ 1;
				let p2_flipped = p2 ^ 1;
				if p1_flipped == p2_flipped {
					return p1_flipped;
				}
				return -1;
			}
		}
		SlopeType::Positive => {
			p1 = p_point_on_line_side(bbox.min_x, bbox.max_z, line, level);
			p2 = p_point_on_line_side(bbox.max_x, bbox.min_z, line, level);
		}
		SlopeType::Negative => {
			p1 = p_point_on_line_side(bbox.max_x, bbox.max_z, line, level);
			p2 = p_point_on_line_side(bbox.min_x, bbox.min_z, line, level);
		}
	}

	if p1 == p2 { p1 } else { -1 }
}

pub(crate) fn line_midpoint(level: &Level, line_id: LineId) -> (f32, f32) {
	let line = level.geom.lines[line_id.0];
	let (v1_x, v1_z) = level.geom.vertices[line.v1.0];
	let (v2_x, v2_z) = level.geom.vertices[line.v2.0];
	
	((v1_x + v2_x) / 2.0, (v1_z + v2_z) / 2.0)
}
