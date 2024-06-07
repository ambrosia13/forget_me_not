use bevy_ecs::prelude::*;
use derived_deref::{Deref, DerefMut};
use std::collections::HashSet;
use std::hash::Hash;
use winit::event::{DeviceId, ElementState, KeyEvent, MouseButton};
use winit::keyboard::{KeyCode, PhysicalKey};

pub fn init(mut commands: Commands) {
    commands.insert_resource(MouseMotion {
        delta_x: 0.0,
        delta_y: 0.0,
    });

    commands.insert_resource(KeyboardInput(Input::new()));
    commands.insert_resource(MouseInput(Input::new()));
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

#[derive(Resource, Deref, DerefMut)]
pub struct KeyboardInput(Input<KeyCode>);

#[derive(Resource, Deref, DerefMut)]
pub struct MouseInput(Input<MouseButton>);

// Input struct referenced from Bevy, MIT license
#[derive(Default)]
pub struct Input<T: Copy + Eq + Hash + Send + Sync + 'static> {
    pressed: HashSet<T>,
    just_pressed: HashSet<T>,
    just_released: HashSet<T>,
}

impl<T: Copy + Eq + Hash + Send + Sync + 'static> Input<T> {
    pub fn new() -> Self {
        Self {
            pressed: HashSet::new(),
            just_pressed: HashSet::new(),
            just_released: HashSet::new(),
        }
    }

    pub fn press(&mut self, input: T) {
        if self.pressed.insert(input) {
            self.just_pressed.insert(input);
        }
    }

    pub fn release(&mut self, input: T) {
        if self.pressed.remove(&input) {
            self.just_released.insert(input);
        }
    }

    pub fn tick(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
    }

    pub fn pressed(&self, input: T) -> bool {
        self.pressed.contains(&input)
    }

    pub fn just_pressed(&self, input: T) -> bool {
        self.just_pressed.contains(&input)
    }

    pub fn just_released(&self, input: T) -> bool {
        self.just_released.contains(&input)
    }
}

pub fn receive_input_events(
    mut keyboard_input: ResMut<KeyboardInput>,
    mut mouse_input: ResMut<MouseInput>,
    mut mouse_events: EventReader<MouseInputEvent>,
    mut keyboard_events: EventReader<KeyboardInputEvent>,
) {
    for event in mouse_events.read() {
        match event.state {
            ElementState::Pressed => mouse_input.press(event.button),
            ElementState::Released => mouse_input.release(event.button),
        }
    }

    for event in keyboard_events.read() {
        let key_code = match event.event.physical_key {
            PhysicalKey::Code(key_code) => key_code,
            PhysicalKey::Unidentified(key) => {
                log::warn!("Unidentified physical key press: {:?}", key);
                return;
            }
        };

        match event.event.state {
            ElementState::Pressed => keyboard_input.press(key_code),
            ElementState::Released => keyboard_input.release(key_code),
        }
    }
}

pub fn tick_input(mut keyboard_input: ResMut<KeyboardInput>, mut mouse_input: ResMut<MouseInput>) {
    keyboard_input.tick();
    mouse_input.tick();
}
