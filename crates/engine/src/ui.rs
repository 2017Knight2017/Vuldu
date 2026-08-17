use strum::IntoEnumIterator;
use wad_parser::{SCREEN_HEIGHT, Ui};

use crate::{AmmoType, Card, Health, PlayerInfo, WeaponType};

pub fn get_stbar(player_info: &PlayerInfo, player_health: &Health, stbar_height: f32, num_widths: (f32, f32)) -> Vec<(Ui, f32, f32)> {
	let mut ui_to_render = Vec::with_capacity(1 + 3 + 5 + 6 + 1 + 4 + 3 + 24);

	let stbar_y_offset = SCREEN_HEIGHT - stbar_height;
	ui_to_render.push((Ui::STBAR, 0.0, stbar_y_offset));

	let (stt_num_width, stys_num_width) = num_widths;
	let stt_y_offset = 4.0;

	/* AMMO */
	let weapon = player_info.inventory.ready_weapon;
	let ammo = AmmoType::from(weapon);

	if ammo != AmmoType::NoAmmo {
		let ammo_x_offset = 28.0;

		let _ = big_red(
			player_info.inventory.ammo[ammo as usize] as i32, 
			&mut ui_to_render, ammo_x_offset, stbar_y_offset, stt_num_width, stt_y_offset
		);
	}

	/* HEALTH */
	let health_x_offset = 90.0; 
	ui_to_render.push((Ui::STTPRCNT, health_x_offset, stbar_y_offset + stt_y_offset));

	let idx = big_red(
		player_health.0.abs(), 
		&mut ui_to_render, health_x_offset - stt_num_width, stbar_y_offset, stt_num_width, stt_y_offset
	);

	if player_health.0.is_negative() {
		ui_to_render.push((
			Ui::STTMINUS, 
			health_x_offset - stt_num_width * idx,
			stbar_y_offset + stt_y_offset
		));
	}
	
	/* ARMS */
	let arms_x_offset = 110.0;
	let arms_y_offset = 4.0;
	let distance_x_arms = 12.0;
	let distance_y_arms = 10.0;
	let weapon_owned = &player_info.inventory.weapon_owned;
	
	ui_to_render.push((
		match weapon_owned[WeaponType::Pistol as usize] {
			true => single_yellow(2),
			false => single_gray(2)
		},
		arms_x_offset,
		stbar_y_offset + arms_y_offset
	));

	ui_to_render.push((
		match (weapon_owned[WeaponType::Shotgun as usize], weapon_owned[WeaponType::SuperShotgun as usize]) {
			(false, false) => single_gray(3),
			_ => single_yellow(3),
		},
		arms_x_offset + distance_x_arms,
		stbar_y_offset + arms_y_offset
	));

	ui_to_render.push((
		match weapon_owned[WeaponType::Chaingun as usize] {
			true => single_yellow(4),
			false => single_gray(4)
		},
		arms_x_offset + distance_x_arms * 2.0,
		stbar_y_offset + arms_y_offset
	));

	ui_to_render.push((
		match weapon_owned[WeaponType::Missile as usize] {
			true => single_yellow(5),
			false => single_gray(5)
		},
		arms_x_offset,
		stbar_y_offset + arms_y_offset + distance_y_arms
	));

	ui_to_render.push((
		match weapon_owned[WeaponType::Plasma as usize] {
			true => single_yellow(6),
			false => single_gray(6)
		},
		arms_x_offset + distance_x_arms,
		stbar_y_offset + arms_y_offset + distance_y_arms
	));

	ui_to_render.push((
		match weapon_owned[WeaponType::BFG as usize] {
			true => single_yellow(7),
			false => single_gray(7)
		},
		arms_x_offset + distance_x_arms * 2.0,
		stbar_y_offset + arms_y_offset + distance_y_arms
	));

	/* FACE */
	ui_to_render.push((
		Ui::STFST00,
		149.0,
		stbar_y_offset + 2.0
	));

	/* ARMOR */
	let armor_x_offset = 221.0; 
	ui_to_render.push((Ui::STTPRCNT, armor_x_offset, stbar_y_offset + stt_y_offset));

	let _ = big_red(
		player_info.stats.armor_points as i32, 
		&mut ui_to_render, armor_x_offset - stt_num_width, stbar_y_offset, stt_num_width, stt_y_offset
	);

	/* KEYS */
	let keys_x_offset = 239.0;
	if player_info.inventory.cards[Card::BlueSkull as usize] {
		ui_to_render.push((
			Ui::STKEYS3,
			keys_x_offset,
			stbar_y_offset + 3.1
		));
	} else if player_info.inventory.cards[Card::BlueCard as usize] {
		ui_to_render.push((
			Ui::STKEYS0,
			keys_x_offset,
			stbar_y_offset + 4.0
		));
	}

	if player_info.inventory.cards[Card::YellowSkull as usize] {
		ui_to_render.push((
			Ui::STKEYS4,
			keys_x_offset,
			stbar_y_offset + 13.1
		));
	} else if player_info.inventory.cards[Card::YellowCard as usize] {
		ui_to_render.push((
			Ui::STKEYS1,
			keys_x_offset,
			stbar_y_offset + 14.0
		));
	}

	if player_info.inventory.cards[Card::RedSkull as usize] {
		ui_to_render.push((
			Ui::STKEYS5,
			keys_x_offset,
			stbar_y_offset + 23.1
		));
	} else if player_info.inventory.cards[Card::RedCard as usize] {
		ui_to_render.push((
			Ui::STKEYS2,
			keys_x_offset,
			stbar_y_offset + 24.0
		));
	}

	/* TOTAL AMMO */
	let total_ammo_x_offset = 284.0;
	let total_ammo_y_offset = 5.0;
	let distance_y_total_ammo = 6.0;
	for ammo in AmmoType::iter() {
		match ammo {
			AmmoType::Clip => {
				yellow(player_info.inventory.ammo[ammo as usize], 
					&mut ui_to_render, total_ammo_x_offset, stbar_y_offset, stys_num_width, total_ammo_y_offset);
				yellow(player_info.inventory.ammo[ammo as usize], 
					&mut ui_to_render, total_ammo_x_offset + 25.0, stbar_y_offset, stys_num_width, total_ammo_y_offset);
			}
			AmmoType::Shell => {
				yellow(player_info.inventory.ammo[ammo as usize], 
					&mut ui_to_render, total_ammo_x_offset, stbar_y_offset, stys_num_width, total_ammo_y_offset + distance_y_total_ammo);
				yellow(player_info.inventory.ammo[ammo as usize], 
					&mut ui_to_render, total_ammo_x_offset + 25.0, stbar_y_offset, stys_num_width, total_ammo_y_offset + distance_y_total_ammo);
			}
			AmmoType::Missile => {
				yellow(player_info.inventory.ammo[ammo as usize], 
					&mut ui_to_render, total_ammo_x_offset, stbar_y_offset, stys_num_width, total_ammo_y_offset + distance_y_total_ammo * 2.0);
				yellow(player_info.inventory.ammo[ammo as usize], 
					&mut ui_to_render, total_ammo_x_offset + 25.0, stbar_y_offset, stys_num_width, total_ammo_y_offset + distance_y_total_ammo * 2.0);
			}
			AmmoType::Cell => {
				yellow(player_info.inventory.ammo[ammo as usize], 
					&mut ui_to_render, total_ammo_x_offset, stbar_y_offset, stys_num_width, total_ammo_y_offset + distance_y_total_ammo * 3.0);
				yellow(player_info.inventory.ammo[ammo as usize], 
					&mut ui_to_render, total_ammo_x_offset + 25.0, stbar_y_offset, stys_num_width, total_ammo_y_offset + distance_y_total_ammo * 3.0);
			}
			AmmoType::NoAmmo => {}
		}
	}

	ui_to_render
}

fn big_red(
	mut n: i32, 
	ui_to_render: &mut Vec<(Ui, f32, f32)>, 
	num_x_offset: f32, 
	stbar_y_offset: f32,
	stt_num_width: f32,
	stt_y_offset: f32,
) -> f32 {
	let mut idx = 0.0;

	let ones = n % 10;
	ui_to_render.push((
		single_big_red(ones), 
		num_x_offset - stt_num_width * idx,
		stbar_y_offset + stt_y_offset
	));
	idx += 1.0;
	n -= ones;
	n /= 10;

	while n > 0 {
		let ones = n % 10;
		ui_to_render.push((
			single_big_red(ones), 
			num_x_offset - stt_num_width * idx,
			stbar_y_offset + stt_y_offset
		));
		idx += 1.0;
		n -= ones;
		n /= 10;
	}

	idx
}

fn yellow(
	mut n: u32, 
	ui_to_render: &mut Vec<(Ui, f32, f32)>, 
	num_x_offset: f32, 
	stbar_y_offset: f32,
	stt_num_width: f32,
	stt_y_offset: f32,
) -> f32 {
	let mut idx = 0.0;

	let ones = n % 10;
	ui_to_render.push((
		single_yellow(ones), 
		num_x_offset - stt_num_width * idx,
		stbar_y_offset + stt_y_offset
	));
	idx += 1.0;
	n -= ones;
	n /= 10;

	while n > 0 {
		let ones = n % 10;
		ui_to_render.push((
			single_yellow(ones), 
			num_x_offset - stt_num_width * idx,
			stbar_y_offset + stt_y_offset
		));
		idx += 1.0;
		n -= ones;
		n /= 10;
	}

	idx
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
