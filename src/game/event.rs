use bevy_ecs::prelude::*;

pub fn init_event<T: Event>(world: &mut World) {
    world.insert_resource(Events::<T>::default());
}

pub fn clear_events<T: Event>(mut events: ResMut<Events<T>>) {
    events.update();
}
