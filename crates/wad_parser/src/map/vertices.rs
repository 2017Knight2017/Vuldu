use crate::{Level, LineFlags, LineId, SectorId, SubsectorId, TextureId, to_u64};
use earcut::Earcut;
use rustc_hash::{FxBuildHasher, FxHashMap};

pub const NF_SUBSECTOR: usize = 0x8000; 

// GpuVertex MUST equal to renderer::Vertex
#[repr(C)]
pub struct GpuVertex {
    pub pos: [f32; 3],
    pub texture_pos: [f32; 2],
    pub light_level: f32,
    pub texture_id: u32,
    pub colormap_idx: u32,
    pub floor_tex_id: u32,
	pub scroll_dir: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct AABB {
    pub min_x: f32,
    pub max_x: f32,
    pub min_z: f32,
    pub max_z: f32,
}

impl AABB {
    fn from_polygon(poly: &[[f32; 2]]) -> Self {
        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let mut min_z = f32::MAX;
        let mut max_z = f32::MIN;
        for pt in poly {
            if pt[0] < min_x { min_x = pt[0]; }
            if pt[0] > max_x { max_x = pt[0]; }
            if pt[1] < min_z { min_z = pt[1]; }
            if pt[1] > max_z { max_z = pt[1]; }
        }
        AABB { min_x, max_x, min_z, max_z }
    }

    pub fn intersects_aabb(&self, other: &Self) -> bool {
        self.min_x < other.max_x && self.max_x > other.min_x &&
        self.min_z < other.max_z && self.max_z > other.min_z
    }

	pub fn intersects_line(&self, line_bbox: &AABB, v1: (f32, f32), v2: (f32, f32)) -> bool {
	    if line_bbox.intersects_aabb(self) {
			return true;
		}

	    let a = v2.1 - v1.1;
	    let b = v1.0 - v2.0;
	    let c = v2.0 * v1.1 - v1.0 * v2.1;

	    let f1 = a * self.min_x + b * self.min_z + c;
	    let f2 = a * self.max_x + b * self.min_z + c;
	    let f3 = a * self.min_x + b * self.max_z + c;
	    let f4 = a * self.max_x + b * self.max_z + c;

	    if (f1 > 0.0 && f2 > 0.0 && f3 > 0.0 && f4 > 0.0) 
	    || (f1 < 0.0 && f2 < 0.0 && f3 < 0.0 && f4 < 0.0) {
	        return false;
	    }

	    true
	}
}

#[derive(Clone, Copy, Debug)]
struct Edge {
    v1: (i32, i32),
    v2: (i32, i32),
    used: bool,
}

impl Level {
	pub fn get_walls_vertices(&mut self, texture_ids: &FxHashMap<u64, (TextureId, u32, u32, bool)>) -> (Vec<GpuVertex>, Vec<u32>) {
	    let mut gpu_vertices = Vec::new();
	    let mut gpu_indices = Vec::new();

		let segs = &self.geom.segs;
		let vertices = &self.geom.vertices;
		let sides_geom = &self.geom.sides;
		let sides_state = &mut self.state.sides;
		let lines = &self.geom.lines;
		let sectors = &self.state.sectors;
		
	    for seg in segs.iter() {
	        if seg.linedef == u16::MAX { continue; }

	        let v1 = vertices[seg.v1 as usize];
	        let v2 = vertices[seg.v2 as usize];
	        let line = &lines[seg.linedef as usize];

			let scroll_dir = match line.special {
				48 => 1.0,
				85 => -1.0,
				_ => 0.0
			};

	        let front_side_idx_opt = if seg.side == 0 { line.sides.0 } else { line.sides.1 };
	        let back_side_idx_opt = if seg.side == 0 { line.sides.1 } else { line.sides.0 };

	        let front_side_idx = match front_side_idx_opt {
				None => continue,
				Some(idx) => idx,
			};
			
			let front_side = &mut sides_state[front_side_idx.0];
	        let front_sector = &sectors[sides_geom[front_side_idx.0].sector.0];
	        let back_sector = match back_side_idx_opt {
				Some(idx) => Some((&sides_geom[idx.0], &sectors[sides_geom[idx.0].sector.0])),
				None => None
			};

	        let dx = (v2.0 - v1.0) as f32;
	        let dy = (v2.1 - v1.1) as f32;
	        let wall_length = (dx * dx + dy * dy).sqrt();
	        let tex_offset = front_side.col_offset as f32;
			let row_offset = front_side.row_offset as f32;

	        let mut add_wall_quad = |
				y_low: f32, 
				y_high: f32, 
				tex_name: &[u8], 
				v_offset: f32, 
				fake_flat_name: &[u8], 
				other_sector_ceilingpic: &[u8],
				texture_width: u32,
			| -> Option<TextureId> {
            	let wall_height = y_high - y_low;
            	if wall_height <= 0.0 { return None; }

            	let is_fake_wall = tex_name.is_empty() || tex_name[0] == 0x2d;
				
            	let final_tex_name = if is_fake_wall { fake_flat_name } else { tex_name };
				
            	let (tex_id, tex_width, tex_height, _) = *texture_ids
    			    .get(&to_u64(final_tex_name))
    			    .unwrap_or(&(TextureId(0), 64, 64, false));

    			let (final_tex_id, floor_tex_id) = if final_tex_name.starts_with(b"F_SKY1") || 
					(other_sector_ceilingpic.starts_with(b"F_SKY1") && fake_flat_name.starts_with(b"F_SKY1")) 
				{
					(TextureId((u16::MAX - 1) as u32), TextureId(0))
				} else if is_fake_wall {
    			    (TextureId(u16::MAX as u32), tex_id)
				} else {
    			    (tex_id, TextureId(0))
    			};

            	let (u_start, u_end, v_start, v_end);

            	if is_fake_wall {
            	    let f_width = tex_width as f32;
            	    let f_height = tex_height as f32;

            	    u_start = -v1.0 / f_width;
            	    u_end = -v2.0 / f_width;

            	    v_start = v1.0 / f_height;
            	    v_end = v2.1 / f_height;
            	} else {
            	    u_start = (seg.offset as f32 + tex_offset) / tex_width as f32;
            	    u_end = u_start + (wall_length / tex_width as f32);

            	    let f_tex_height = tex_height as f32;
                	v_start = (v_offset + row_offset) / f_tex_height;
                	v_end = v_start + (wall_height / f_tex_height);
            	}

	            let start_idx = gpu_vertices.len() as u32;

	            let clamped_light = front_sector.light.clamp(0, 255) as f32;
	            let modern_light = clamped_light / 255.0;
	            let colormap_idx = 31 - ((clamped_light / 8.0).floor() as u32).clamp(0, 31);

	            gpu_vertices.push(GpuVertex { 
	                pos: [v1.0, y_low, v1.1],
	                texture_pos: [u_start, v_end],
					light_level: modern_light,
	                texture_id: final_tex_id.0,
	                colormap_idx,
					floor_tex_id: floor_tex_id.0,
					scroll_dir: scroll_dir / texture_width as f32,
	            });

	            gpu_vertices.push(GpuVertex { 
	                pos: [v2.0, y_low, v2.1],
	                texture_pos: [u_end, v_end],
					light_level: modern_light,
	                texture_id: final_tex_id.0,
	                colormap_idx,
					floor_tex_id: floor_tex_id.0,
					scroll_dir: scroll_dir / texture_width as f32,
	            });

	            gpu_vertices.push(GpuVertex { 
	                pos: [v1.0, y_high, v1.1],
	                texture_pos: [u_start, v_start],
					light_level: modern_light,
	                texture_id: final_tex_id.0,
	                colormap_idx,
					floor_tex_id: floor_tex_id.0,
					scroll_dir: scroll_dir / texture_width as f32,
	            });

	            gpu_vertices.push(GpuVertex { 
	                pos: [v2.0, y_high, v2.1],
	                texture_pos: [u_end, v_start],
					light_level: modern_light,
	                texture_id: final_tex_id.0,
	                colormap_idx,
					floor_tex_id: floor_tex_id.0,
					scroll_dir: scroll_dir / texture_width as f32,
	            });
			
	            gpu_indices.push(start_idx + 0);
	            gpu_indices.push(start_idx + 1);
	            gpu_indices.push(start_idx + 2);

	            gpu_indices.push(start_idx + 2);
	            gpu_indices.push(start_idx + 1);
	            gpu_indices.push(start_idx + 3);

				Some(final_tex_id)
	        };

			let dont_peg_top = line.flags.contains(LineFlags::DONT_PEG_TOP);
        	let dont_peg_bottom = line.flags.contains(LineFlags::DONT_PEG_BOTTOM);

	        match back_sector {
        	    None => {
					let (_, tex_w, tex_h, _) = texture_ids
						.get(&to_u64(&front_side.midtexture))
						.unwrap_or(&(TextureId(0), 64, 64, false));
                
                	let v_offset = if dont_peg_bottom {
                	    let offset = front_sector.ceil_h - front_sector.floor_h;
                	    *tex_h as f32 - offset
                	} else {
                	    0.0
                	};

					
        	        front_side.mid_tex = add_wall_quad(
						front_sector.floor_h, 
						front_sector.ceil_h, 
						&front_side.midtexture, 
						v_offset, 
						&front_sector.floorpic,
						&[],
						*tex_w,
					);
        	    },
        	    Some((_, b_sector)) => {
        	        if front_sector.ceil_h > b_sector.ceil_h {
						let (_, tex_w, tex_h, _) = texture_ids
							.get(&to_u64(&front_side.midtexture))
							.unwrap_or(&(TextureId(0), 64, 64, false));

						let v_offset = if dont_peg_top {
							0.0
                        } else {
							let offset = b_sector.ceil_h - front_sector.ceil_h;
                            offset - *tex_h as f32
                        };

						front_side.top_tex = add_wall_quad(
							b_sector.ceil_h, 
							front_sector.ceil_h, 
							&front_side.toptexture, 
							v_offset, 
							&front_sector.ceilingpic,
							&b_sector.ceilingpic,
							*tex_w
						);
        	        }

        	        if front_sector.floor_h < b_sector.floor_h {
						let (_, tex_w, tex_h, _) = texture_ids
							.get(&to_u64(&front_side.midtexture))
							.unwrap_or(&(TextureId(0), 64, 64, false));

						let v_offset = if dont_peg_bottom {
    						let offset = front_sector.ceil_h - b_sector.floor_h;
							offset - *tex_h as f32
                    	} else {
							0.0 
                    	};

        	            front_side.bottom_tex = add_wall_quad(
							front_sector.floor_h, 
							b_sector.floor_h, 
							&front_side.bottomtexture, 
							v_offset,
							&b_sector.floorpic,
							&front_sector.ceilingpic,
							*tex_w
						);
        	        }

        	        if front_side.midtexture[0] != 0x2d {
						let tex_w = match texture_ids.get(&to_u64(&front_side.bottomtexture)) {
							Some(tex) => tex.1,
							None => 64
						};

        	            let mid_low = front_sector.floor_h.max(b_sector.floor_h);
        	            let mid_high = front_sector.ceil_h.min(b_sector.ceil_h);
        	            front_side.mid_tex = add_wall_quad(mid_low, mid_high, &front_side.midtexture, 
							0.0, &front_sector.floorpic, &b_sector.ceilingpic, tex_w);
        	        }
        	    }
        	}
	    }

	    (gpu_vertices, gpu_indices)
	}

	pub fn get_flats_vertices(&mut self, texture_ids: &FxHashMap<u64, (TextureId, u32, u32, bool)>) -> (Vec<GpuVertex>, Vec<u32>) {
	    let mut gpu_vertices: Vec<GpuVertex> = Vec::new();
	    let mut gpu_indices: Vec<u32> = Vec::new();

		self.geom.sector_lines = vec![Vec::new(); self.state.sectors.len()];

		let lines = &self.geom.lines;
		let sides = &self.geom.sides;
		let sectors = &mut self.state.sectors;
		let vertices = &self.geom.vertices;

    	for (i, line) in lines.iter().enumerate() {
    	    if let Some(front_side) = line.sides.0 {
				let sector_idx = sides[front_side.0].sector.0;

    	    	self.geom.sector_lines[sector_idx].push(LineId(i));
    	    }

    	    if let Some(back_side) = line.sides.1 {
				let sector_idx = sides[back_side.0].sector.0;

    	        self.geom.sector_lines[sector_idx].push(LineId(i));
    	    }
    	}

	    for (sector_id, sector) in sectors.iter_mut().enumerate() {
			let current_sector_lines = &self.geom.sector_lines[sector_id];
	        let mut edges: Vec<Edge> = Vec::with_capacity(current_sector_lines.len() * 2);

	        for line_id in current_sector_lines {
				let line = &lines[line_id.0];

				let sector_front = match line.sides.0 {
					Some(side_id) => Some(sides[side_id.0].sector.0),
					None => None
				};
			
    			let sector_back = match line.sides.1 {
					Some(side_id) => Some(sides[side_id.0].sector.0),
					None => None
				};
						
    			if sector_front == Some(sector_id) && sector_back == Some(sector_id) {
    			    continue;
    			}

	            let v1 = (vertices[line.v1.0].0 as i32, vertices[line.v1.0].1 as i32);
	            let v2 = (vertices[line.v2.0].0 as i32, vertices[line.v2.0].1 as i32);

	            if sector_front == Some(sector_id) {
    			    edges.push(Edge { v1, v2, used: false });
    			}
			
    			if sector_back == Some(sector_id) {
    			    edges.push(Edge { v1: v2, v2: v1, used: false });
    			}
	        }

	        if edges.is_empty() { continue; }

			let mut adjacency: FxHashMap<(i32, i32), Vec<usize>> =
            	FxHashMap::with_capacity_and_hasher(edges.len(), FxBuildHasher::default());

			for (idx, edge) in edges.iter().enumerate() {
        	    adjacency.entry(edge.v1).or_default().push(idx);
        	}

	        let mut polygon_loops: Vec<Vec<[f32; 2]>> = Vec::new();

	        for i in 0..edges.len() {
				if edges[i].used {
					continue
				}

				edges[i].used = true;
            	let start_edge = edges[i];

	            let mut current_loop = Vec::new();
	            current_loop.push([start_edge.v1.0 as f32, start_edge.v1.1 as f32]);

				let start_point = start_edge.v1;
				let mut prev_point = start_edge.v1;
	            let mut current_tip = start_edge.v2;

	            let mut stuck = false;

				let max_steps = edges.len() + 1;
				let mut steps = 0;

    			while current_tip != start_point {
					steps += 1;
    				if steps > max_steps {
    				    stuck = true;
    				    break;
    				}

    			    current_loop.push([current_tip.0 as f32, current_tip.1 as f32]);
				
    			    if let Some(next_idx) = find_next_edge_by_angle(prev_point, current_tip, &edges, &adjacency) {
    			        edges[next_idx].used = true;
    			        prev_point = current_tip;
    			        current_tip = edges[next_idx].v2;
    			    } else {
    			        stuck = true;
						break;
    			    }
    			}

    			if stuck || current_loop.len() < 3 {
    			    println!("Loop got stuck at tip: {:?}", current_tip);
					continue;
    			}
			
	            let cleaned = clean_polygon(&current_loop);
	            if cleaned.len() >= 3 {
	                polygon_loops.push(cleaned);
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

	        let mut outer_sectors: Vec<(Vec<[f32; 2]>, AABB)> = Vec::new();
	        let mut hole_loops: Vec<Vec<[f32; 2]>> = Vec::new();

	        for poly_loop in polygon_loops.iter().cloned() {
				let poly_aabb = AABB::from_polygon(&poly_loop);
	            let mut is_hole = false;

	            for (outer, outer_aabb) in &outer_sectors {
            	    if poly_aabb.intersects_aabb(outer_aabb) {
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

	            if outer_area < 0.0 { 
					outer_loop.reverse(); 
				}

	            for pt in &outer_loop { 
					flat_points.push(*pt); 
				}

	            for hole in &hole_loops {
					let hole_aabb = AABB::from_polygon(hole);
                	if !outer_aabb.intersects_aabb(&hole_aabb) { continue; }

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

	                if h_area > 0.0 { 
						hole_copy.reverse(); 
					}

	                for pt in hole_copy { 
						flat_points.push(pt); 
					}
	            }
			
	            let mut sector_indices: Vec<u32> = Vec::new();
	            let mut earcut = Earcut::new();
	            earcut.earcut(flat_points.iter().copied(), &hole_indices, &mut sector_indices);
			
	            if sector_indices.is_empty() { 
	                println!("[WARN] Sector {}: Earcut failed to triangulate polygon!", sector_id);
	                continue; 
	            }
				
	            let floor_texture_name = to_u64(&sector.floorpic);
	            let ceil_texture_name = to_u64(&sector.ceilingpic);

	            sector.floor_tex = if sector.floorpic.starts_with(b"F_SKY1") {
					TextureId((u16::MAX - 2) as u32)
				} else {
					match texture_ids.get(&floor_texture_name){
						Some(tex) => tex.0,
						None => TextureId(0)
					}
				};

	            sector.ceil_tex = if sector.ceilingpic.starts_with(b"F_SKY1") {
					TextureId((u16::MAX - 2) as u32)
				} else { 
					match texture_ids.get(&ceil_texture_name){
						Some(tex) => tex.0,
						None => TextureId(0)
					}
				};

	            let clamped_light = sector.light.clamp(0, 255) as f32;
	            let modern_light = clamped_light / 255.0;
	            let colormap_idx = 31 - ((clamped_light / 8.0).floor() as u32).clamp(0, 31);

	            let floor_start_idx = gpu_vertices.len() as u32;
	            for pt in &flat_points {
	                gpu_vertices.push(GpuVertex { 
	                    pos: [pt[0], sector.floor_h, pt[1]],
	                    texture_pos: [pt[0] / 64.0, pt[1] / 64.0],
						light_level: modern_light,
	                    texture_id: sector.floor_tex.0,
	                    colormap_idx,
						floor_tex_id: 0,
						scroll_dir: 0.0,
	                });
	            }
	            for chunk in sector_indices.chunks_exact(3) {
	                gpu_indices.push(floor_start_idx + chunk[0]);
	                gpu_indices.push(floor_start_idx + chunk[1]);
	                gpu_indices.push(floor_start_idx + chunk[2]);
	            }

	            let ceil_start_idx = gpu_vertices.len() as u32;
	            for pt in &flat_points {
	                gpu_vertices.push(GpuVertex { 
	                    pos: [pt[0], sector.ceil_h, pt[1]],
	                    texture_pos: [pt[0] / 64.0, pt[1] / 64.0],
						light_level: modern_light,
	                    texture_id: sector.ceil_tex.0,
	                    colormap_idx,
						floor_tex_id: 0,
						scroll_dir: 0.0,
	                });
	            }
	            for chunk in sector_indices.chunks_exact(3) {
	                gpu_indices.push(ceil_start_idx + chunk[0]);
	                gpu_indices.push(ceil_start_idx + chunk[2]);
	                gpu_indices.push(ceil_start_idx + chunk[1]);
	            }
	        }
	    }

	    (gpu_vertices, gpu_indices)
	}

	pub fn get_sector_by_pos(&self, x: f32, z: f32) -> SectorId {
		if self.geom.nodes.is_empty() {
            return SectorId(0);
        }

		let root_node_idx = self.geom.nodes.len() - 1;
        let subsector_id = self.find_subsector_by_pos(root_node_idx, x, z);

        self.geom.subsector_sector[subsector_id.0]
    }

    fn find_subsector_by_pos(&self, node_idx: usize, x: f32, z: f32) -> SubsectorId {
        if (node_idx & NF_SUBSECTOR) != 0 {
            return SubsectorId(node_idx & !NF_SUBSECTOR);
        }

        let node = &self.geom.nodes[node_idx];

        let dx = x - node.x as f32;
        let dz = z - node.y as f32;

        let is_left = (dx * node.dy as f32) - (dz * node.dx as f32) <= 0.0;

        if is_left {
            self.find_subsector_by_pos(node.children[1] as usize, x, z)
        } else {
            self.find_subsector_by_pos(node.children[0] as usize, x, z)
        }
    }

	pub fn get_objects_vertices(&self) -> (Vec<GpuVertex>, Vec<u32>) {
	    let corners = [
		    ([0.0, 0.0, 0.0], [1.0, 1.0]),
		    ([1.0, 0.0, 0.0], [0.0, 1.0]),
		    ([1.0, 1.0, 0.0], [0.0, 0.0]),
		    ([0.0, 1.0, 0.0], [1.0, 0.0]),
		];

	    let vertices: Vec<GpuVertex> = corners
	        .iter()
	        .map(|&(pos, texture_pos)| GpuVertex {
	            pos,
	            texture_pos,

				// stub values; they are used from ObjectInstance instead
	            light_level: 1.0,
	            texture_id: 0,
	            colormap_idx: 0,
				floor_tex_id: 0,
				scroll_dir: 0.0,
	        })
	        .collect();

	    let indices = vec![0, 3, 2, 0, 2, 1];

	    (vertices, indices)
	}

	pub fn get_ui_vertices(&self) -> (Vec<GpuVertex>, Vec<u32>) {
		let corners = [
		    ([0.0, 0.0, 0.0], [0.0, 0.0]),
		    ([1.0, 0.0, 0.0], [1.0, 0.0]),
		    ([1.0, 1.0, 0.0], [1.0, 1.0]),
		    ([0.0, 1.0, 0.0], [0.0, 1.0]),
		];

	    let vertices: Vec<GpuVertex> = corners
	        .iter()
	        .map(|&(pos, texture_pos)| GpuVertex {
	            pos,
	            texture_pos,

				// stub values; they are used from UiInstance instead
	            light_level: 1.0,
	            texture_id: 0,
	            colormap_idx: 0,
				floor_tex_id: 0,
				scroll_dir: 0.0,
	        })
	        .collect();

	    let indices = vec![0, 3, 2, 0, 2, 1];

	    (vertices, indices)
	}
}

fn point_in_polygon(point: [f32; 2], poly: &[[f32; 2]]) -> bool {
    let mut inside = false;
    let mut j = poly.len() - 1;

    for i in 0..poly.len() {
		let pi = poly[i];
		let pj = poly[j];

		let crosses_y_span = (pi[1] > point[1]) != (pj[1] > point[1]);

		if crosses_y_span {
			let x_intersection = (pj[0] - pi[0]) * (point[1] - pi[1]) / (pj[1] - pi[1]) + pi[0];

        	if point[0] < x_intersection {
        	    inside = !inside;
        	}
		}
        j = i;
    }
    inside
}

fn clean_polygon(poly: &[[f32; 2]]) -> Vec<[f32; 2]> {
	let len = poly.len();
    if len < 3 { 
		return Vec::new(); 
	}

    let mut cleaned = Vec::with_capacity(len);

    for i in 0..poly.len() {
        let prev = poly[(i + poly.len() - 1) % poly.len()];
        let curr = poly[i];
        let next = poly[(i + 1) % poly.len()];

		let dx1 = curr[0] - prev[0];
        let dy1 = curr[1] - prev[1];
        let dx2 = next[0] - curr[0];
        let dy2 = next[1] - curr[1];

		let cross = dx1 * dy2 - dy1 * dx2;

        if cross.abs() > 0.001 { 
            cleaned.push(curr);
        }
    }
    cleaned
}

fn find_next_edge_by_angle(
    prev_point: (i32, i32),
    current_tip: (i32, i32),
    edges: &[Edge],
	adjacency: &FxHashMap<(i32, i32), Vec<usize>>,
) -> Option<usize> {
	let candidate_indices = adjacency.get(&current_tip)?;

    let in_dir = [
		(current_tip.0 - prev_point.0) as f32, 
		(current_tip.1 - prev_point.1) as f32,
	];
    let in_angle = pseudo_angle(in_dir[0], in_dir[1]);

    let mut best_idx = None;
    let mut min_turn_angle = f32::MAX;

    for &idx in candidate_indices {
		let edge = &edges[idx];
        if edge.used {
            continue;
        }
            
        let out_dir = [
			(edge.v2.0 - current_tip.0) as f32, 
			(edge.v2.1 - current_tip.1) as f32
		];
        let out_angle = pseudo_angle(out_dir[0], out_dir[1]);

        let mut turn_angle = out_angle - in_angle;
        if turn_angle < 0.0 {
            turn_angle += 4.0;
        }

        if turn_angle < min_turn_angle {
            min_turn_angle = turn_angle;
            best_idx = Some(idx);
        }
    }

    best_idx
}

fn pseudo_angle(dx: f32, dy: f32) -> f32 {
    let sum = dx.abs() + dy.abs();
    if sum == 0.0 {
        return 0.0;
    }

    let p = dx / sum;
    if dy >= 0.0 {
        1.0 - p  // [0.0, 2.0]
    } else {
        3.0 + p  // [2.0, 4.0]
    }
}
