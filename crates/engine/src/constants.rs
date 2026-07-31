use phf::{Map, phf_map};
use crate::MobjNum;

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
pub const MAXRADIUS: f32 = 32.0;

pub const XSPEED: [f32; 8] = [1.0, 0.7171, 0.0, -0.7171, -1.0, -0.7171, 0.0, 0.7171];
pub const YSPEED: [f32; 8] = [0.0, 0.7171, 1.0, 0.7171, 0.0, -0.7171, -1.0, -0.7171];
pub const FLOATSPEED: f32 = 4.0;

pub const NUMQUITMESSAGES: usize = 21;
pub const ENDMESSAGE: [&'static str; NUMQUITMESSAGES] = [
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

pub const MOBJTYPE_BY_DOOMEDNUM: Map<i16, Option<MobjNum>> = phf_map! {
	1i16 => Some(MobjNum::Player),
	2i16 => None,
	3i16 => None,
	4i16 => None,
	5i16 => Some(MobjNum::Misc4),
	6i16 => Some(MobjNum::Misc6),
	7i16 => Some(MobjNum::Spider),
	8i16 => Some(MobjNum::Misc24),
	9i16 => Some(MobjNum::Shotguy),
	10i16 => Some(MobjNum::Misc68),
	11i16 => None,
	12i16 => Some(MobjNum::Misc69),
	13i16 => Some(MobjNum::Misc5),
	14i16 => Some(MobjNum::Teleportman),
	15i16 => Some(MobjNum::Misc62),
	16i16 => Some(MobjNum::Cyborg),
	17i16 => Some(MobjNum::Misc21),
	18i16 => Some(MobjNum::Misc63),
	19i16 => Some(MobjNum::Misc67),
	20i16 => Some(MobjNum::Misc66),
	21i16 => Some(MobjNum::Misc64),
	22i16 => Some(MobjNum::Misc61),
	23i16 => Some(MobjNum::Misc65),
	24i16 => Some(MobjNum::Misc71),
	25i16 => Some(MobjNum::Misc74),
	26i16 => Some(MobjNum::Misc75),
	27i16 => Some(MobjNum::Misc72),
	28i16 => Some(MobjNum::Misc70),
	29i16 => Some(MobjNum::Misc73),
	30i16 => Some(MobjNum::Misc32),
	31i16 => Some(MobjNum::Misc33),
	32i16 => Some(MobjNum::Misc34),
	33i16 => Some(MobjNum::Misc35),
	34i16 => Some(MobjNum::Misc49),
	35i16 => Some(MobjNum::Misc50),
	36i16 => Some(MobjNum::Misc37),
	37i16 => Some(MobjNum::Misc36),
	38i16 => Some(MobjNum::Misc8),
	39i16 => Some(MobjNum::Misc7),
	40i16 => Some(MobjNum::Misc9),
	41i16 => Some(MobjNum::Misc38),
	42i16 => Some(MobjNum::Misc39),
	43i16 => Some(MobjNum::Misc40),
	44i16 => Some(MobjNum::Misc41),
	45i16 => Some(MobjNum::Misc42),
	46i16 => Some(MobjNum::Misc43),
	47i16 => Some(MobjNum::Misc47),
	48i16 => Some(MobjNum::Misc48),
	49i16 => Some(MobjNum::Misc51),
	50i16 => Some(MobjNum::Misc52),
	51i16 => Some(MobjNum::Misc53),
	52i16 => Some(MobjNum::Misc54),
	53i16 => Some(MobjNum::Misc55),
	54i16 => Some(MobjNum::Misc76),
	55i16 => Some(MobjNum::Misc44),
	56i16 => Some(MobjNum::Misc45),
	57i16 => Some(MobjNum::Misc46),
	58i16 => Some(MobjNum::Shadows),
	59i16 => Some(MobjNum::Misc56),
	60i16 => Some(MobjNum::Misc57),
	61i16 => Some(MobjNum::Misc58),
	62i16 => Some(MobjNum::Misc59),
	63i16 => Some(MobjNum::Misc60),
	64i16 => Some(MobjNum::Vile),
	65i16 => Some(MobjNum::Chainguy),
	66i16 => Some(MobjNum::Undead),
	67i16 => Some(MobjNum::Fatso),
	68i16 => Some(MobjNum::Baby),
	69i16 => Some(MobjNum::Knight),
	70i16 => Some(MobjNum::Misc77),
	71i16 => Some(MobjNum::Pain),
	72i16 => Some(MobjNum::Keen),
	73i16 => Some(MobjNum::Misc78),
	74i16 => Some(MobjNum::Misc79),
	75i16 => Some(MobjNum::Misc80),
	76i16 => Some(MobjNum::Misc81),
	77i16 => Some(MobjNum::Misc82),
	78i16 => Some(MobjNum::Misc83),
	79i16 => Some(MobjNum::Misc84),
	80i16 => Some(MobjNum::Misc85),
	81i16 => Some(MobjNum::Misc86),
	82i16 => Some(MobjNum::Supershotgun),
	83i16 => Some(MobjNum::Mega),
	84i16 => Some(MobjNum::Wolfss),
	85i16 => Some(MobjNum::Misc29),
	86i16 => Some(MobjNum::Misc30),
	87i16 => Some(MobjNum::Bosstarget),
	88i16 => Some(MobjNum::Bossbrain),
	89i16 => Some(MobjNum::Bossspit),
	2001i16 => Some(MobjNum::Shotgun),
	2002i16 => Some(MobjNum::Chaingun),
	2003i16 => Some(MobjNum::Misc27),
	2004i16 => Some(MobjNum::Misc28),
	2005i16 => Some(MobjNum::Misc26),
	2006i16 => Some(MobjNum::Misc25),
	2007i16 => Some(MobjNum::Clip),
	2008i16 => Some(MobjNum::Misc22),
	2010i16 => Some(MobjNum::Misc18),
	2011i16 => Some(MobjNum::Misc10),
	2012i16 => Some(MobjNum::Misc11),
	2013i16 => Some(MobjNum::Misc12),
	2014i16 => Some(MobjNum::Misc2),
	2015i16 => Some(MobjNum::Misc3),
	2018i16 => Some(MobjNum::Misc0),
	2019i16 => Some(MobjNum::Misc1),
	2022i16 => Some(MobjNum::Inv),
	2023i16 => Some(MobjNum::Misc13),
	2024i16 => Some(MobjNum::Ins),
	2025i16 => Some(MobjNum::Misc14),
	2026i16 => Some(MobjNum::Misc15),
	2028i16 => Some(MobjNum::Misc31),
	2035i16 => Some(MobjNum::Barrel),
	2045i16 => Some(MobjNum::Misc16),
	2046i16 => Some(MobjNum::Misc19),
	2047i16 => Some(MobjNum::Misc20),
	2048i16 => Some(MobjNum::Misc17),
	2049i16 => Some(MobjNum::Misc23),
	3001i16 => Some(MobjNum::Troop),
	3002i16 => Some(MobjNum::Sergeant),
	3003i16 => Some(MobjNum::Bruiser),
	3004i16 => Some(MobjNum::Possessed),
	3005i16 => Some(MobjNum::Head),
	3006i16 => Some(MobjNum::Skull),
};
