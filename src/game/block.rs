use bevy_ecs::prelude::*;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Block {
    Air,
    Stone,
}

impl Block {
    pub fn is_empty(self) -> bool {
        self == Block::Air
    }
}
