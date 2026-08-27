use wad_parser::{Line, LineFlags};
use crate::{MobjNum, MobjType};

pub fn p_use_special_line(
	mobj_type: &MobjType,
	line: &Line,
) -> bool {
	if mobj_type.type_ != MobjNum::Player {
		if line.flags.contains(LineFlags::SECRET) {
			return false;
		}

		match line.special {
			1 | 32 | 33 | 34 => {}
			_ => return false
		}
	}

	// TODO
	//match line.special {
	//	_ => {}
	//}

	true
}