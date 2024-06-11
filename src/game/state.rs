use bevy_ecs::prelude::*;

#[derive(Resource, Eq, PartialEq)]
pub enum PauseState {
    Paused,
    Unpaused,
}

impl PauseState {
    pub fn init(mut commands: Commands) {
        commands.insert_resource(PauseState::Unpaused);
    }
}