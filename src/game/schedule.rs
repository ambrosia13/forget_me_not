use crate::game::{schedule, world_renderer};
use crate::render_state::{CommandEncoderResource, SurfaceTextureResource};
use bevy_ecs::prelude::Schedule;
use bevy_ecs::schedule::ScheduleLabel;

#[derive(ScheduleLabel, Eq, PartialEq, Copy, Clone, Hash, Debug)]
pub struct RenderInitSchedule;

#[derive(ScheduleLabel, Eq, PartialEq, Copy, Clone, Hash, Debug)]
pub struct RenderUpdateSchedule;

pub fn create_render_init_schedule() -> Schedule {
    let mut schedule = Schedule::new(RenderInitSchedule);

    schedule.add_systems(world_renderer::init_solid_terrain_renderer);

    schedule
}

pub fn create_render_update_schedule() -> Schedule {
    let mut schedule = Schedule::new(RenderUpdateSchedule);

    schedule.add_systems(world_renderer::draw_solid_terrain);

    schedule
}
