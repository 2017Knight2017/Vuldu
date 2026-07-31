use wad_parser::{DoomMap, NF_SUBSECTOR};
use crate::{CurrentSector, DynLinedef, LinedefFlags, Position};

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

pub struct SightContext {
    pub strace: DivLine,
    pub t2x: f32,
    pub t2y: f32,
    pub sight_zstart: f32,
    pub top_slope: f32,
    pub bottom_slope: f32,
    pub valid_count: u32,
}

impl SightContext {
    pub fn new(t1_pos: (f32, f32, f32), t1_height: f32, t2_pos: (f32, f32, f32), t2_height: f32, valid_count: u32) -> Self {
        let sight_zstart = t1_pos.2 + t1_height - (t1_height * 0.25);
        let top_slope = (t2_pos.2 + t2_height) - sight_zstart;
        let bottom_slope = t2_pos.2 - sight_zstart;

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
            sight_zstart,
            top_slope,
            bottom_slope,
            valid_count,
        }
    }

    pub fn cross_subsector(&mut self, num: usize, map: &DoomMap, linedefs: &mut [DynLinedef]) -> bool {
        let sub = &map.subsectors[num];
        let mut seg_idx = sub.firstseg as usize;

        for _ in 0..sub.numsegs {
            let seg = &map.segs[seg_idx];
            seg_idx += 1;

            let line = &mut linedefs[seg.linedef as usize];

            if line.valid_count == self.valid_count {
                continue;
            }
            line.valid_count = self.valid_count;

			let seg_v1_x = map.vertices[seg.v1 as usize].x as f32;
			let seg_v1_y = map.vertices[seg.v1 as usize].y as f32;
			let seg_v2_x = map.vertices[seg.v2 as usize].x as f32;
			let seg_v2_y = map.vertices[seg.v2 as usize].y as f32;

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

            if line.props.flags & LinedefFlags::TWO_SIDED.bits() as i16 == 0 {
                return false;
            }

	        let front_side_idx = if seg.side == 0 { line.props.sidenum[0] } else { line.props.sidenum[1] };
	        let back_side_idx = if seg.side == 0 { line.props.sidenum[1] } else { line.props.sidenum[0] };

	        if front_side_idx == u16::MAX { continue; }
	        let front_sidedef = map.sidedefs[front_side_idx as usize];
	        let front_sector_idx = front_sidedef.sector;
	        let front = map.sectors[front_sector_idx as usize];

	        let back = if back_side_idx != u16::MAX {
				map.sectors[map.sidedefs[back_side_idx as usize].sector as usize]
	        } else {
	            continue;
	        };

            if front.floorheight == back.floorheight && front.ceilingheight == back.ceilingheight {
                continue;
            }

            let open_top = front.ceilingheight.min(back.ceilingheight) as f32;
            let open_bottom = front.floorheight.max(back.floorheight) as f32;

            if open_bottom >= open_top {
                return false;
            }

            let frac = p_intercept_vector2(&self.strace, &divl);

            if front.floorheight != back.floorheight {
                let slope = (open_bottom - self.sight_zstart) / frac;
                if slope > self.bottom_slope {
                    self.bottom_slope = slope;
                }
            }

            if front.ceilingheight != back.ceilingheight {
                let slope = (open_top - self.sight_zstart) / frac;
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

    pub fn cross_bsp_node(&mut self, bspnum: usize, map: &DoomMap, linedefs: &mut [DynLinedef]) -> bool {
        if (bspnum & NF_SUBSECTOR) != 0 {
            if bspnum == u16::MAX as usize {
                return self.cross_subsector(0, map, linedefs);
            } else {
                return self.cross_subsector(bspnum & !NF_SUBSECTOR, map, linedefs);
            }
        }

        let bsp = &map.nodes[bspnum];
		let bsp_divline = DivLine { x: bsp.x as f32, y: bsp.y as f32, dx: bsp.dx as f32, dy: bsp.dy as f32 };

        let mut side = p_divline_side(self.strace.x, self.strace.y, &bsp_divline);
        if side == 2 {
            side = 0;
        }

        if !self.cross_bsp_node(bsp.children[side as usize] as usize, map, linedefs) {
            return false;
        }

        if side == p_divline_side(self.t2x, self.t2y, &bsp_divline) {
            return true;
        }

        self.cross_bsp_node(bsp.children[(side ^ 1) as usize] as usize, map, linedefs)
    }
}


pub fn p_check_sight(
    pos: &Position, 
    cur_sector: &CurrentSector,
	height: f32, 
    player_pos: &Position,
    player_cur_sector: &CurrentSector, 
	player_height: f32,
	map: &DoomMap,
    linedefs: &mut [DynLinedef], 
	valid_count: &mut u32,
) -> bool {
	if let Some(_) = map.reject_table.0 {
        if map.reject_table.is_rejected(cur_sector.0, player_cur_sector.0, map.sectors.len()) {
            return false;
        }
    }

	*valid_count = valid_count.wrapping_add(1);

	if *valid_count == 0 {
	    for line in linedefs.iter_mut() {
	        line.valid_count = 0;
	    }
	    *valid_count = 1;
	}

    let mut context = SightContext::new(
		(pos.x, pos.y, pos.z), 
		height, 
		(player_pos.x, player_pos.y, player_pos.z), 
		player_height, 
		*valid_count
	);

    if map.nodes.is_empty() {
        return true;
    }

    let head_node = map.nodes.len() - 1;
    context.cross_bsp_node(head_node, map, linedefs)
}