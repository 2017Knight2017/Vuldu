use wad_parser::LineId;

use crate::WorldEvent;

#[derive(Debug, Clone, Copy)]
pub struct Button {
	pub slot: u32,
	pub line_id: LineId,
	pub timer: u32,
	pub y: f32,
}

pub fn button_system(buttons: &mut Vec<Button>, world_events: &mut Vec<WorldEvent>) {
	for i in (0..buttons.len()).rev() {
		buttons[i].timer -= 1;
		if buttons[i].timer == 0 {
			let line_id = buttons[i].line_id;
			let y = buttons[i].y;

			world_events.push(WorldEvent::ChangeSwitchTex {
				line_id,
				use_again: false,
				single_use: false,
				y,
			});
			buttons.swap_remove(i);
		}
	}
}
