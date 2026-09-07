use wad_parser::{Level, LineId, TextureId, to_u64};

use crate::{SfxEvent, UseContext, line_midpoint};

#[derive(Debug, Clone, Copy)]
pub struct Button {
	pub slot: u32,
	pub line_id: LineId,
	pub timer: u32,
	pub pos_y: Option<f32>,
}

const BUTTONTIME: u32 = 35;

pub(crate) fn p_change_switch_texture(
	ctx: &mut UseContext,
	line_id: LineId,
	use_again: bool,
	pos_y_opt: Option<f32>,
) {
	let line = ctx.level.geom.lines[line_id.0];
	let Some(side_id) = line.sides.0 else { return };
	let Some(slot) = ctx.level.state.sides[side_id.0].switch_slot else {
		return;
	};

	let cur = TextureId(ctx.level.state.switch_ids[slot as usize]);
	let Some(&next) = ctx.level.geom.switch_pairs.get(&cur) else {
		return;
	};
	ctx.level.state.switch_ids[slot as usize] = next.0;

	let sfx = if line.special == 11 {
		b"DSSWTCHX"
	} else {
		b"DSSWTCHN"
	};

	let pos = pos_y_opt.map(|y| {
		let (x, z) = line_midpoint(ctx.level, line_id);
		(x, y, z)
	});

	ctx.audio.push(SfxEvent {
		sfx_id: to_u64(sfx),
		pos,
	});

	if use_again && !ctx.buttons.iter().any(|b| b.slot == slot) {
		ctx.buttons.push(Button {
			slot,
			line_id,
			timer: BUTTONTIME,
			pos_y: pos.map(|(_, y, _)| y),
		});
	}
}

pub fn button_system(level: &mut Level, buttons: &mut Vec<Button>, audio: &mut Vec<SfxEvent>) {
	for i in (0..buttons.len()).rev() {
		buttons[i].timer -= 1;
		if buttons[i].timer == 0 {
			let line_id = buttons[i].line_id;
			let pos_y = buttons[i].pos_y;

			let mut ctx = UseContext {
				level,
				buttons,
				audio,
			};

			p_change_switch_texture(&mut ctx, line_id, false, pos_y);
			buttons.swap_remove(i);
		}
	}
}
