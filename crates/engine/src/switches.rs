use wad_parser::{Level, LineId, TextureId, to_u64};

use crate::{SfxEvent, line_midpoint};

pub struct Button {
    pub slot: u32,
    pub line_id: LineId,
    pub timer: u32,
	pub pos_y: f32,
}

const BUTTONTIME: u32 = 35;

pub(crate) fn p_change_switch_texture(
    level: &mut Level,
    line_id: LineId,
    use_again: bool,
    audio: &mut Vec<SfxEvent>,
    buttons: &mut Vec<Button>,
	pos_y: f32,
) {
    let line = level.geom.lines[line_id.0];
    let Some(side_id) = line.sides.0 else { return };
    let Some(slot) = level.state.sides[side_id.0].switch_slot else { return };

    let cur = TextureId(level.state.switch_ids[slot as usize]);
    let Some(&next) = level.geom.switch_pairs.get(&cur) else { return };
    level.state.switch_ids[slot as usize] = next.0;

    let sfx = if line.special == 11 { b"DSSWTCHX" } else { b"DSSWTCHN" };
	let (x, z) = line_midpoint(level, line_id);
    audio.push(SfxEvent { sfx_id: to_u64(sfx), pos: Some((x, pos_y, z)) });

    if use_again {
        buttons.push(Button { slot, line_id, timer: BUTTONTIME, pos_y });
    }
}

pub fn button_system(
	level: &mut Level, 
	buttons: &mut Vec<Button>, 
	audio: &mut Vec<SfxEvent>
) {
	for i in (0..buttons.len()).rev() {
		buttons[i].timer -= 1;
		if buttons[i].timer == 0 {
			p_change_switch_texture(level, buttons[i].line_id, false, audio, buttons, buttons[i].pos_y);
			buttons.swap_remove(i);
		}
	}
}
