use crate::*;
use bytemuck::{Pod, Zeroable};
use renderer::Vertex;

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct MapVertex
{
  x: i16,
  y: i16,
}

#[repr(C, packed)]
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

#[repr(C, packed)]
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

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct MapSector
{
	floorheight: i16,
	ceilingheight: i16,
  	floorpic: [u8; 8],
  	ceilingpic:[u8; 8],
	lightlevel: i16,
	special: i16,
	tag: i16,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct MapSubsector
{
	numsegs: i16,
	firstseg: i16	
}

#[repr(C, packed)]
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

pub const NF_SUBSECTOR: u16 = 0x8000;

#[repr(C, packed)]
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

#[repr(C, packed)] 
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct MapThing
{
	pub x: i16,
	pub y: i16,
	pub angle: i16,
	pub type_: i16,
	pub options: i16,
}

#[derive(Debug, Default)]
pub struct DoomMap {
    vertices: Vec<MapVertex>,
    linedefs: Vec<MapLinedef>,
    sidedefs: Vec<MapSidedef>,
    sectors: Vec<MapSector>,
    pub things: Vec<MapThing>,
	subsectors: Vec<MapSubsector>,
	segs: Vec<MapSegment>
}



impl DoomMap {
    pub fn from_wad(wad: &Wad, map_name: &str) -> Result<Self, String> {
        let mut map = DoomMap::default();

        if let Some(lump) = wad.directory.get(&format!("VERTEXES_{}", map_name)) {
            let bytes = wad.data.get(lump.offset..lump.offset + lump.size)
                .ok_or("Failed to get VERTEXES bytes")?;
            let typed_slice: &[MapVertex] = bytemuck::cast_slice(bytes);
            map.vertices = typed_slice.to_vec();
        }

        if let Some(lump) = wad.directory.get(&format!("LINEDEFS_{}", map_name)) {
            let bytes = wad.data.get(lump.offset..lump.offset + lump.size)
                .ok_or("Failed to get LINEDEFS bytes")?;
            let typed_slice: &[MapLinedef] = bytemuck::cast_slice(bytes);
            map.linedefs = typed_slice.to_vec();
        }

        if let Some(lump) = wad.directory.get(&format!("SIDEDEFS_{}", map_name)) {
            let bytes = wad.data.get(lump.offset..lump.offset + lump.size)
                .ok_or("Failed to get SIDEDEFS bytes")?;
            let typed_slice: &[MapSidedef] = bytemuck::cast_slice(bytes);
            map.sidedefs = typed_slice.to_vec();
        }

        if let Some(lump) = wad.directory.get(&format!("SECTORS_{}", map_name)) {
            let bytes = wad.data.get(lump.offset..lump.offset + lump.size)
                .ok_or("Failed to get SECTORS bytes")?;
            let typed_slice: &[MapSector] = bytemuck::cast_slice(bytes);
            map.sectors = typed_slice.to_vec();
        }

        if let Some(lump) = wad.directory.get(&format!("THINGS_{}", map_name)) {
            let bytes = wad.data.get(lump.offset..lump.offset + lump.size)
                .ok_or("Failed to get THINGS bytes")?;
            let typed_slice: &[MapThing] = bytemuck::cast_slice(bytes);
            map.things = typed_slice.to_vec();
        }

        if let Some(lump) = wad.directory.get(&format!("SSECTORS_{}", map_name)) {
            let bytes = wad.data.get(lump.offset..lump.offset + lump.size)
                .ok_or("Failed to get SSECTORS bytes")?;
            let typed_slice: &[MapSubsector] = bytemuck::cast_slice(bytes);
            map.subsectors = typed_slice.to_vec();
        }

        if let Some(lump) = wad.directory.get(&format!("SEGS_{}", map_name)) {
            let bytes = wad.data.get(lump.offset..lump.offset + lump.size)
                .ok_or("Failed to get SEGS bytes")?;
            let typed_slice: &[MapSegment] = bytemuck::cast_slice(bytes);
            map.segs = typed_slice.to_vec();
        }

        Ok(map)
    }

	pub fn get_walls_vertices(&self, texture_ids: &HashMap<String, (u32, u32, u32)>) -> (Vec<Vertex>, Vec<u16>) {
		let mut vertices = Vec::new();
		let mut indices = Vec::new();
		for seg in self.segs.iter() {
			if seg.linedef == -1 { continue; }
			let v1 = self.vertices[seg.v1 as usize];
			let v2 = self.vertices[seg.v2 as usize];

			let linedef = self.linedefs[seg.linedef as usize];
			let sidedef = self.sidedefs[linedef.sidenum[0] as usize];
			let sector_id = sidedef.sector;
			let sector = self.sectors[sector_id as usize];

			let dx = (v2.x - v1.x) as f32;
        	let dy = (v2.y - v1.y) as f32;
        	let wall_length = (dx * dx + dy * dy).sqrt();
        	let wall_height = (sector.ceilingheight - sector.floorheight) as f32;

			let tex_name = String::from_utf8_lossy(&sidedef.midtexture)
                .trim_matches('\0')
                .trim()
                .to_uppercase();

			if tex_name == "-" || tex_name.is_empty() || linedef.sidenum[1] != -1 {
    		    continue;
    		}

			let (tex_id, tex_width, tex_height) = *texture_ids.get(&tex_name).unwrap_or(&(0, 64, 64));
			
        	let tex_offset = sidedef.textureoffset as f32;
        	let u_start = (seg.offset as f32 + tex_offset) / tex_width as f32; 
			let u_end = u_start + (wall_length / tex_width as f32);
			let v_max = wall_height / tex_height as f32;

			let start_idx = vertices.len() as u16;

			let clamped_light = sector.lightlevel.clamp(0, 255) as f32;
			let light_f32 = clamped_light / 255.0;
			let normalized_light_level = [light_f32, light_f32, light_f32];

			let raw_map_idx = (clamped_light / 8.0).floor() as u32;
			let colormap_idx = 31 - raw_map_idx.clamp(0, 31);

			vertices.push(Vertex { 
				pos: [-(v1.x as f32), sector.floorheight.into(), v1.y.into()],
    			light_level: normalized_light_level,
    			texture_pos: [u_start, v_max],
    			texture_id: tex_id,
    			sector_id: sector_id as u32,
				colormap_idx: colormap_idx,
			});

			vertices.push(Vertex { 
				pos: [-(v2.x as f32), sector.floorheight.into(), v2.y.into()],
    			light_level: normalized_light_level,
    			texture_pos: [u_end, v_max],
    			texture_id: tex_id,
    			sector_id: sector_id as u32,
				colormap_idx: colormap_idx,
			});

			vertices.push(Vertex { 
				pos: [-(v1.x as f32), sector.ceilingheight.into(), v1.y.into()],
    			light_level: normalized_light_level,
    			texture_pos: [u_start, 0.0],
    			texture_id: tex_id,
    			sector_id: sector_id as u32,
				colormap_idx: colormap_idx,
			});

			vertices.push(Vertex { 
				pos: [-(v2.x as f32), sector.ceilingheight.into(), v2.y.into()],
    			light_level: normalized_light_level,
    			texture_pos: [u_end, 0.0],
    			texture_id: tex_id,
    			sector_id: sector_id as u32,
				colormap_idx: colormap_idx,
			});
    		
			indices.push(start_idx + 0);
			indices.push(start_idx + 1);
			indices.push(start_idx + 2);

			indices.push(start_idx + 2);
			indices.push(start_idx + 1);
			indices.push(start_idx + 3);
		}
		(vertices, indices)
	}

	pub fn get_flats_vertices(&self, texture_ids: &HashMap<String, (u32, u32, u32)>) -> (Vec<Vertex>, Vec<u16>) {
		let mut vertices: Vec<Vertex> = Vec::new();
    	let mut indices: Vec<u16> = Vec::new();

    	for ssector in self.subsectors.iter() {
    	    let mut current_sector_id = 0;

			let mut unique_points: Vec<[f32; 2]> = Vec::new();
        	let mut sum_x = 0.0;
        	let mut sum_y = 0.0;

        	let mut add_unique_point = |pt: [f32; 2]| {
        	    if !unique_points.iter().any(|p| (p[0] - pt[0]).abs() < 0.1 && (p[1] - pt[1]).abs() < 0.1) {
        	        unique_points.push(pt);
        	    }
        	};

            for i in 0..ssector.numsegs {
                let seg_idx = (ssector.firstseg + i) as usize;
                let seg = self.segs[seg_idx];
                
                let v1 = self.vertices[seg.v1 as usize];
                let v2 = self.vertices[seg.v2 as usize];

                let p1 = [v1.x as f32, v1.y as f32];
                let p2 = [v2.x as f32, v2.y as f32];

                add_unique_point(p1);
            	add_unique_point(p2);

                if seg.linedef != -1 {
            	    if let Some(line) = self.linedefs.get(seg.linedef as usize) {
            	        let side_idx = if seg.side == 0 { line.sidenum[0] } else { line.sidenum[1] };
            	        if side_idx != -1 {
            	            if let Some(side) = self.sidedefs.get(side_idx as usize) {
            	                current_sector_id = side.sector;
            	            }
            	        }
            	    }
            	}
            }

			if unique_points.len() < 3 { continue; }

			for pt in &unique_points {
        	    sum_x += pt[0];
        	    sum_y += pt[1];
        	}
        	let center_pt = [sum_x / unique_points.len() as f32, sum_y / unique_points.len() as f32];

        	unique_points.sort_by(|a, b| {
        	    let angle_a = (a[1] - center_pt[1]).atan2(a[0] - center_pt[0]);
        	    let angle_b = (b[1] - center_pt[1]).atan2(b[0] - center_pt[0]);
        	    angle_a.partial_cmp(&angle_b).unwrap_or(std::cmp::Ordering::Equal)
        	});

    	    let sector = self.sectors[current_sector_id as usize];

			let floor_texture_name = String::from_utf8_lossy(&sector.floorpic)
                .trim_matches('\0')
                .trim()
                .to_uppercase();
			let ceil_texture_name = String::from_utf8_lossy(&sector.ceilingpic)
                .trim_matches('\0')
                .trim()
                .to_uppercase();
			let floor_texture_id = texture_ids[&floor_texture_name].0;
			let ceil_texture_id = texture_ids[&ceil_texture_name].0;

			let clamped_light = sector.lightlevel.clamp(0, 255) as f32;
			let light_f32 = clamped_light / 255.0;
			let normalized_light_level = [light_f32, light_f32, light_f32];

			let raw_map_idx = (clamped_light / 8.0).floor() as u32;
			let colormap_idx = 31 - raw_map_idx.clamp(0, 31);

    	    let floor_start_idx = vertices.len() as u16;
        
        	for pt in &unique_points {
        	    vertices.push(Vertex { 
        	        pos: [-(pt[0]), sector.floorheight.into(), pt[1]],
        	        light_level: normalized_light_level,
        	        texture_pos: [pt[0] / 64.0, pt[1] / 64.0],
        	        texture_id: floor_texture_id,
        	        sector_id: current_sector_id as u32,
        	        colormap_idx,
        	    });
        	}

        	let num_pts = unique_points.len() as u16;
        	for i in 1..(num_pts - 1) {
        	    indices.push(floor_start_idx + 0);
        	    indices.push(floor_start_idx + i);
        	    indices.push(floor_start_idx + i + 1);
        	}

    	    let ceil_start_idx = vertices.len() as u16;
        
        	for pt in &unique_points {
        	    vertices.push(Vertex { 
        	        pos: [-(pt[0]), sector.ceilingheight.into(), pt[1]],
        	        light_level: normalized_light_level,
        	        texture_pos: [pt[0] / 64.0, pt[1] / 64.0],
        	        texture_id: ceil_texture_id,
        	        sector_id: current_sector_id as u32,
        	        colormap_idx,
        	    });
        	}

        	for i in 1..(num_pts - 1) {
        	    indices.push(ceil_start_idx + 0);
        	    indices.push(ceil_start_idx + i + 1);
        	    indices.push(ceil_start_idx + i);
        	}
    	}

    	(vertices, indices)
	}
}
