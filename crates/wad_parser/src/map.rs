use crate::*;
use bytemuck::{Pod, Zeroable};
use renderer::{Vertex};
use earcut::Earcut;
use std::collections::HashMap;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct MapVertex
{
  x: i16,
  y: i16,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct MapSidedef
{
	textureoffset: i16,
	rowoffset: i16,
  	toptexture: [u8; 8],
  	bottomtexture: [u8; 8],
  	midtexture: [u8; 8],
	sector: i16,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct MapLinedef
{
	v1: i16,
	v2: i16,
	flags: i16,
	special: i16,
	tag: i16,
	sidenum: [i16; 2]	
}

//pub mod LinedefFlags {
//	const ML_BLOCKING: i16 = 1;
//	const ML_BLOCKMONSTER: i16 = 2;
//	const ML_TWOSIDED: i16 = 4;
//	const ML_DONTPEGTOP: i16 = 8;
//	const ML_DONTPEGBOTTO: i16 = 16;
//	const ML_SECRET: i16 = 32;
//	const ML_SOUNDBLOCK: i16 = 64;
//	const ML_DONTDRAW: i16 = 128;
//	const ML_MAPPED: i16 = 256;
//}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct MapSector
{
	pub floorheight: i16,
	pub ceilingheight: i16,
  	pub floorpic: [u8; 8],
  	pub ceilingpic:[u8; 8],
	pub lightlevel: i16,
	pub special: i16,
	pub tag: i16,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct MapSubsector
{
	numsegs: i16,
	firstseg: i16	
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct MapSegment
{
	v1: i16,
	v2: i16,
	angle: i16,
	linedef: i16,
	side: i16,
	offset: i16,
}

pub const NF_SUBSECTOR: usize = 0x8000;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct MapNode
{
	x: i16,
	y: i16,
	dx: i16,
	dy: i16,
	bbox: [[i16; 4]; 2],
	children: [u16; 2],
}

#[repr(C)] 
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct MapThing
{
	pub x: i16,
	pub y: i16,
	pub angle: i16,
	pub type_: i16,
	pub options: i16,
}

#[derive(Debug, Clone, Copy)]
struct Aabb {
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
}

impl Aabb {
    fn from_polygon(poly: &[[f32; 2]]) -> Self {
        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;
        for pt in poly {
            if pt[0] < min_x { min_x = pt[0]; }
            if pt[0] > max_x { max_x = pt[0]; }
            if pt[1] < min_y { min_y = pt[1]; }
            if pt[1] > max_y { max_y = pt[1]; }
        }
        Aabb { min_x, max_x, min_y, max_y }
    }

    fn intersects(&self, other: &Self) -> bool {
        self.min_x <= other.max_x && self.max_x >= other.min_x &&
        self.min_y <= other.max_y && self.max_y >= other.min_y
    }
}

#[derive(Debug, Default)]
pub struct DoomMap {
    vertices: Vec<MapVertex>,
    linedefs: Vec<MapLinedef>,
    sidedefs: Vec<MapSidedef>,
    pub sectors: Vec<MapSector>,
    pub things: Vec<MapThing>,
	subsectors: Vec<MapSubsector>,
	segs: Vec<MapSegment>,
	nodes: Vec<MapNode>
}

impl DoomMap {
    pub fn from_wad(wad_manager: &WadManager, map_name: &String) -> Result<Self, String> {
        let mut map = DoomMap::default();

    	let vertexes_bytes = wad_manager.get_data(&format!("VERTEXES_{}", map_name))?;
    	let typed_slice: &[MapVertex] = bytemuck::cast_slice(vertexes_bytes);
    	map.vertices = typed_slice.to_vec();

		let linedefs_bytes = wad_manager.get_data(&format!("LINEDEFS_{}", map_name))?;
    	let typed_slice: &[MapLinedef] = bytemuck::cast_slice(linedefs_bytes);
    	map.linedefs = typed_slice.to_vec();

		let sidedefs_bytes = wad_manager.get_data(&format!("SIDEDEFS_{}", map_name))?;
    	let typed_slice: &[MapSidedef] = bytemuck::cast_slice(sidedefs_bytes);
    	map.sidedefs = typed_slice.to_vec();

		let sectors_bytes = wad_manager.get_data(&format!("SECTORS_{}", map_name))?;
    	let typed_slice: &[MapSector] = bytemuck::cast_slice(sectors_bytes);
    	map.sectors = typed_slice.to_vec();

		let things_bytes = wad_manager.get_data(&format!("THINGS_{}", map_name))?;
    	let typed_slice: &[MapThing] = bytemuck::cast_slice(things_bytes);
    	map.things = typed_slice.to_vec();

		let ssectors_bytes = wad_manager.get_data(&format!("SSECTORS_{}", map_name))?;
    	let typed_slice: &[MapSubsector] = bytemuck::cast_slice(ssectors_bytes);
    	map.subsectors = typed_slice.to_vec();

		let segs_bytes = wad_manager.get_data(&format!("SEGS_{}", map_name))?;
    	let typed_slice: &[MapSegment] = bytemuck::cast_slice(segs_bytes);
    	map.segs = typed_slice.to_vec();

		let nodes_bytes = wad_manager.get_data(&format!("NODES_{}", map_name))?;
    	let typed_slice: &[MapNode] = bytemuck::cast_slice(nodes_bytes);
    	map.nodes = typed_slice.to_vec();

        Ok(map)
    }

	pub fn get_walls_vertices(&self, texture_ids: &HashMap<String, (u32, u32, u32)>) -> (Vec<Vertex>, Vec<u32>) {
	    let mut vertices = Vec::new();
	    let mut indices = Vec::new();

	    for seg in self.segs.iter() {
	        if seg.linedef == -1 { continue; }

	        let v1 = self.vertices[seg.v1 as usize];
	        let v2 = self.vertices[seg.v2 as usize];
	        let linedef = self.linedefs[seg.linedef as usize];

	        let front_side_idx = if seg.side == 0 { linedef.sidenum[0] } else { linedef.sidenum[1] };
	        let back_side_idx = if seg.side == 0 { linedef.sidenum[1] } else { linedef.sidenum[0] };

	        if front_side_idx == -1 { continue; }
	        let front_sidedef = self.sidedefs[front_side_idx as usize];
	        let front_sector_id = front_sidedef.sector;
	        let front_sector = self.sectors[front_sector_id as usize];

	        let back_sector = if back_side_idx != -1 {
	            let back_sidedef = self.sidedefs[back_side_idx as usize];
	            Some((back_sidedef, self.sectors[back_sidedef.sector as usize]))
	        } else {
	            None
	        };

	        let dx = (v2.x - v1.x) as f32;
	        let dy = (v2.y - v1.y) as f32;
	        let wall_length = (dx * dx + dy * dy).sqrt();
	        let tex_offset = front_sidedef.textureoffset as f32;

	        let mut add_wall_quad = |y_low: f32, y_high: f32, tex_name: &str, v_top_align: bool| {
	            if tex_name == "-" || tex_name.is_empty() { return; }
			
	            let (tex_id, tex_width, tex_height) = *texture_ids.get(tex_name).unwrap_or(&(0, 64, 64));
	            let wall_height = y_high - y_low;
	            if wall_height <= 0.0 { return; }

	            let u_start = (seg.offset as f32 + tex_offset) / tex_width as f32;
	            let u_end = u_start + (wall_length / tex_width as f32);

	            let (v_start, v_end) = if v_top_align {
	                (0.0, wall_height / tex_height as f32)
	            } else {
	                (-(wall_height / tex_height as f32), 0.0)
	            };

	            let start_idx = vertices.len() as u32;

	            let clamped_light = front_sector.lightlevel.clamp(0, 255) as f32;
	            let modern_light = clamped_light / 255.0;
	            let colormap_idx = 31 - ((clamped_light / 8.0).floor() as u32).clamp(0, 31);

	            vertices.push(Vertex { 
	                pos: [-(v1.x as f32), y_low, v1.y as f32],
	                texture_pos: [u_start, v_end],
					light_level: modern_light,
	                texture_id: tex_id,
	                colormap_idx,
	            });

	            vertices.push(Vertex { 
	                pos: [-(v2.x as f32), y_low, v2.y as f32],
	                texture_pos: [u_end, v_end],
					light_level: modern_light,
	                texture_id: tex_id,
	                colormap_idx,
	            });

	            vertices.push(Vertex { 
	                pos: [-(v1.x as f32), y_high, v1.y as f32],
	                texture_pos: [u_start, v_start],
					light_level: modern_light,
	                texture_id: tex_id,
	                colormap_idx,
	            });

	            vertices.push(Vertex { 
	                pos: [-(v2.x as f32), y_high, v2.y as f32],
	                texture_pos: [u_end, v_start],
					light_level: modern_light,
	                texture_id: tex_id,
	                colormap_idx,
	            });
			
	            indices.push(start_idx + 0);
	            indices.push(start_idx + 1);
	            indices.push(start_idx + 2);

	            indices.push(start_idx + 2);
	            indices.push(start_idx + 1);
	            indices.push(start_idx + 3);
	        };

	        let mid_tex = String::from_utf8_lossy(&front_sidedef.midtexture).trim_matches('\0').trim().to_uppercase();
	        let top_tex = String::from_utf8_lossy(&front_sidedef.toptexture).trim_matches('\0').trim().to_uppercase();
	        let bottom_tex = String::from_utf8_lossy(&front_sidedef.bottomtexture).trim_matches('\0').trim().to_uppercase();

	        match back_sector {
	            None => {
	                add_wall_quad(front_sector.floorheight as f32, front_sector.ceilingheight as f32, &mid_tex, true);
	            },
	            Some((_, b_sector)) => {
	                if front_sector.ceilingheight > b_sector.ceilingheight {
	                    add_wall_quad(b_sector.ceilingheight as f32, front_sector.ceilingheight as f32, &top_tex, true);
	                }

	                if front_sector.floorheight < b_sector.floorheight {
	                    add_wall_quad(front_sector.floorheight as f32, b_sector.floorheight as f32, &bottom_tex, false);
	                }

	                if mid_tex != "-" && !mid_tex.is_empty() {
	                    let mid_low = f32::max(front_sector.floorheight as f32, b_sector.floorheight as f32);
	                    let mid_high = f32::min(front_sector.ceilingheight as f32, b_sector.ceilingheight as f32);
	                    add_wall_quad(mid_low, mid_high, &mid_tex, true);
	                }
	            }
	        }
	    }

	    (vertices, indices)
	}

	pub fn get_flats_vertices(&self, texture_ids: &HashMap<String, (u32, u32, u32)>) -> (Vec<Vertex>, Vec<u32>) {
	    let mut vertices: Vec<Vertex> = Vec::new();
	    let mut indices: Vec<u32> = Vec::new();

		let mut sector_to_linedefs: HashMap<i16, Vec<&MapLinedef>> = HashMap::with_capacity(self.sectors.len());
    	for linedef in self.linedefs.iter() {
    	    if linedef.sidenum[0] != -1 {
    	        if let Some(side) = self.sidedefs.get(linedef.sidenum[0] as usize) {
    	            sector_to_linedefs.entry(side.sector).or_default().push(linedef);
    	        }
    	    }
    	    if linedef.sidenum[1] != -1 {
    	        if let Some(side) = self.sidedefs.get(linedef.sidenum[1] as usize) {
    	            sector_to_linedefs.entry(side.sector).or_default().push(linedef);
    	        }
    	    }
    	}

	    for (sector_id, sector) in self.sectors.iter().enumerate() {
	        let current_sector_id = sector_id as i16;
			let sector_linedefs = match sector_to_linedefs.get(&current_sector_id) {
        	    Some(list) => list,
        	    None => continue,
        	};
	        let mut edges: Vec<([f32; 2], [f32; 2])> = Vec::with_capacity(sector_linedefs.len() * 2);

	        for linedef in sector_linedefs {
	            let v1 = self.vertices[linedef.v1 as usize];
	            let v2 = self.vertices[linedef.v2 as usize];
	            let p1 = [v1.x as f32, v1.y as f32];
	            let p2 = [v2.x as f32, v2.y as f32];

	            if linedef.sidenum[0] != -1 {
	                if let Some(side) = self.sidedefs.get(linedef.sidenum[0] as usize) {
	                    if side.sector == current_sector_id { edges.push((p1, p2)); }
	                }
	            }
			
	            if linedef.sidenum[1] != -1 {
	                if let Some(side) = self.sidedefs.get(linedef.sidenum[1] as usize) {
	                    if side.sector == current_sector_id { edges.push((p2, p1)); }
	                }
	            }
	        }

	        if edges.is_empty() { continue; }

	        let mut polygon_loops: Vec<Vec<[f32; 2]>> = Vec::new();

	        while !edges.is_empty() {
	            let mut current_loop = Vec::new();
	            let (p1, p2) = edges.swap_remove(0);
	            current_loop.push(p1);
	            let mut current_tip = p2;

	            let mut stuck = false;
	            while !stuck && !edges.is_empty() {
	                let mut found_idx = None;
	                for (idx, edge) in edges.iter().enumerate() {
	                    if (edge.0[0] - current_tip[0]).abs() < 0.1 && (edge.0[1] - current_tip[1]).abs() < 0.1 {
	                        found_idx = Some(idx); 
	                        break;
	                    }
	                }

	                if let Some(idx) = found_idx {
	                    let next_edge = edges.swap_remove(idx);
	                    current_loop.push(current_tip);
	                    current_tip = next_edge.1;
	                } else {
	                    stuck = true;
	                }
	            }
	            current_loop.push(current_tip);
	            current_loop.dedup_by(|a, b| (a[0] - b[0]).abs() < 0.1 && (a[1] - b[1]).abs() < 0.1);
			
	            if current_loop.len() >= 3 {
	                let first = current_loop.first().unwrap();
	                let last = current_loop.last().unwrap();
	                if (first[0] - last[0]).abs() < 0.5 && (first[1] - last[1]).abs() < 0.5 {
	                    current_loop.pop();
	                }
	                if current_loop.len() >= 3 {
	                    let cleaned = clean_polygon(&current_loop);
	                    if cleaned.len() >= 3 {
	                        polygon_loops.push(cleaned);
	                    }
	                }
	            }
	        }

	        if polygon_loops.is_empty() { continue; }

	        let calc_true_area = |poly: &Vec<[f32; 2]>| -> f32 {
	            let len = poly.len();
	            if len < 3 { return 0.0; }
	            let mut area = 0.0;
	            for i in 0..len {
	                let next = (i + 1) % len;
	                area += poly[i][0] * poly[next][1] - poly[next][0] * poly[i][1];
	            }
	            area.abs()
	        };

	        polygon_loops.sort_by(|a, b| {
	            let area_a = calc_true_area(a);
	            let area_b = calc_true_area(b);
	            area_b.partial_cmp(&area_a).unwrap_or(std::cmp::Ordering::Equal)
	        });

	        let mut outer_sectors: Vec<(Vec<[f32; 2]>, Aabb)> = Vec::new();
	        let mut hole_loops: Vec<Vec<[f32; 2]>> = Vec::new();

	        for poly_loop in polygon_loops.iter().cloned() {
				let poly_aabb = Aabb::from_polygon(&poly_loop);
	            let mut is_hole = false;
	            for (outer, outer_aabb) in &outer_sectors {
            	    if poly_aabb.intersects(outer_aabb) {
            	        if poly_loop.iter().any(|&pt| point_in_polygon(pt, outer)) {
            	            is_hole = true;
            	            break;
            	        }
            	    }
            	}

	            if is_hole {
	                hole_loops.push(poly_loop);
	            } else {
	                outer_sectors.push((poly_loop, poly_aabb));
	            }
	        }

	        for (mut outer_loop, outer_aabb) in outer_sectors {
	            if outer_loop.len() < 3 { continue; }

	            let mut flat_points = Vec::new();
	            let mut hole_indices = Vec::new();

	            let mut outer_area = 0.0;
	            let o_len = outer_loop.len();
	            for i in 0..o_len {
	                let next = (i + 1) % o_len;
	                outer_area += outer_loop[i][0] * outer_loop[next][1] - outer_loop[next][0] * outer_loop[i][1];
	            }
	            if outer_area < 0.0 { outer_loop.reverse(); }

	            for pt in &outer_loop { flat_points.push(*pt); }

	            for hole in &hole_loops {
					let hole_aabb = Aabb::from_polygon(hole);
                	if !outer_aabb.intersects(&hole_aabb) { continue; }

	                if !hole.iter().any(|&pt| point_in_polygon(pt, &outer_loop)) { continue; }

	                let mut hole_copy = hole.clone();
	                if hole_copy.len() < 3 { continue; }
				
	                let current_hole_start = flat_points.len() as u32;
	                hole_indices.push(current_hole_start);
				
	                let mut h_area = 0.0;
	                let h_len = hole_copy.len();
	                for i in 0..h_len {
	                    let next = (i + 1) % h_len;
	                    h_area += hole_copy[i][0] * hole_copy[next][1] - hole_copy[next][0] * hole_copy[i][1];
	                }
	                if h_area > 0.0 { hole_copy.reverse(); }
	                for pt in hole_copy { flat_points.push(pt); }
	            }
			
	            let mut sector_indices: Vec<u32> = Vec::new();
	            let mut earcut = Earcut::new();
	            earcut.earcut(flat_points.iter().copied(), &hole_indices, &mut sector_indices);
			
	            if sector_indices.is_empty() { 
	                println!("[WARN] Sector {}: Earcut failed to triangulate polygon!", sector_id);
	                continue; 
	            }

	            let floor_texture_name = String::from_utf8_lossy(&sector.floorpic).trim_matches('\0').trim().to_uppercase();
	            let ceil_texture_name = String::from_utf8_lossy(&sector.ceilingpic).trim_matches('\0').trim().to_uppercase();
	            let floor_texture_id = texture_ids.get(&floor_texture_name).unwrap_or(&(0,0,0)).0;
	            let ceil_texture_id = texture_ids.get(&ceil_texture_name).unwrap_or(&(0,0,0)).0;

	            let clamped_light = sector.lightlevel.clamp(0, 255) as f32;
	            let modern_light = clamped_light / 255.0;
	            let colormap_idx = 31 - ((clamped_light / 8.0).floor() as u32).clamp(0, 31);

	            let floor_start_idx = vertices.len() as u32;
	            for pt in &flat_points {
	                vertices.push(Vertex { 
	                    pos: [-(pt[0]), sector.floorheight.into(), pt[1]],
	                    texture_pos: [pt[0] / 64.0, pt[1] / 64.0],
						light_level: modern_light,
	                    texture_id: floor_texture_id,
	                    colormap_idx,
	                });
	            }
	            for chunk in sector_indices.chunks_exact(3) {
	                indices.push(floor_start_idx + chunk[0]);
	                indices.push(floor_start_idx + chunk[1]);
	                indices.push(floor_start_idx + chunk[2]);
	            }

	            let ceil_start_idx = vertices.len() as u32;
	            for pt in &flat_points {
	                vertices.push(Vertex { 
	                    pos: [-(pt[0]), sector.ceilingheight.into(), pt[1]],
	                    texture_pos: [pt[0] / 64.0, pt[1] / 64.0],
						light_level: modern_light,
	                    texture_id: ceil_texture_id,
	                    colormap_idx,
	                });
	            }
	            for chunk in sector_indices.chunks_exact(3) {
	                indices.push(ceil_start_idx + chunk[0]);
	                indices.push(ceil_start_idx + chunk[2]);
	                indices.push(ceil_start_idx + chunk[1]);
	            }
	        }
	    }
	    (vertices, indices)
	}

	pub fn get_sector_by_pos(&self, x: f32, y: f32) -> usize {
        if self.nodes.is_empty() {
            return 0;
        }
        
        let root_node_idx = self.nodes.len() - 1;
        let subsector_idx = self.find_subsector_by_pos(root_node_idx, x, y);
        let subsector = &self.subsectors[subsector_idx];
        
        let first_seg_idx = subsector.firstseg as usize;
        let seg = &self.segs[first_seg_idx];
        
        if seg.linedef != -1 {
            let linedef = &self.linedefs[seg.linedef as usize];
            let side = linedef.sidenum[seg.side as usize];
            if side != -1 {
                return self.sidedefs[side as usize].sector as usize;
            }
        }

        0
    }

    fn find_subsector_by_pos(&self, node_idx: usize, x: f32, y: f32) -> usize {
        if (node_idx & NF_SUBSECTOR) != 0 {
            return node_idx & !NF_SUBSECTOR;
        }

        let node = &self.nodes[node_idx];

        let dx = x - node.x as f32;
        let dy = y - node.y as f32;

        let is_left = (dx * node.dy as f32) - (dy * node.dx as f32) <= 0.0;

        if is_left {
            self.find_subsector_by_pos(node.children[1] as usize, x, y)
        } else {
            self.find_subsector_by_pos(node.children[0] as usize, x, y)
        }
    }

	pub fn get_objects_vertices(&self) -> (Vec<Vertex>, Vec<u32>) {
	    let corners = [
		    ([0.0, 0.0, 0.0], [0.0, 1.0]),
		    ([1.0, 0.0, 0.0], [1.0, 1.0]),
		    ([1.0, 1.0, 0.0], [1.0, 0.0]),
		    ([0.0, 1.0, 0.0], [0.0, 0.0]),
		];

	    let vertices: Vec<Vertex> = corners
	        .iter()
	        .map(|&(pos, uv)| Vertex {
	            pos,
	            texture_pos: uv,

				// stub values; they are used from ObjectInstance instead
	            light_level: 1.0,
	            texture_id: 0,
	            colormap_idx: 0,
	        })
	        .collect();

	    let indices = vec![0, 1, 2, 0, 2, 3];

	    (vertices, indices)
	}
}

fn point_in_polygon(point: [f32; 2], poly: &[[f32; 2]]) -> bool {
    let mut inside = false;
    let mut j = poly.len() - 1;
    for i in 0..poly.len() {
        if (poly[i][1] > point[1]) != (poly[j][1] > point[1]) &&
           (point[0] < (poly[j][0] - poly[i][0]) * (point[1] - poly[i][1]) / (poly[j][1] - poly[i][1]) + poly[i][0]) {
            inside = !inside;
        }
        j = i;
    }
    inside
}

fn clean_polygon(poly: &[[f32; 2]]) -> Vec<[f32; 2]> {
    if poly.len() < 3 { return poly.to_vec(); }
    let mut cleaned = Vec::new();
    for i in 0..poly.len() {
        let prev = poly[(i + poly.len() - 1) % poly.len()];
        let curr = poly[i];
        let next = poly[(i + 1) % poly.len()];

        let area = (curr[0] - prev[0]) * (next[1] - prev[1]) - (next[0] - prev[0]) * (curr[1] - prev[1]);
        if area.abs() > 0.1 { 
            cleaned.push(curr);
        }
    }
    cleaned
}
