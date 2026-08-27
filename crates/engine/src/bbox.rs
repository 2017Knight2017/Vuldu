use wad_parser::{AABB, Level, Line, SlopeType};

pub fn p_point_on_line_side(x: f32, y: f32, line: &Line, level: &Level) -> i32 {
	let v1 = level.geom.vertices[line.v1.0];

	if line.delta.0 == 0.0 {
		if x <= v1.0 {
			return if line.delta.1 > 0.0 { 1 } else { 0 };
		}
		return if line.delta.1 < 0.0 { 1 } else { 0 };
	}

	if line.delta.1 == 0.0 {
		if y <= v1.1 {
			return if line.delta.0 < 0.0 { 1 } else { 0 };
		}
		return if line.delta.0 > 0.0 { 1 } else { 0 };
	}

	let dx = x - v1.0;
	let dy = y - v1.1;

	let left = line.delta.1 * dx;
	let right = dy * line.delta.0;

	if right < left { 0 } else { 1 }
}

pub fn p_box_on_line_side(bbox: &AABB, line: &Line, level: &Level) -> i32 {
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
