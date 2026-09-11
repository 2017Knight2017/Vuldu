use wad_parser::{AABB, Level, Line, LineId, MAPBLOCKSIZE, SlopeType};

use crate::Pass;

#[derive(Debug, Clone, Copy)]
pub(crate) struct DivLine {
	pub x: f32,
	pub z: f32,
	pub dx: f32,
	pub dz: f32,
}

pub(crate) fn p_divline_side(x: f32, z: f32, node: DivLine) -> i32 {
	if node.dx == 0.0 {
		if x == node.x {
			return 2;
		}
		if x <= node.x {
			return if node.dz > 0.0 { 1 } else { 0 };
		}
		return if node.dz < 0.0 { 1 } else { 0 };
	}

	if node.dz == 0.0 {
		if z == node.z {
			return 2;
		}
		if z <= node.z {
			return if node.dx < 0.0 { 1 } else { 0 };
		}
		return if node.dx > 0.0 { 1 } else { 0 };
	}

	let px = x - node.x;
	let py = z - node.z;

	let left = node.dz * px;
	let right = py * node.dx;

	if right < left {
		0
	} else if (left - right).abs() < f32::EPSILON {
		2
	} else {
		1
	}
}

pub(crate) fn p_point_on_line_side(x: f32, y: f32, line: Line, level: &Level) -> i32 {
	let v1 = level.geom.vertices[line.v1.0];
	p_divline_side(
		x,
		y,
		DivLine {
			x: v1.0,
			z: v1.1,
			dx: line.delta.0,
			dz: line.delta.1,
		},
	)
}

pub(crate) fn p_box_on_line_side(bbox: AABB, line: Line, level: &Level) -> i32 {
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

#[derive(Debug, Clone, Copy)]
pub struct Intercept {
	pub frac: f32,
	pub line_id: LineId,
}

pub(crate) fn p_intercept_vector(v2: DivLine, v1: DivLine) -> f32 {
	let den = v1.dz * v2.dx - v1.dx * v2.dz;

	if den == 0.0 {
		return 0.0;
	}

	let num = (v1.x - v2.x) * v1.dz + (v2.z - v1.z) * v1.dx;
	num / den
}

pub(crate) fn collect_line_intercepts(
	level: &Level,
	pass: &mut Pass,
	x1: f32,
	z1: f32,
	x2: f32,
	z2: f32,
	out: &mut Vec<Intercept>,
) {
	let bm = &level.geom.blockmap;

	let dx = x2 - x1;
	let dz = z2 - z1;

	let (start_col, start_row) = bm.world_to_grid(x1, z1);
	let (mut col, mut row) = (start_col as isize, start_row as isize);

	let step_col: isize = if dx > 0.0 {
		1
	} else if dx < 0.0 {
		-1
	} else {
		0
	};
	let step_row: isize = if dz > 0.0 {
		1
	} else if dz < 0.0 {
		-1
	} else {
		0
	};

	let (mut t_next_col, t_step_col) = if dx != 0.0 {
		let edge =
			bm.origin_x as f32 + (if dx > 0.0 { col + 1 } else { col }) as f32 * MAPBLOCKSIZE;
		((edge - x1) / dx, (MAPBLOCKSIZE / dx).abs())
	} else {
		(f32::INFINITY, f32::INFINITY)
	};

	let (mut t_next_row, t_step_row) = if dz != 0.0 {
		let edge =
			bm.origin_z as f32 + (if dz > 0.0 { row + 1 } else { row }) as f32 * MAPBLOCKSIZE;
		((edge - z1) / dz, (MAPBLOCKSIZE / dz).abs())
	} else {
		(f32::INFINITY, f32::INFINITY)
	};

	loop {
		if col < 0 || row < 0 || col >= bm.col_num as isize || row >= bm.row_num as isize {
			return;
		}

		let idx = row as usize * bm.col_num + col as usize;

		for &line_id in bm.blocklists[idx].iter() {
			if !pass.visit_line(line_id) {
				continue;
			}

			let line = level.geom.lines[line_id.0];
			let v1 = level.geom.vertices[line.v1.0];
			let v2 = level.geom.vertices[line.v2.0];

			let s1 = p_divline_side(
				v1.0,
				v1.1,
				DivLine {
					x: x1,
					z: z1,
					dx,
					dz,
				},
			);
			let s2 = p_divline_side(
				v2.0,
				v2.1,
				DivLine {
					x: x1,
					z: z1,
					dx,
					dz,
				},
			);

			if s1 == s2 {
				continue;
			}

			let frac = p_intercept_vector(
				DivLine {
					x: x1,
					z: z1,
					dx,
					dz,
				},
				DivLine {
					x: v1.0,
					z: v1.1,
					dx: line.delta.0,
					dz: line.delta.1,
				},
			);

			if !(0.0..=1.0).contains(&frac) {
				continue;
			}

			out.push(Intercept { frac, line_id });
		}

		if t_next_col > 1.0 && t_next_row > 1.0 {
			return;
		}

		if t_next_col < t_next_row {
			t_next_col += t_step_col;
			col += step_col;
		} else {
			t_next_row += t_step_row;
			row += step_row;
		}
	}
}
