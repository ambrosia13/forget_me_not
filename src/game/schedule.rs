use crate::game::{event, render, schedule};
use crate::render_state::{CommandEncoderResource, ResizeEvent, SurfaceTextureResource};
use bevy_ecs::prelude::*;
use bevy_ecs::schedule::ScheduleLabel;

#[derive(ScheduleLabel, Eq, PartialEq, Copy, Clone, Hash, Debug)]
pub struct RenderInitSchedule;

#[derive(ScheduleLabel, Eq, PartialEq, Copy, Clone, Hash, Debug)]
pub struct RenderUpdateSchedule;

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

    // To clean up events
    schedule.add_systems(event::cleanup_events::<ResizeEvent>);

    schedule
}
