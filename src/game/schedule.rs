use bevy_ecs::prelude::*;
use bevy_ecs::schedule::ScheduleLabel;

use crate::game::{event, input, render};
use crate::render_state;

#[derive(ScheduleLabel, Eq, PartialEq, Copy, Clone, Hash, Debug)]
pub struct EventInitSchedule;

#[derive(ScheduleLabel, Eq, PartialEq, Copy, Clone, Hash, Debug)]
pub struct EventUpdateSchedule;

#[derive(ScheduleLabel, Eq, PartialEq, Copy, Clone, Hash, Debug)]
pub struct RenderInitSchedule;

#[derive(ScheduleLabel, Eq, PartialEq, Copy, Clone, Hash, Debug)]
pub struct RenderUpdateSchedule;

pub fn create_event_init_schedule() -> Schedule {
    let mut schedule = Schedule::new(EventInitSchedule);

    schedule.add_systems((
        event::init_event::<input::KeyboardInputEvent>,
        event::init_event::<input::MouseInputEvent>,
        event::init_event::<render_state::WindowResizeEvent>,
    ));

    schedule
}

pub fn create_event_update_schedule() -> Schedule {
    let mut schedule = Schedule::new(EventUpdateSchedule);

    schedule.add_systems((
        event::clear_events::<input::KeyboardInputEvent>,
        event::clear_events::<input::MouseInputEvent>,
        event::clear_events::<render_state::WindowResizeEvent>,
    ));

    schedule
}

pub fn create_render_init_schedule() -> Schedule {
    let mut schedule = Schedule::new(RenderInitSchedule);

    schedule.add_systems(
        (
            render::world::init_solid_terrain_renderer,
            render::post::init_post_renderer,
        )
            .chain(),
    );

    schedule
}

pub fn create_render_update_schedule() -> Schedule {
    let mut schedule = Schedule::new(RenderUpdateSchedule);

    schedule.add_systems(
        (
            render::world::draw_solid_terrain,
            render::post::draw_post_passes,
        )
            .chain(),
    );

    schedule
}
