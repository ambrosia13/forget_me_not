use crate::game::event;
use bevy_ecs::prelude::*;
use winit::event::{DeviceId, ElementState, KeyEvent, MouseButton};

pub fn init(world: &mut World) {
    world.insert_resource(MouseMotion {
        delta_x: 0.0,
        delta_y: 0.0,
    });
}

#[derive(Event)]
pub struct KeyboardInputEvent {
    pub device_id: DeviceId,
    pub event: KeyEvent,
    pub is_synthetic: bool,
}

#[derive(Event)]
pub struct MouseInputEvent {
    pub device_id: DeviceId,
    pub state: ElementState,
    pub button: MouseButton,
}

#[derive(Resource)]
pub struct MouseMotion {
    pub delta_x: f64,
    pub delta_y: f64,
}
