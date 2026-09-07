use wad_parser::Line;

use crate::PlayerInventory;

pub enum PlatType {
	PerpetualRaise,
	DownWaitUpStay,
	RaiseAndChange,
	RaiseToNearestAndChange,
	BlazeDWUS,
}

pub enum DoorType {
	Normal,
	Close30ThenOpen,
	Close,
	Open,
	RaiseIn5Mins,
	BlazeRaise,
	BlazeOpen,
	BlazeClose,
}

pub enum CeilType {
	LowerToFloor,
	RaiseToHighest,
	LowerAndCrush,
	CrushAndRaise,
	FastCrushAndRaise,
	SilentCrushAndRaise,
}

pub enum FloorType {
	// lower floor to highest surrounding floor
	LowerFloor,

	// lower floor to lowest surrounding floor
	LowerFloorToLowest,

	// lower floor to highest surrounding floor VERY FAST
	TurboLower,

	// raise floor to lowest surrounding CEILING
	RaiseFloor,

	// raise floor to next highest surrounding floor
	RaiseFloorToNearest,

	// raise floor to shortest height texture around it
	RaiseToTexture,

	// lower floor to lowest surrounding floor
	//  and change floorpic
	LowerAndChange,

	RaiseFloor24,
	RaiseFloor24AndChange,
	RaiseFloorCrush,

	// raise to next highest floor, turbo-speed
	RaiseFloorTurbo,
	DonutRaise,
	RaiseFloor512,
}

pub enum StairType {
	Build8,  // slowly build by 8
	Turbo16, // quickly build by 16
}

pub fn ev_build_stairs(_line: Line, _type_: StairType) -> bool {
	todo!();
}

pub fn ev_do_plat(_line: Line, _type_: PlatType, _amount: u32) -> bool {
	todo!();
}

pub fn ev_do_door(_line: Line, _type_: DoorType) -> bool {
	todo!();
}

pub fn ev_do_locked_door(_line: Line, _type_: DoorType, _inv: PlayerInventory) -> bool {
	todo!();
}

pub fn ev_do_floor(_line: Line, _type_: FloorType) -> bool {
	todo!();
}

pub fn ev_do_ceiling(_line: Line, _type_: CeilType) -> bool {
	todo!();
}

pub fn ev_do_donut(_line: Line) -> bool {
	todo!();
}

pub fn ev_vertical_door(_line: Line, _inv: PlayerInventory) {
	todo!();
}
