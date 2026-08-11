use hecs::Entity;

pub enum WorldEvent {
    DamageMobj {
        target: Entity,
        inflictor: Entity,
        damage: u32,
    },
    ResetSkullFly {
        actor_id: Entity,
    },
    TouchSpecialThing {
        special_item: Entity,
        picker: Entity,
    },
}

pub fn execute_events_system(world_events: &mut Vec<WorldEvent>) {
	for event in world_events.drain(..) {
		match event {
			WorldEvent::DamageMobj { target: _, inflictor: _, damage: _ } => {

			},
			WorldEvent::ResetSkullFly { actor_id: _ } => {

			}
			WorldEvent::TouchSpecialThing { special_item: _, picker: _ } => {

			}
		}
	}
}