use wad_parser::{Level, LineFlags, LineId, NF_SUBSECTOR, SubsectorId};
use crate::{CurrentSector, PLAYERHEIGHT, Pass, Position, Traversal};

#[derive(Debug, Clone, Copy)]
pub struct DivLine {
    pub x: f32,
    pub y: f32,
    pub dx: f32,
    pub dy: f32,
}

pub fn p_divline_side(x: f32, y: f32, node: &DivLine) -> i32 {
    if node.dx == 0.0 {
        if x == node.x {
            return 2;
        }
        if x <= node.x {
            return if node.dy > 0.0 { 1 } else { 0 };
        }
        return if node.dy < 0.0 { 1 } else { 0 };
    }

    if node.dy == 0.0 {
        if y == node.y {
            return 2;
        }
        if y <= node.y {
            return if node.dx < 0.0 { 1 } else { 0 };
        }
        return if node.dx > 0.0 { 1 } else { 0 };
    }

    let dx = x - node.x;
    let dy = y - node.y;

    let left = node.dy * dx;
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
    let den = v1.dy * v2.dx - v1.dx * v2.dy;

    if den == 0.0 {
        return 0.0;
    }

    let num = (v1.x - v2.x) * v1.dy + (v2.y - v1.y) * v1.dx;
    num / den
}

pub struct SightContext<'a> {
    pub strace: DivLine,
    pub t2x: f32,
    pub t2y: f32,
    pub sight_ystart: f32,
    pub top_slope: f32,
    pub bottom_slope: f32,
    pub pass: Pass<'a>,
}

impl<'a> SightContext<'a> {
    pub fn new(t1_pos: (f32, f32, f32), t1_height: f32, t2_pos: (f32, f32, f32), t2_height: f32, pass: Pass<'a>) -> Self {
        let sight_ystart = t1_pos.2 + t1_height - (t1_height * 0.25);
        let top_slope = (t2_pos.2 + t2_height) - sight_ystart;
        let bottom_slope = t2_pos.2 - sight_ystart;

        let strace = DivLine {
            x: t1_pos.0,
            y: t1_pos.1,
            dx: t2_pos.0 - t1_pos.0,
            dy: t2_pos.1 - t1_pos.1,
        };

        Self {
            strace,
            t2x: t2_pos.0,
            t2y: t2_pos.1,
            sight_ystart,
            top_slope,
            bottom_slope,
            pass,
        }
    }

    pub fn cross_subsector(&mut self, subsector_id: SubsectorId, level: &Level) -> bool {
        let subsector = &level.geom.subsectors[subsector_id.0];
        let mut seg_idx = subsector.firstseg as usize;

        for _ in 0..subsector.numsegs {
            let seg = &level.geom.segs[seg_idx];
            seg_idx += 1;

            let line_id = LineId(seg.linedef as usize);
            let line = &level.geom.lines[line_id.0];

            if !line.flags.contains(LineFlags::TWO_SIDED) {
                return false;
            }

            let (front_side_id, back_side_id) = match line.sides {
                (Some(front_id), Some(back_id)) => (front_id, back_id),
                _ => return false
            };

            let front_sector = &level.state.sectors[level.geom.sides[front_side_id.0].sector.0];
            let back_sector = &level.state.sectors[level.geom.sides[back_side_id.0].sector.0];

            // We already checked whether sides are None,
            // so unwrap() is safe here.
            let open = level.get_opening(line_id).unwrap();  

            if open.floor_high >= open.top {
                return false;
            }

			let seg_v1_x = level.geom.vertices[seg.v1 as usize].0;
			let seg_v1_y = level.geom.vertices[seg.v1 as usize].1;
			let seg_v2_x = level.geom.vertices[seg.v2 as usize].0;
			let seg_v2_y = level.geom.vertices[seg.v2 as usize].1;

            let s1 = p_divline_side(seg_v1_x, seg_v1_y, &self.strace);
            let s2 = p_divline_side(seg_v2_x, seg_v2_y, &self.strace);

            if s1 == s2 {
                continue;
            }

            let divl = DivLine {
                x: seg_v1_x,
                y: seg_v1_y,
                dx: seg_v2_x - seg_v1_x,
                dy: seg_v2_y - seg_v1_y,
            };

            let s1 = p_divline_side(self.strace.x, self.strace.y, &divl);
            let s2 = p_divline_side(self.t2x, self.t2y, &divl);

            if s1 == s2 {
                continue;
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
        if (bspnum & NF_SUBSECTOR) != 0 {
            if bspnum == u16::MAX as usize {
                return self.cross_subsector(SubsectorId(0), level);
            } else {
                return self.cross_subsector(SubsectorId(bspnum & !NF_SUBSECTOR), level);
            }
        }

        let bsp = &level.geom.nodes[bspnum];
		let bsp_divline = DivLine { x: bsp.x as f32, y: bsp.y as f32, dx: bsp.dx as f32, dy: bsp.dy as f32 };

        let mut side = p_divline_side(self.strace.x, self.strace.y, &bsp_divline);
        if side == 2 {
            side = 0;
        }

        if !self.cross_bsp_node(bsp.children[side as usize] as usize, level) {
            return false;
        }

        if side == p_divline_side(self.t2x, self.t2y, &bsp_divline) {
            return true;
        }

        self.cross_bsp_node(bsp.children[(side ^ 1) as usize] as usize, level)
    }
}


pub fn p_check_sight(
    pos: &Position, 
    cur_sector: &CurrentSector,
	height: f32, 
    player_pos: &Position,
    player_cur_sector: &CurrentSector, 
	level: &Level,
	traversal: &mut Traversal
) -> bool {
	if let Some(_) = level.geom.reject_table.0 {
        if level.geom.reject_table.is_rejected(cur_sector.0, player_cur_sector.0, level.state.sectors.len()) {
            return false;
        }
    }

	let pass = traversal.begin();

    let mut context = SightContext::new(
		(pos.x, pos.y, pos.z), 
		height, 
		(player_pos.x, player_pos.y, player_pos.z), 
		PLAYERHEIGHT, 
		pass
	);

    if level.geom.nodes.is_empty() {
        return true;
    }

    let head_node = level.geom.nodes.len() - 1;
    context.cross_bsp_node(head_node, level)
}