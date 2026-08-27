use hecs::{Entity, World};
use wad_parser::{SCREEN_HEIGHT, Ui};

use crate::{AmmoType, Card, Health, NUMCARDS, NUMWEAPONS, PlayerInventory, PlayerStats, WeaponType};

const STT_NUM_WIDTH: f32 = 14.0;
const STYS_NUM_WIDTH: f32 = 4.0; 
const STBAR_Y_OFFSET: f32 = SCREEN_HEIGHT - 32.0;
const NUM_Y_OFFSET: f32 = 4.0;

pub struct STBarUi {
	pub stbar: [(Ui, f32, f32); 1],
    pub ammo: Vec<(Ui, f32, f32)>,
    pub hp: Vec<(Ui, f32, f32)>,
    pub arms: Vec<(Ui, f32, f32)>,
    pub face: [(Ui, f32, f32); 1],
    pub armor: Vec<(Ui, f32, f32)>,
    pub keys: Vec<(Ui, f32, f32)>,
    pub total_ammo: Vec<(Ui, f32, f32)>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UpdatableUiType {
	Ammo,
	Hp,
    Arms,
    Face,
    Armor,
    Keys,
    TotalAmmo,
}

impl Default for STBarUi {
    fn default() -> Self {
        STBarUi { 
			stbar: [(Ui::STBAR, 0.0, STBAR_Y_OFFSET)], 
			ammo: Vec::with_capacity(3), 
			hp: Vec::with_capacity(5), 
			arms: Vec::with_capacity(6), 
			face: [(Ui::STFST00, 149.0, STBAR_Y_OFFSET + 2.0)], 
			armor: Vec::with_capacity(4), 
			keys: Vec::with_capacity(3), 
			total_ammo: Vec::with_capacity(24) 
		}
    }
}

pub fn get_stbar(world: &World, player_entity: Entity, stbar_ui: &mut STBarUi) {
	let inventory = world.get::<&PlayerInventory>(player_entity).unwrap();
	let stats = world.get::<&PlayerStats>(player_entity).unwrap();
	let hp = world.get::<&Health>(player_entity).unwrap();

	update_ammo_ui(&inventory, &mut stbar_ui.ammo);
	
	update_hp_ui(&hp, &mut stbar_ui.hp);

	update_arms_ui(&inventory.weapon_owned, &mut stbar_ui.arms);
	
	update_armor_ui(stats.armor_points, &mut stbar_ui.armor);

	update_keys_ui(&inventory.cards, &mut stbar_ui.keys);

	update_total_ammo_ui(&inventory, &mut stbar_ui.total_ammo);
}

pub fn update_total_ammo_ui(inventory: &PlayerInventory, ui_to_render: &mut Vec<(Ui, f32, f32)>) {
	let total_ammo_x_offset = 289.0;
	let total_ammo_y_offset = 5.0;
	let distance_y_total_ammo = 6.0;

	ui_to_render.clear();
	
	yellow(inventory.ammo[AmmoType::Clip as usize], 
		ui_to_render, total_ammo_x_offset, total_ammo_y_offset);
	yellow(if inventory.backpack { 400 } else { 200 }, 
		ui_to_render, total_ammo_x_offset + 25.0, total_ammo_y_offset);
		
	yellow(inventory.ammo[AmmoType::Shell as usize], 
		ui_to_render, total_ammo_x_offset, total_ammo_y_offset + distance_y_total_ammo);
	yellow(if inventory.backpack { 100 } else { 50 }, 
		ui_to_render, total_ammo_x_offset + 25.0, total_ammo_y_offset + distance_y_total_ammo);
		
	yellow(inventory.ammo[AmmoType::Missile as usize], 
		ui_to_render, total_ammo_x_offset, total_ammo_y_offset + distance_y_total_ammo * 2.0);
	yellow(if inventory.backpack { 100 } else { 50 }, 
		ui_to_render, total_ammo_x_offset + 25.0, total_ammo_y_offset + distance_y_total_ammo * 2.0);
		
	yellow(inventory.ammo[AmmoType::Cell as usize], 
		ui_to_render, total_ammo_x_offset, total_ammo_y_offset + distance_y_total_ammo * 3.0);
	yellow(if inventory.backpack { 600 } else { 300 }, 
		ui_to_render, total_ammo_x_offset + 25.0, total_ammo_y_offset + distance_y_total_ammo * 3.0);
}

pub fn update_keys_ui(cards: &[bool; NUMCARDS], ui_to_render: &mut Vec<(Ui, f32, f32)>) {
	let keys_x_offset = 239.0;

	ui_to_render.clear();

	if cards[Card::BlueSkull as usize] {
		ui_to_render.push((
			Ui::STKEYS3,
			keys_x_offset,
			STBAR_Y_OFFSET + 3.1
		));
	} else if cards[Card::BlueCard as usize] {
		ui_to_render.push((
			Ui::STKEYS0,
			keys_x_offset,
			STBAR_Y_OFFSET + 4.0
		));
	}

	if cards[Card::YellowSkull as usize] {
		ui_to_render.push((
			Ui::STKEYS4,
			keys_x_offset,
			STBAR_Y_OFFSET + 13.1
		));
	} else if cards[Card::YellowCard as usize] {
		ui_to_render.push((
			Ui::STKEYS1,
			keys_x_offset,
			STBAR_Y_OFFSET + 14.0
		));
	}

	if cards[Card::RedSkull as usize] {
		ui_to_render.push((
			Ui::STKEYS5,
			keys_x_offset,
			STBAR_Y_OFFSET + 23.1
		));
	} else if cards[Card::RedCard as usize] {
		ui_to_render.push((
			Ui::STKEYS2,
			keys_x_offset,
			STBAR_Y_OFFSET + 24.0
		));
	}
}

pub fn update_armor_ui(armor_points: u32, ui_to_render: &mut Vec<(Ui, f32, f32)>) {
	let armor_x_offset = 221.0; 

	ui_to_render.clear();
	
	ui_to_render.push((Ui::STTPRCNT, armor_x_offset, STBAR_Y_OFFSET + NUM_Y_OFFSET));

	big_red(armor_points as i32, ui_to_render, armor_x_offset);
}

pub fn update_face_ui(ui_to_render: &mut [(Ui, f32, f32); 1]) {
	ui_to_render[0] = (Ui::STFST00, 149.0, STBAR_Y_OFFSET + 2.0);
}

pub fn update_arms_ui(weapon_owned: &[bool; NUMWEAPONS], ui_to_render: &mut Vec<(Ui, f32, f32)>) {
	let arms_x_offset = 111.0;
	let arms_y_offset = 4.0;
	let distance_x_arms = 12.0;
	let distance_y_arms = 10.0;

	ui_to_render.clear();

	ui_to_render.push((
		Ui::STARMS,
		104.0,
		STBAR_Y_OFFSET
	));
	
	ui_to_render.push((
		match weapon_owned[WeaponType::Pistol as usize] {
			true => single_yellow(2),
			false => single_gray(2)
		},
		arms_x_offset,
		STBAR_Y_OFFSET + arms_y_offset
	));

	ui_to_render.push((
		match (weapon_owned[WeaponType::Shotgun as usize], weapon_owned[WeaponType::SuperShotgun as usize]) {
			(false, false) => single_gray(3),
			_ => single_yellow(3),
		},
		arms_x_offset + distance_x_arms,
		STBAR_Y_OFFSET + arms_y_offset
	));

	ui_to_render.push((
		match weapon_owned[WeaponType::Chaingun as usize] {
			true => single_yellow(4),
			false => single_gray(4)
		},
		arms_x_offset + distance_x_arms * 2.0,
		STBAR_Y_OFFSET + arms_y_offset
	));

	ui_to_render.push((
		match weapon_owned[WeaponType::Missile as usize] {
			true => single_yellow(5),
			false => single_gray(5)
		},
		arms_x_offset,
		STBAR_Y_OFFSET + arms_y_offset + distance_y_arms
	));

	ui_to_render.push((
		match weapon_owned[WeaponType::Plasma as usize] {
			true => single_yellow(6),
			false => single_gray(6)
		},
		arms_x_offset + distance_x_arms,
		STBAR_Y_OFFSET + arms_y_offset + distance_y_arms
	));

	ui_to_render.push((
		match weapon_owned[WeaponType::BFG as usize] {
			true => single_yellow(7),
			false => single_gray(7)
		},
		arms_x_offset + distance_x_arms * 2.0,
		STBAR_Y_OFFSET + arms_y_offset + distance_y_arms
	));
}

pub fn update_hp_ui(player_health: &Health, ui_to_render: &mut Vec<(Ui, f32, f32)>) {
	let health_x_offset = 90.0; 
	
	ui_to_render.clear();

	ui_to_render.push((Ui::STTPRCNT, health_x_offset, STBAR_Y_OFFSET + NUM_Y_OFFSET));

	let idx = big_red(player_health.0.abs(), ui_to_render, health_x_offset);

	if player_health.0.is_negative() {
		ui_to_render.push((
			Ui::STTMINUS, 
			health_x_offset - STT_NUM_WIDTH * idx - 6.0,
			STBAR_Y_OFFSET + NUM_Y_OFFSET + 5.0
		));
	}
}

pub fn update_ammo_ui(inventory: &PlayerInventory, ui_to_render: &mut Vec<(Ui, f32, f32)>) {
	let weapon = inventory.ready_weapon;
	let ammo_type = AmmoType::from(weapon);

	ui_to_render.clear();

	if ammo_type != AmmoType::NoAmmo {
		let ammo_x_offset = 44.0;

		big_red(inventory.ammo[ammo_type as usize] as i32, ui_to_render, ammo_x_offset);
	}
}

fn big_red(
	mut n: i32, 
	ui_to_render: &mut Vec<(Ui, f32, f32)>,
	num_x_offset: f32, 
) -> f32 {
	let num_digits = if n < 10 { 1 } else if n < 100 { 2 } else { 3 };

	let mut offset_1 = 0.0;

	let ones = n % 10;
	if ones == 1 {
		offset_1 += 2.0;
	}
	ui_to_render.push((
		single_big_red(ones), 
		num_x_offset - STT_NUM_WIDTH + offset_1,
		STBAR_Y_OFFSET + NUM_Y_OFFSET
	));
	n -= ones;
	n /= 10;

	if num_digits >= 2 {
		let ones = n % 10;
		if ones == 1 {
			offset_1 += 2.0;
		}
		ui_to_render.push((
			single_big_red(ones), 
			num_x_offset - STT_NUM_WIDTH * 2.0 + offset_1,
			STBAR_Y_OFFSET + NUM_Y_OFFSET
		));
		n -= ones;
		n /= 10;
	}

	if num_digits == 3 {
		let ones = n % 10;
		if ones == 1 {
			offset_1 = 2.0;
		}
		ui_to_render.push((
			single_big_red(ones), 
			num_x_offset - STT_NUM_WIDTH * 3.0 + offset_1,
			STBAR_Y_OFFSET + NUM_Y_OFFSET
		)); 
	}

	num_digits as f32
}

fn yellow(
	mut n: u32, 
	ui_to_render: &mut Vec<(Ui, f32, f32)>,
	num_x_offset: f32, 
	num_y_offset: f32,
) {
	let num_digits = if n < 10 { 1 } else if n < 100 { 2 } else { 3 };

	let ones = n % 10;
	ui_to_render.push((
		single_yellow(ones), 
		num_x_offset - STYS_NUM_WIDTH,
		STBAR_Y_OFFSET + num_y_offset
	));
	n -= ones;
	n /= 10;

	if num_digits >= 2 {
		let ones = n % 10;
		ui_to_render.push((
			single_yellow(ones), 
			num_x_offset - STYS_NUM_WIDTH * 2.0,
			STBAR_Y_OFFSET + num_y_offset
		));
		n -= ones;
		n /= 10;
	}

	if num_digits == 3 {
		let ones = n % 10;
		ui_to_render.push((
			single_yellow(ones), 
			num_x_offset - STYS_NUM_WIDTH * 3.0,
			STBAR_Y_OFFSET + num_y_offset
		));
	}
}

fn single_big_red(n: i32) -> Ui {
	match n {
		0 => Ui::STTNUM0,
		1 => Ui::STTNUM1,
		2 => Ui::STTNUM2,
		3 => Ui::STTNUM3,
		4 => Ui::STTNUM4,
		5 => Ui::STTNUM5,
		6 => Ui::STTNUM6,
		7 => Ui::STTNUM7,
		8 => Ui::STTNUM8,
		9 => Ui::STTNUM9,
		_ => panic!("[single_big_red] Only for numbers in 0..=9")
	}
}

fn single_gray(n: u32) -> Ui {
	match n {
		0 => Ui::STGNUM0,
		1 => Ui::STGNUM1,
		2 => Ui::STGNUM2,
		3 => Ui::STGNUM3,
		4 => Ui::STGNUM4,
		5 => Ui::STGNUM5,
		6 => Ui::STGNUM6,
		7 => Ui::STGNUM7,
		8 => Ui::STGNUM8,
		9 => Ui::STGNUM9,
		_ => panic!("[single_gray] Only for numbers in 0..=9")
	}
}

fn single_yellow(n: u32) -> Ui {
	match n {
		0 => Ui::STYSNUM0,
		1 => Ui::STYSNUM1,
		2 => Ui::STYSNUM2,
		3 => Ui::STYSNUM3,
		4 => Ui::STYSNUM4,
		5 => Ui::STYSNUM5,
		6 => Ui::STYSNUM6,
		7 => Ui::STYSNUM7,
		8 => Ui::STYSNUM8,
		9 => Ui::STYSNUM9,
		_ => panic!("[single_yellow] Only for numbers in 0..=9")
	}
}
