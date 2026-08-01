use wad_parser::{AABB, DoomMap, MapLinedef};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SlopeType {
    Horizontal,
    Vertical,
    Positive,
    Negative,
}

fn get_slope_type(dx: f32, dy: f32) -> SlopeType {
    if dx == 0.0 {
        SlopeType::Vertical
    } else if dy == 0.0 {
        SlopeType::Horizontal
    } else if (dx > 0.0 && dy > 0.0) || (dx < 0.0 && dy < 0.0) {
        SlopeType::Positive
    } else {
        SlopeType::Negative
    }
}

pub fn p_point_on_line_side(x: f32, y: f32, line: &MapLinedef, map: &DoomMap) -> i32 {
    let v1 = map.vertices[line.v1 as usize];
	let v2 = map.vertices[line.v2 as usize];

    let line_dx = (v2.x - v1.x) as f32;
    let line_dy = (v2.y - v1.y) as f32;

    if line_dx == 0.0 {
        if x <= v1.x as f32 {
            return if line_dy > 0.0 { 1 } else { 0 };
        }
        return if line_dy < 0.0 { 1 } else { 0 };
    }

    if line_dy == 0.0 {
        if y <= v1.y as f32 {
            return if line_dx < 0.0 { 1 } else { 0 };
        }
        return if line_dx > 0.0 { 1 } else { 0 };
    }

    let dx = x - v1.x as f32;
    let dy = y - v1.y as f32;

    let left = line_dy * dx;
    let right = dy * line_dx;

    if right < left {
        0
    } else {
        1
    }
}

pub fn p_box_on_line_side(bbox: &AABB, line: &MapLinedef, map: &DoomMap) -> i32 {
    let v1 = map.vertices[line.v1 as usize];
	let v2 = map.vertices[line.v2 as usize];
    let v1_x = v1.x as f32;
    let v1_y = v1.y as f32;

    let line_dx = (v2.x - v1.x) as f32;
    let line_dy = (v2.y - v1.y) as f32;

    let slope = get_slope_type(line_dx, line_dy);

    let p1: i32;
    let p2: i32;

    match slope {
        SlopeType::Horizontal => {
            p1 = if bbox.max_y > v1_y { 1 } else { 0 };
            p2 = if bbox.min_y > v1_y { 1 } else { 0 };

            if line_dx < 0.0 {
                let p1_flipped = p1 ^ 1;
                let p2_flipped = p2 ^ 1;
                if p1_flipped == p2_flipped {
                    return p1_flipped;
                }
                return -1;
            }
        }
        SlopeType::Vertical => {
            p1 = if bbox.max_x < v1_x { 1 } else { 0 };
            p2 = if bbox.min_x < v1_x { 1 } else { 0 };

            if line_dy < 0.0 {
                let p1_flipped = p1 ^ 1;
                let p2_flipped = p2 ^ 1;
                if p1_flipped == p2_flipped {
                    return p1_flipped;
                }
                return -1;
            }
        }
        SlopeType::Positive => {
            p1 = p_point_on_line_side(bbox.min_x, bbox.max_y, line, map);
            p2 = p_point_on_line_side(bbox.max_x, bbox.min_y, line, map);
        }
        SlopeType::Negative => {
            p1 = p_point_on_line_side(bbox.max_x, bbox.max_y, line, map);
            p2 = p_point_on_line_side(bbox.min_x, bbox.min_y, line, map);
        }
    }

    if p1 == p2 {
        p1
    } else {
        -1
    }
}
