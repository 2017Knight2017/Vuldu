use wad_parser::{Level, LineFlags, LineId, NF_SUBSECTOR, SubsectorId};
use crate::{CurrentSector, Pass, Position, Traversal};

#[derive(Debug, Clone, Copy)]
pub struct DivLine {
    pub x: f32,
    pub z: f32,
    pub dx: f32,
    pub dz: f32,
}

pub fn p_divline_side(x: f32, y: f32, node: &DivLine) -> i32 {
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
        if y == node.z {
            return 2;
        }
        if y <= node.z {
            return if node.dx < 0.0 { 1 } else { 0 };
        }
        return if node.dx > 0.0 { 1 } else { 0 };
    }

    let dx = x - node.x;
    let dy = y - node.z;

    let left = node.dz * dx;
    let right = dy * node.dx;

    if right < left {
        0 // front side
    } else if (left - right).abs() < f32::EPSILON {
        2 // on line
    } else {
        1 // back side
    }
}

pub fn p_intercept_vector2(v2: &DivLine, v1: &DivLine) -> f32 {
    let den = v1.dz * v2.dx - v1.dx * v2.dz;

    if den == 0.0 {
        return 0.0;
    }

    let num = (v1.x - v2.x) * v1.dz + (v2.z - v1.z) * v1.dx;
    num / den
}

pub struct SightContext<'a> {
    pub strace: DivLine,
    pub t2x: f32,
    pub t2z: f32,
    pub sight_ystart: f32,
    pub top_slope: f32,
    pub bottom_slope: f32,
    pub pass: Pass<'a>,
}

impl<'a> SightContext<'a> {
    pub fn new(t1_pos: (f32, f32, f32), t1_height: f32, t2_pos: (f32, f32, f32), t2_height: f32, pass: Pass<'a>) -> Self {
        let sight_ystart = t1_pos.1 + t1_height - (t1_height * 0.25);
        let top_slope = (t2_pos.1 + t2_height) - sight_ystart;
        let bottom_slope = t2_pos.1 - sight_ystart;

        let strace = DivLine {
            x: t1_pos.0,
            z: t1_pos.2,
            dx: t2_pos.0 - t1_pos.0,
            dz: t2_pos.2 - t1_pos.2,
        };

        Self {
            strace,
            t2x: t2_pos.0,
            t2z: t2_pos.2,
            sight_ystart,
            top_slope,
            bottom_slope,
            pass,
        }
    }

    pub fn cross_subsector(&mut self, subsector_id: SubsectorId, level: &Level) -> bool {
        let subsector = &level.geom.subsectors[subsector_id.0];

        let min = subsector.firstseg as usize;
        let max = (subsector.firstseg + subsector.numsegs) as usize;

        for i in min..max {
            let seg = &level.geom.segs[i];

            let line_id = LineId(seg.linedef as usize);
            let line = &level.geom.lines[line_id.0];

            if !self.pass.visit_line(line_id) {
                continue;
            }

            let (Some(front_side_id), Some(back_side_id)) = line.sides else {
                continue
            };

            let front_sector = &level.state.sectors[level.geom.sides[front_side_id.0].sector.0];
            let back_sector = &level.state.sectors[level.geom.sides[back_side_id.0].sector.0];

            // We already checked whether sides are None,
            // so unwrap() is safe here.
            let open = level.get_opening(line_id).unwrap();  

            if open.floor_high >= open.top {
                continue;
            }

			let seg_v1_x = level.geom.vertices[seg.v1 as usize].0;
			let seg_v1_z = level.geom.vertices[seg.v1 as usize].1;
			let seg_v2_x = level.geom.vertices[seg.v2 as usize].0;
			let seg_v2_z = level.geom.vertices[seg.v2 as usize].1;

            let s1 = p_divline_side(seg_v1_x, seg_v1_z, &self.strace);
            let s2 = p_divline_side(seg_v2_x, seg_v2_z, &self.strace);

            if s1 == s2 {
                continue;
            }

            let divl = DivLine {
                x: seg_v1_x,
                z: seg_v1_z,
                dx: seg_v2_x - seg_v1_x,
                dz: seg_v2_z - seg_v1_z,
            };

            let s1 = p_divline_side(self.strace.x, self.strace.z, &divl);
            let s2 = p_divline_side(self.t2x, self.t2z, &divl);

            if s1 == s2 {
                continue;
            }

            if !line.flags.contains(LineFlags::TWO_SIDED) {
                return false;
            }

            let frac = p_intercept_vector2(&self.strace, &divl);

            if front_sector.floor_h != back_sector.floor_h {
                let slope = (open.floor_high - self.sight_ystart) / frac;
                if slope > self.bottom_slope {
                    self.bottom_slope = slope;
                }
            }

            if front_sector.ceil_h != back_sector.ceil_h {
                let slope = (open.top - self.sight_ystart) / frac;
                if slope < self.top_slope {
                    self.top_slope = slope;
                }
            }

            if self.top_slope <= self.bottom_slope {
                return false;
            }
        }

        true
    }

    pub fn cross_bsp_node(&mut self, bspnum: usize, level: &Level) -> bool {
        if bspnum & NF_SUBSECTOR != 0 {
            if bspnum == u16::MAX as usize {
                return self.cross_subsector(SubsectorId(0), level);
            } else {
                return self.cross_subsector(SubsectorId(bspnum & !NF_SUBSECTOR), level);
            }
        }

        let bsp = &level.geom.nodes[bspnum];
		let bsp_divline = DivLine { x: bsp.x as f32, z: bsp.y as f32, dx: bsp.dx as f32, dz: bsp.dy as f32 };

        let mut side = p_divline_side(self.strace.x, self.strace.z, &bsp_divline);
        if side == 2 {
            side = 0;
        }

        if !self.cross_bsp_node(bsp.children[side as usize] as usize, level) {
            return false;
        }

        if side == p_divline_side(self.t2x, self.t2z, &bsp_divline) {
            return true;
        }

        self.cross_bsp_node(bsp.children[(side ^ 1) as usize] as usize, level)
    }
}

#[allow(clippy::too_many_arguments)]
pub fn p_check_sight(
    pos: &Position, 
    cur_sector: &CurrentSector,
	height: f32, 
    target_pos: &Position,
    target_cur_sector: &CurrentSector, 
    target_height: f32,
	level: &Level,
	traversal: &mut Traversal
) -> bool {
    if level.geom.reject_table.is_rejected(cur_sector.0, target_cur_sector.0, level.state.sectors.len()) {
        return false;
    }

	let pass = traversal.begin();

    let mut context = SightContext::new(
		(pos.x, pos.y, pos.z), 
		height, 
		(target_pos.x, target_pos.y, target_pos.z), 
		target_height, 
		pass
	);

    if level.geom.nodes.is_empty() {
        return true;
    }

    let head_node = level.geom.nodes.len() - 1;
    context.cross_bsp_node(head_node, level)
}