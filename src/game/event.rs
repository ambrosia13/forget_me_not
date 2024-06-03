use bevy_ecs::prelude::*;

pub fn cleanup_events<T: Event>(mut events: ResMut<Events<T>>) {
    events.update();
}
