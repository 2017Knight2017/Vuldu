use std::collections::VecDeque;
use engine::WorldEvent;
use winit::keyboard::{KeyCode, NativeKeyCode::Unidentified, PhysicalKey};

pub fn cheat_system(last_buttons: &mut VecDeque<PhysicalKey>, world_events: &mut Vec<WorldEvent>) {
	let mut cheat_activated = false;

	if last_buttons[4] == PhysicalKey::Code(KeyCode::KeyI)
	&& last_buttons[3] == PhysicalKey::Code(KeyCode::KeyD)
	&& last_buttons[2] == PhysicalKey::Code(KeyCode::KeyK)
	&& last_buttons[1] == PhysicalKey::Code(KeyCode::KeyF)
	&& last_buttons[0] == PhysicalKey::Code(KeyCode::KeyA)
	{
		world_events.push(WorldEvent::CheatIDKFA);
		cheat_activated = true;
	} 
	else if last_buttons[3] == PhysicalKey::Code(KeyCode::KeyI)
	&& last_buttons[2] == PhysicalKey::Code(KeyCode::KeyD)
	&& last_buttons[1] == PhysicalKey::Code(KeyCode::KeyF)
	&& last_buttons[0] == PhysicalKey::Code(KeyCode::KeyA)
	{
		world_events.push(WorldEvent::CheatIDFA);
		cheat_activated = true;
	}
	else if last_buttons[3] == PhysicalKey::Code(KeyCode::KeyI)
	&& last_buttons[2] == PhysicalKey::Code(KeyCode::KeyD)
	&& last_buttons[1] == PhysicalKey::Code(KeyCode::KeyQ)
	&& last_buttons[0] == PhysicalKey::Code(KeyCode::KeyD)
	{
		world_events.push(WorldEvent::CheatIDDQD);
		cheat_activated = true;
	}
	else if last_buttons[5] == PhysicalKey::Code(KeyCode::KeyN)
	&& last_buttons[4] == PhysicalKey::Code(KeyCode::KeyO)
	&& last_buttons[3] == PhysicalKey::Code(KeyCode::KeyC)
	&& last_buttons[2] == PhysicalKey::Code(KeyCode::KeyL)
	&& last_buttons[1] == PhysicalKey::Code(KeyCode::KeyI)
	&& last_buttons[0] == PhysicalKey::Code(KeyCode::KeyP)
	{
		world_events.push(WorldEvent::CheatNOCLIP);
		cheat_activated = true;
	}

	if cheat_activated {
		last_buttons.iter_mut().for_each(|btn| *btn = PhysicalKey::Unidentified(Unidentified));
	}
}