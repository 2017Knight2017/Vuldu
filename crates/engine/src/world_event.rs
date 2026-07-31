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
			WorldEvent::DamageMobj { target, inflictor, damage } => {

			},
			WorldEvent::ResetSkullFly { actor_id } => {

			}
			WorldEvent::TouchSpecialThing { special_item, picker } => {

			}
		}
	}
}