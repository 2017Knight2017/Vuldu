use std::f64::consts::TAU;

use crate::{
	Button, Intercept, MobjNum, MobjType, PlayerInventory, PlayerRotation, Position, SfxEvent,
	Traversal, USERANGE, collect_line_intercepts, p_change_switch_texture,
};
use hecs::{Entity, World};
use wad_parser::{Level, Line, LineFlags, LineId, to_u64};

fn ptr_use_traverse(
	level: &mut Level,
	line_id: LineId,
	user_pos: Position,
	mobj: MobjType,
	inv_opt: Option<PlayerInventory>,
	buttons: &mut Vec<Button>,
	audio: &mut Vec<SfxEvent>,
) -> bool {
	let line = level.geom.lines[line_id.0];

	if line.special == 0 {
		let blocked = match level.get_opening(line_id) {
			Some(open) => open.top - open.floor_high <= 0.0,
			None => true,
		};

		if blocked {
			audio.push(SfxEvent {
				sfx_id: to_u64(b"DSNOWAY"),
				pos: Some((user_pos.x, user_pos.y, user_pos.z)),
			});

			return false;
		}

		return true;
	}

	p_use_special_line(
		level,
		line_id,
		mobj,
		inv_opt,
		Some(user_pos.y),
		buttons,
		audio,
	);

	false
}

#[allow(clippy::too_many_arguments)]
pub fn p_use_lines(
	world: &World,
	player_ent: Entity,
	level: &mut Level,
	traversal: &mut Traversal,
	intercepts: &mut Vec<Intercept>,
	buttons: &mut Vec<Button>,
	audio: &mut Vec<SfxEvent>,
) {
	let mut query =
		world.query_one::<(&Position, &PlayerRotation, &MobjType, &PlayerInventory)>(player_ent);

	let Ok((pos, rot, mobj, inv)) = query.get().map(|(p, r, m, i)| (*p, *r, *m, *i)) else {
		return;
	};

	let angle = (rot.angle as f64 / u32::MAX as f64) * TAU;
	let x2 = pos.x + USERANGE * f64::sin(angle) as f32;
	let z2 = pos.z + USERANGE * f64::cos(angle) as f32;

	intercepts.clear();

	{
		let mut pass = traversal.begin();
		collect_line_intercepts(level, &mut pass, pos.x, pos.z, x2, z2, intercepts);
	}

	intercepts.sort_unstable_by(|a, b| a.frac.total_cmp(&b.frac));

	for i in 0..intercepts.len() {
		let line_id = intercepts[i].line_id;

		if !ptr_use_traverse(level, line_id, pos, mobj, Some(inv), buttons, audio) {
			return;
		}
	}
}

pub fn p_cross_special_line() {
	todo!();
}

pub fn p_use_special_line(
	level: &mut Level,
	line_id: LineId,
	mobj: MobjType,
	inv_opt: Option<PlayerInventory>,
	player_y_opt: Option<f32>,
	buttons: &mut Vec<Button>,
	audio: &mut Vec<SfxEvent>,
) -> bool {
	let line = level.geom.lines[line_id.0];

	if level.state.lines[line_id.0].used {
		return false;
	}

	if mobj.type_ != MobjNum::Player {
		if line.flags.contains(LineFlags::SECRET) {
			return false;
		}

		match line.special {
			1 | 32 | 33 | 34 => {}
			_ => return false,
		}
	}

	match line.special {
		//1			// Vertical Door
		//| 26		// Blue Door/Locked
		//| 27		// Yellow Door /Locked
		//| 28		// Red Door /Locked
		//
		//| 31		// Manual door open
		//| 32		// Blue locked door open
		//| 33		// Red locked door open
		//| 34		// Yellow locked door open
		//
		//| 117		// Blazing door raise
		//| 118 		// Blazing door open
		//if let Some(inv) = inv_opt =>
		//	ev_vertical_door(line, inv),
		7 | 9 | 11 | 14 | 15 | 18 | 20 | 21 | 23 | 29 | 41 | 71 | 49 | 50 | 51 | 55 | 101 | 102
		| 103 | 111 | 112 | 113 | 122 | 127 | 131 | 133 | 135 | 137 | 140
			if effect(line, inv_opt) =>
		{
			p_change_switch_texture(level, line_id, false, audio, buttons, player_y_opt);
			level.state.lines[line_id.0].used = true;
		}

		42 | 43 | 45 | 60 | 61 | 62 | 63 | 64 | 66 | 67 | 65 | 68 | 69 | 70 | 114 | 115 | 116
		| 123 | 132 | 99 | 134 | 136 | 138 | 139
			if effect(line, inv_opt) =>
		{
			p_change_switch_texture(level, line_id, true, audio, buttons, player_y_opt);
		}

		_ => return false,
	}

	true
}

fn effect(_line: Line, _inv_opt: Option<PlayerInventory>) -> bool {
	/*
	match line.special {
		// Build Stairs// SWITCHES
		7 if ev_build_stairs(line,StairType::Build8) => return true,

		// Change Donut
		9 if ev_do_donut(line) => return true,

		// Exit level
		//11 => {
		//	G_ExitLevel();
		//	return true;
		//}

		// Raise Floor 32 and change texture
		14 if ev_do_plat(line,PlatType::RaiseAndChange,32) => return true,

		// Raise Floor 24 and change texture
		15 if ev_do_plat(line,PlatType::RaiseAndChange,24) => return true,

		// Raise Floor to next highest floor
		18 if ev_do_floor(line, FloorType::RaiseFloorToNearest) => return true,

		// Raise Plat next highest floor and change texture
		20 if ev_do_plat(line, PlatType::RaiseToNearestAndChange,0) => return true,

		// PlatDownWaitUpStay
		21 if ev_do_plat(line,PlatType::DownWaitUpStay,0) => return true,

		// Lower Floor to Lowest
		23 if ev_do_floor(line,FloorType::LowerFloorToLowest) => return true,

		// Raise Door
		29 if ev_do_door(line,DoorType::Normal) => return true,

		// Lower Ceiling to Floor
		41 if ev_do_ceiling(line,CeilType::LowerToFloor) => return true,

		// Turbo Lower Floor
		71 if ev_do_floor(line,FloorType::TurboLower) => return true,

		// Ceiling Crush And Raise
		49 if ev_do_ceiling(line,CeilType::CrushAndRaise) => return true,

		// Close Door
		50 if ev_do_door(line,DoorType::Close) => return true,

		// Secret EXIT
		//51 => {
		//	return true;
		//	G_SecretExitLevel();
		//}

		// Raise Floor Crush
		55 if ev_do_floor(line,FloorType::RaiseFloorCrush) => return true,

		// Raise Floor
		101 if ev_do_floor(line,FloorType::RaiseFloor) => return true,

		// Lower Floor to Surrounding floor height
		102 if ev_do_floor(line,FloorType::LowerFloor) => return true,

		// Open Door
		103 if ev_do_door(line,DoorType::Open) => return true,

		// Blazing Door Raise (faster than TURBO!)
		111 if ev_do_door(line,DoorType::BlazeRaise) => return true,

		// Blazing Door Open (faster than TURBO!)
		112 if ev_do_door(line,DoorType::BlazeOpen) => return true,

		// Blazing Door Close (faster than TURBO!)
		113 if ev_do_door(line,DoorType::BlazeClose) => return true,

		// Blazing PlatDownWaitUpStay
		122 if ev_do_plat(line,PlatType::BlazeDWUS,0) => return true,

		// Build Stairs Turbo 16
		127 if ev_build_stairs(line,StairType::Turbo16) => return true,

		// Raise Floor Turbo
		131 if ev_do_floor(line,FloorType::RaiseFloorTurbo) => return true,

		// BlzOpenDoor BLUE
		133
		// BlzOpenDoor RED
		| 135
		// BlzOpenDoor YELLOW
		| 137 if let Some(inv) = inv_opt && ev_do_locked_door(line,DoorType::BlazeOpen,inv) => return true,

		// Raise Floor 512
		140 if ev_do_floor(line,FloorType::RaiseFloor512) => return true,

		// Close Door// BUTTONS
		42 if ev_do_door(line,DoorType::Close) => return true,

		// Lower Ceiling to Floor
		43 if ev_do_ceiling(line,CeilType::LowerToFloor) => return true,

		// Lower Floor to Surrounding floor height
		45 if ev_do_floor(line,FloorType::LowerFloor) => return true,

		// Lower Floor to Lowest
		60 if ev_do_floor(line,FloorType::LowerFloorToLowest) => return true,

		// Open Door
		61 if ev_do_door(line,DoorType::Open) => return true,

		// PlatDownWaitUpStay
		62 if ev_do_plat(line,PlatType::DownWaitUpStay,1) => return true,

		// Raise Door
		63 if ev_do_door(line,DoorType::Normal) => return true,

		// Raise Floor to ceiling
		64 if ev_do_floor(line,FloorType::RaiseFloor) => return true,

		// Raise Floor 24 and change texture
		66 if ev_do_plat(line,PlatType::RaiseAndChange,24) => return true,

		// Raise Floor 32 and change texture
		67 if ev_do_plat(line,PlatType::RaiseAndChange,32) => return true,

		// Raise Floor Crush
		65 if ev_do_floor(line,FloorType::RaiseFloorCrush) => return true,

		// Raise Plat to next highest floor and change texture
		68 if ev_do_plat(line,PlatType::RaiseToNearestAndChange,0) => return true,

		// Raise Floor to next highest floor
		69 if ev_do_floor(line, FloorType::RaiseFloorToNearest) => return true,

		// Turbo Lower Floor
		70 if ev_do_floor(line,FloorType::TurboLower) => return true,

		// Blazing Door Raise (faster than TURBO!)
		114 if ev_do_door(line,DoorType::BlazeRaise) => return true,

		// Blazing Door Open (faster than TURBO!)
		115 if ev_do_door(line,DoorType::BlazeOpen) => return true,

		// Blazing Door Close (faster than TURBO!)
		116 if ev_do_door(line,DoorType::BlazeClose) => return true,

		// Blazing PlatDownWaitUpStay
		123 if ev_do_plat(line,PlatType::BlazeDWUS,0) => return true,

		// Raise Floor Turbo
		132 if ev_do_floor(line,FloorType::RaiseFloorTurbo) => return true,

		// BlzOpenDoor BLUE
		99
		// BlzOpenDoor RED
		| 134
		// BlzOpenDoor YELLOW
		| 136
		if let Some(inv) = inv_opt && ev_do_locked_door(line,DoorType::BlazeOpen,inv) => return true,

		// Light Turn On
		//138 => {
		//	ev_LightTurnOn(line,255);
		//	return true;
		//}

		// Light Turn Off
		//139 => {
		//	ev_LightTurnOn(line,35);
		//	return true;
		//}

		_ => return false,
	}
	*/
	true
}
