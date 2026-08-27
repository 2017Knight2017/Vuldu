use phf::{Map, phf_map};
use crate::{Direction, MobjNum};

pub	const NUMAMMO: usize = 4;
pub	const NUMPOWERS: usize = 6;
pub	const NUMWEAPONS: usize = 9;
pub	const NUMCARDS: usize = 6;
pub	const NUMSPRITES: usize = 137;
pub const NUMSTATES: usize = 966;
pub const NUMMOBJTYPES: usize = 137;

pub	const FRICTION: f32 = 0.9375;

pub const TRANSLATION: u32 = 0xc000000;
pub const TRANSSHIFT: u32 = 26;

pub const INVULNTICS: u32 = 30;
pub const INVISTICS: u32 = 60;
pub const INFRATICS: u32 = 120;
pub const IRONTICS: u32 = 60;

pub const USERANGE: f32 = 64.0;
pub const MELEERANGE: f32 = 64.0;
pub const MISSILERANGE: f32 = 2048.0;

pub const PLAYERHEIGHT: f32 = 56.0;
pub const EYEHEIGHT: f32 = 41.0;
pub const MAXRADIUS: f32 = 32.0;

pub const XSPEED: [f32; 8] = [1.0, 0.7171, 0.0, -0.7171, -1.0, -0.7171, 0.0, 0.7171];
pub const YSPEED: [f32; 8] = [0.0, 0.7171, 1.0, 0.7171, 0.0, -0.7171, -1.0, -0.7171];
pub const FLOATSPEED: f32 = 4.0;

pub const MAXSPECHIT: usize = 8;
pub const MAXSOUNDBLOCKS: u8 = 2;

pub const DIAGS: [Direction; 4] = [
	Direction::NorthWest,
	Direction::NorthEast,
	Direction::SouthWest,
	Direction::SouthEast
];

pub const OPPOSITE: [Direction; 8] = [
    Direction::West, 
    Direction::SouthWest, 
    Direction::South, 
    Direction::SouthEast, 
    Direction::East, 
    Direction::NorthEast, 
    Direction::North, 
    Direction::NorthWest, 
];

pub const NUMQUITMESSAGES: usize = 21;
pub const ENDMESSAGE: [&str; NUMQUITMESSAGES] = [
	"please don't leave, there's more\ndemons to toast!",
	"let's beat it -- this is turning\ninto a bloodbath!",
	"i wouldn't leave if i were you.\ndos is much worse.",
	"you're trying to say you like dos\nbetter than me, right?",
	"don't leave yet -- there's a\ndemon around that corner!",
	"ya know, next time you come in here\ni'm gonna toast ya.",
	"go ahead and leave. see if i care.",

	"you want to quit?\nthen, thou hast lost an eighth!",
	"don't go now, there's a \ndimensional shambler waiting\nat the dos prompt!",
	"get outta here and go back\nto your boring programs.",
	"if i were your boss, i'd \n deathmatch ya in a minute!",
	"look, bud. you leave now\nand you forfeit your body count!",
	"just leave. when you come\nback, i'll be waiting with a bat.",
	"you're lucky i don't smack\nyou for thinking about leaving.",

	"fuck you, pussy!\nget the fuck out!",
	"you quit and i'll jizz\nin your cystholes!",
	"if you leave, i'll make\nthe lord drink my jizz.",
	"hey, ron! can we say\n'fuck' in the game?",
	"i'd leave: this is just\nmore monsters and levels.\nwhat a load.",
	"suck it down, asshole!\nyou're a fucking wimp!",
	"don't quit now! we're \nstill spending your money!",
];

pub const MOBJTYPE_BY_DOOMEDNUM: Map<u16, Option<MobjNum>> = phf_map! {
	1u16 => Some(MobjNum::Player),
	2u16 => None,
	3u16 => None,
	4u16 => None,
	5u16 => Some(MobjNum::Misc4),
	6u16 => Some(MobjNum::Misc6),
	7u16 => Some(MobjNum::Spider),
	8u16 => Some(MobjNum::Misc24),
	9u16 => Some(MobjNum::Shotguy),
	10u16 => Some(MobjNum::Misc68),
	11u16 => None,
	12u16 => Some(MobjNum::Misc69),
	13u16 => Some(MobjNum::Misc5),
	14u16 => Some(MobjNum::Teleportman),
	15u16 => Some(MobjNum::Misc62),
	16u16 => Some(MobjNum::Cyborg),
	17u16 => Some(MobjNum::Misc21),
	18u16 => Some(MobjNum::Misc63),
	19u16 => Some(MobjNum::Misc67),
	20u16 => Some(MobjNum::Misc66),
	21u16 => Some(MobjNum::Misc64),
	22u16 => Some(MobjNum::Misc61),
	23u16 => Some(MobjNum::Misc65),
	24u16 => Some(MobjNum::Misc71),
	25u16 => Some(MobjNum::Misc74),
	26u16 => Some(MobjNum::Misc75),
	27u16 => Some(MobjNum::Misc72),
	28u16 => Some(MobjNum::Misc70),
	29u16 => Some(MobjNum::Misc73),
	30u16 => Some(MobjNum::Misc32),
	31u16 => Some(MobjNum::Misc33),
	32u16 => Some(MobjNum::Misc34),
	33u16 => Some(MobjNum::Misc35),
	34u16 => Some(MobjNum::Misc49),
	35u16 => Some(MobjNum::Misc50),
	36u16 => Some(MobjNum::Misc37),
	37u16 => Some(MobjNum::Misc36),
	38u16 => Some(MobjNum::Misc8),
	39u16 => Some(MobjNum::Misc7),
	40u16 => Some(MobjNum::Misc9),
	41u16 => Some(MobjNum::Misc38),
	42u16 => Some(MobjNum::Misc39),
	43u16 => Some(MobjNum::Misc40),
	44u16 => Some(MobjNum::Misc41),
	45u16 => Some(MobjNum::Misc42),
	46u16 => Some(MobjNum::Misc43),
	47u16 => Some(MobjNum::Misc47),
	48u16 => Some(MobjNum::Misc48),
	49u16 => Some(MobjNum::Misc51),
	50u16 => Some(MobjNum::Misc52),
	51u16 => Some(MobjNum::Misc53),
	52u16 => Some(MobjNum::Misc54),
	53u16 => Some(MobjNum::Misc55),
	54u16 => Some(MobjNum::Misc76),
	55u16 => Some(MobjNum::Misc44),
	56u16 => Some(MobjNum::Misc45),
	57u16 => Some(MobjNum::Misc46),
	58u16 => Some(MobjNum::Shadows),
	59u16 => Some(MobjNum::Misc56),
	60u16 => Some(MobjNum::Misc57),
	61u16 => Some(MobjNum::Misc58),
	62u16 => Some(MobjNum::Misc59),
	63u16 => Some(MobjNum::Misc60),
	64u16 => Some(MobjNum::Vile),
	65u16 => Some(MobjNum::Chainguy),
	66u16 => Some(MobjNum::Undead),
	67u16 => Some(MobjNum::Fatso),
	68u16 => Some(MobjNum::Baby),
	69u16 => Some(MobjNum::Knight),
	70u16 => Some(MobjNum::Misc77),
	71u16 => Some(MobjNum::Pain),
	72u16 => Some(MobjNum::Keen),
	73u16 => Some(MobjNum::Misc78),
	74u16 => Some(MobjNum::Misc79),
	75u16 => Some(MobjNum::Misc80),
	76u16 => Some(MobjNum::Misc81),
	77u16 => Some(MobjNum::Misc82),
	78u16 => Some(MobjNum::Misc83),
	79u16 => Some(MobjNum::Misc84),
	80u16 => Some(MobjNum::Misc85),
	81u16 => Some(MobjNum::Misc86),
	82u16 => Some(MobjNum::Supershotgun),
	83u16 => Some(MobjNum::Mega),
	84u16 => Some(MobjNum::Wolfss),
	85u16 => Some(MobjNum::Misc29),
	86u16 => Some(MobjNum::Misc30),
	87u16 => Some(MobjNum::Bosstarget),
	88u16 => Some(MobjNum::Bossbrain),
	89u16 => Some(MobjNum::Bossspit),
	2001u16 => Some(MobjNum::Shotgun),
	2002u16 => Some(MobjNum::Chaingun),
	2003u16 => Some(MobjNum::Misc27),
	2004u16 => Some(MobjNum::Misc28),
	2005u16 => Some(MobjNum::Misc26),
	2006u16 => Some(MobjNum::Misc25),
	2007u16 => Some(MobjNum::Clip),
	2008u16 => Some(MobjNum::Misc22),
	2010u16 => Some(MobjNum::Misc18),
	2011u16 => Some(MobjNum::Misc10),
	2012u16 => Some(MobjNum::Misc11),
	2013u16 => Some(MobjNum::Misc12),
	2014u16 => Some(MobjNum::Misc2),
	2015u16 => Some(MobjNum::Misc3),
	2018u16 => Some(MobjNum::Misc0),
	2019u16 => Some(MobjNum::Misc1),
	2022u16 => Some(MobjNum::Inv),
	2023u16 => Some(MobjNum::Misc13),
	2024u16 => Some(MobjNum::Ins),
	2025u16 => Some(MobjNum::Misc14),
	2026u16 => Some(MobjNum::Misc15),
	2028u16 => Some(MobjNum::Misc31),
	2035u16 => Some(MobjNum::Barrel),
	2045u16 => Some(MobjNum::Misc16),
	2046u16 => Some(MobjNum::Misc19),
	2047u16 => Some(MobjNum::Misc20),
	2048u16 => Some(MobjNum::Misc17),
	2049u16 => Some(MobjNum::Misc23),
	3001u16 => Some(MobjNum::Troop),
	3002u16 => Some(MobjNum::Sergeant),
	3003u16 => Some(MobjNum::Bruiser),
	3004u16 => Some(MobjNum::Possessed),
	3005u16 => Some(MobjNum::Head),
	3006u16 => Some(MobjNum::Skull),
};
