use crate::*;
use renderer::{Vertex};
use earcut::Earcut;
use rustc_hash::{FxBuildHasher, FxHashMap};
use std::ptr::read_unaligned;
use std::mem::size_of;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MapVertex
{
  x: i16,
  y: i16,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
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
#[derive(Debug, Clone, Copy)]
pub struct MapLinedef
{
	v1: i16,
	v2: i16,
	flags: i16,
	special: i16,
	tag: i16,
	sidenum: [u16; 2]	
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
#[derive(Debug, Clone, Copy)]
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
#[derive(Debug, Clone, Copy)]
pub struct MapSubsector
{
	numsegs: i16,
	firstseg: u16	
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
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
#[derive(Debug, Clone, Copy)]
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
#[derive(Debug, Clone, Copy)]
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
	pub map_num: u32,
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
    pub fn from_wad(wad_manager: &WadManager, is_doom1: bool, map_num: u32) -> Result<Self, String> {
		let mut map = DoomMap::default();

		map.map_num = map_num;

		let map_name = construct_map_name(is_doom1, map_num);

    	let vertexes_bytes = wad_manager.get_data(&format!("VERTEXES_{}", map_name))?;
		map.vertices = vertexes_bytes
    		.chunks_exact(size_of::<MapVertex>())
    		.map(|chunk| {
    		    unsafe { read_unaligned(chunk.as_ptr() as *const MapVertex) }
    		})
    		.collect();

		let linedefs_bytes = wad_manager.get_data(&format!("LINEDEFS_{}", map_name))?;
		map.linedefs = linedefs_bytes
    		.chunks_exact(size_of::<MapLinedef>())
    		.map(|chunk| {
    		    unsafe { read_unaligned(chunk.as_ptr() as *const MapLinedef) }
    		})
    		.collect();

		let sidedefs_bytes = wad_manager.get_data(&format!("SIDEDEFS_{}", map_name))?;
		map.sidedefs = sidedefs_bytes
    		.chunks_exact(size_of::<MapSidedef>())
    		.map(|chunk| {
    		    unsafe { read_unaligned(chunk.as_ptr() as *const MapSidedef) }
    		})
    		.collect();

		let sectors_bytes = wad_manager.get_data(&format!("SECTORS_{}", map_name))?;
		map.sectors = sectors_bytes
    		.chunks_exact(size_of::<MapSector>())
    		.map(|chunk| {
    		    unsafe { read_unaligned(chunk.as_ptr() as *const MapSector) }
    		})
    		.collect();

		let things_bytes = wad_manager.get_data(&format!("THINGS_{}", map_name))?;
		map.things = things_bytes
    		.chunks_exact(size_of::<MapThing>())
    		.map(|chunk| {
    		    unsafe { read_unaligned(chunk.as_ptr() as *const MapThing) }
    		})
    		.collect();

		let ssectors_bytes = wad_manager.get_data(&format!("SSECTORS_{}", map_name))?;
		map.subsectors = ssectors_bytes
    		.chunks_exact(size_of::<MapSubsector>())
    		.map(|chunk| {
    		    unsafe { read_unaligned(chunk.as_ptr() as *const MapSubsector) }
    		})
    		.collect();

		let segs_bytes = wad_manager.get_data(&format!("SEGS_{}", map_name))?;
		map.segs = segs_bytes
    		.chunks_exact(size_of::<MapSegment>())
    		.map(|chunk| {
    		    unsafe { read_unaligned(chunk.as_ptr() as *const MapSegment) }
    		})
    		.collect();

		let nodes_bytes = wad_manager.get_data(&format!("NODES_{}", map_name))?;
		map.nodes = nodes_bytes
    		.chunks_exact(size_of::<MapNode>())
    		.map(|chunk| {
    		    unsafe { read_unaligned(chunk.as_ptr() as *const MapNode) }
    		})
    		.collect();

        Ok(map)
    }

	pub fn get_walls_vertices(&self, texture_ids: &FxHashMap<u64, (u32, u32, u32, bool)>) -> (Vec<Vertex>, Vec<u32>) {
	    let mut vertices = Vec::new();
	    let mut indices = Vec::new();

	    for seg in self.segs.iter() {
	        if seg.linedef == -1 { continue; }

	        let v1 = self.vertices[seg.v1 as usize];
	        let v2 = self.vertices[seg.v2 as usize];
	        let linedef = self.linedefs[seg.linedef as usize];

	        let front_side_idx = if seg.side == 0 { linedef.sidenum[0] as u16 } else { linedef.sidenum[1] as u16 };
	        let back_side_idx = if seg.side == 0 { linedef.sidenum[1] as u16 } else { linedef.sidenum[0] as u16 };

	        if front_side_idx == u16::MAX { continue; }
	        let front_sidedef = self.sidedefs[front_side_idx as usize];
	        let front_sector_id = front_sidedef.sector;
	        let front_sector = self.sectors[front_sector_id as usize];

	        let back_sector = if back_side_idx != u16::MAX {
	            let back_sidedef = self.sidedefs[back_side_idx as usize];
	            Some((back_sidedef, self.sectors[back_sidedef.sector as usize]))
	        } else {
	            None
	        };

	        let dx = (v2.x - v1.x) as f32;
	        let dy = (v2.y - v1.y) as f32;
	        let wall_length = (dx * dx + dy * dy).sqrt();
	        let tex_offset = front_sidedef.textureoffset as f32;

	        let mut add_wall_quad = |y_low: f32, y_high: f32, tex_name: &[u8], v_top_align: bool, fake_flat_name: &[u8], other_sector_ceilingpic: &[u8]| {
            	let wall_height = y_high - y_low;
            	if wall_height <= 0.0 { return; }

            	let is_fake_wall = tex_name.is_empty() || tex_name[0] == 0x2d;
				
            	let final_tex_name = if is_fake_wall { fake_flat_name } else { tex_name };
				
            	let (tex_id, tex_width, tex_height, _) = *texture_ids
    			    .get(&pack_name_to_u64(final_tex_name))
    			    .unwrap_or(&(0, 64, 64, false));

    			let (final_tex_id, floor_tex_id) = if final_tex_name == b"F_SKY1\0\0" || 
					(other_sector_ceilingpic == b"F_SKY1\0\0" && fake_flat_name == b"F_SKY1\0\0") {
					((u16::MAX - 1) as u32, 0)
				} else if is_fake_wall {
    			    (u16::MAX as u32, tex_id)
				} else {
    			    (tex_id, 0)
    			};

            	let (u_start, u_end, v_start, v_end);

            	if is_fake_wall {
            	    let f_width = tex_width as f32;
            	    let f_height = tex_height as f32;

            	    u_start = -(v1.x as f32) / f_width;
            	    u_end = -(v2.x as f32) / f_width;

            	    v_start = (v1.y as f32) / f_height;
            	    v_end = (v2.y as f32) / f_height;
            	} else {
            	    u_start = (seg.offset as f32 + tex_offset) / tex_width as f32;
            	    u_end = u_start + (wall_length / tex_width as f32);

            	    if v_top_align {
            	        v_start = 0.0;
            	        v_end = wall_height / tex_height as f32;
            	    } else {
            	        v_start = -(wall_height / tex_height as f32);
            	        v_end = 0.0;
            	    }
            	}

	            let start_idx = vertices.len() as u32;

	            let clamped_light = front_sector.lightlevel.clamp(0, 255) as f32;
	            let modern_light = clamped_light / 255.0;
	            let colormap_idx = 31 - ((clamped_light / 8.0).floor() as u32).clamp(0, 31);

	            vertices.push(Vertex { 
	                pos: [-(v1.x as f32), y_low, v1.y as f32],
	                texture_pos: [u_start, v_end],
					light_level: modern_light,
	                texture_id: final_tex_id,
	                colormap_idx,
					floor_tex_id
	            });

	            vertices.push(Vertex { 
	                pos: [-(v2.x as f32), y_low, v2.y as f32],
	                texture_pos: [u_end, v_end],
					light_level: modern_light,
	                texture_id: final_tex_id,
	                colormap_idx,
					floor_tex_id
	            });

	            vertices.push(Vertex { 
	                pos: [-(v1.x as f32), y_high, v1.y as f32],
	                texture_pos: [u_start, v_start],
					light_level: modern_light,
	                texture_id: final_tex_id,
	                colormap_idx,
					floor_tex_id
	            });

	            vertices.push(Vertex { 
	                pos: [-(v2.x as f32), y_high, v2.y as f32],
	                texture_pos: [u_end, v_start],
					light_level: modern_light,
	                texture_id: final_tex_id,
	                colormap_idx,
					floor_tex_id
	            });
			
	            indices.push(start_idx + 0);
	            indices.push(start_idx + 1);
	            indices.push(start_idx + 2);

	            indices.push(start_idx + 2);
	            indices.push(start_idx + 1);
	            indices.push(start_idx + 3);
	        };

	        match back_sector {
        	    None => {
        	        add_wall_quad(
						front_sector.floorheight as f32, 
						front_sector.ceilingheight as f32, 
						&front_sidedef.midtexture, 
						true, 
						&front_sector.floorpic,
						&[]
					);
        	    },
        	    Some((_, b_sector)) => {
        	        if front_sector.ceilingheight > b_sector.ceilingheight {
        	            add_wall_quad(
							b_sector.ceilingheight as f32, 
							front_sector.ceilingheight as f32, 
							&front_sidedef.toptexture, 
							true, 
							&front_sector.ceilingpic,
							&b_sector.ceilingpic
						);
        	        }

        	        if front_sector.floorheight < b_sector.floorheight {
        	            add_wall_quad(
							front_sector.floorheight as f32, 
							b_sector.floorheight as f32, 
							&front_sidedef.bottomtexture, 
							false, 
							&b_sector.floorpic,
							&front_sector.ceilingpic
						);
        	        }

        	        if front_sidedef.midtexture[0] != 0x2d {
        	            let mid_low = f32::max(front_sector.floorheight as f32, b_sector.floorheight as f32);
        	            let mid_high = f32::min(front_sector.ceilingheight as f32, b_sector.ceilingheight as f32);
        	            add_wall_quad(mid_low, mid_high, &front_sidedef.midtexture, true, &front_sector.floorpic, &b_sector.ceilingpic);
        	        }
        	    }
        	}
	    }

	    (vertices, indices)
	}

	pub fn get_flats_vertices(&self, texture_ids: &FxHashMap<u64, (u32, u32, u32, bool)>) -> (Vec<Vertex>, Vec<u32>) {
	    let mut vertices: Vec<Vertex> = Vec::new();
	    let mut indices: Vec<u32> = Vec::new();

		let mut sector_to_linedefs: FxHashMap<i16, Vec<&MapLinedef>> = FxHashMap::with_capacity_and_hasher(self.sectors.len(), FxBuildHasher::default());
    	for linedef in self.linedefs.iter() {
    	    if linedef.sidenum[0] != u16::MAX {
    	        if let Some(side) = self.sidedefs.get(linedef.sidenum[0] as usize) {
    	            sector_to_linedefs.entry(side.sector).or_default().push(linedef);
    	        }
    	    }
    	    if linedef.sidenum[1] != u16::MAX {
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

	            if linedef.sidenum[0] != u16::MAX {
	                if let Some(side) = self.sidedefs.get(linedef.sidenum[0] as usize) {
	                    if side.sector == current_sector_id { edges.push((p1, p2)); }
	                }
	            }
			
	            if linedef.sidenum[1] != u16::MAX {
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
				
	            let floor_texture_name = pack_name_to_u64(&sector.floorpic);
	            let ceil_texture_name = pack_name_to_u64(&sector.ceilingpic);

	            let floor_texture_id = if &sector.floorpic == b"F_SKY1\0\0" {
					(u16::MAX - 2) as u32
				} else {
					texture_ids.get(&floor_texture_name).unwrap_or(&(0,0,0,false)).0
				};

	            let ceil_texture_id = if &sector.ceilingpic == b"F_SKY1\0\0" {
					(u16::MAX - 2) as u32
				} else { 
					texture_ids.get(&ceil_texture_name).unwrap_or(&(0,0,0,false)).0 
				};

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
						floor_tex_id: 0,
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
						floor_tex_id: 0,
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
            if side != u16::MAX {
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
				floor_tex_id: 0,
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
        if area.abs() > 0.001 { 
            cleaned.push(curr);
        }
    }
    cleaned
}

fn construct_map_name(is_doom1: bool, num: u32) -> String {
	if is_doom1 {
        let episode = (num / 9) + 1;
        let map_num = (num % 9) + 1;
        format!("E{}M{}", episode, map_num)
    } else {
        format!("MAP{:02}", num)
    }
}

pub fn pack_name_to_u64(name_bytes: &[u8]) -> u64 {
    let mut buf = [0u8; 8];

    for i in 0..8 {
        if i >= name_bytes.len() || name_bytes[i] == 0 {
            break;
        }
        buf[i] = name_bytes[i].to_ascii_uppercase();
    }

    u64::from_le_bytes(buf)
}
