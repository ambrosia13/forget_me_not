use std::marker::PhantomData;

use bevy_ecs::event::Event;

pub mod post;
pub mod world;

#[derive(Event)]
pub struct ReloadRenderContextEvent;
