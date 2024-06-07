use bevy_ecs::prelude::*;
use bevy_ecs::schedule::ScheduleLabel;

use crate::game::{camera, event, input, render};
use crate::render_state;

/*
    Execution order:

    Startup:
    - RenderInit
    - EventInit
    - Startup

    Update:
    - PreFrame
    - RenderUpdate
    - Update
    - EventUpdate
    - PostFrame
*/

#[derive(ScheduleLabel, Eq, PartialEq, Copy, Clone, Hash, Debug)]
pub struct StartupSchedule;

#[derive(ScheduleLabel, Eq, PartialEq, Copy, Clone, Hash, Debug)]
pub struct UpdateSchedule;

/// Special schedule where events are initialized.
#[derive(ScheduleLabel, Eq, PartialEq, Copy, Clone, Hash, Debug)]
pub struct EventInitSchedule;

/// Special schedule where events are updated and cleaned up.
#[derive(ScheduleLabel, Eq, PartialEq, Copy, Clone, Hash, Debug)]
pub struct EventUpdateSchedule;

/// Schedule to initialize render resources before execution begins.
#[derive(ScheduleLabel, Eq, PartialEq, Copy, Clone, Hash, Debug)]
pub struct RenderInitSchedule;

#[derive(ScheduleLabel, Eq, PartialEq, Copy, Clone, Hash, Debug)]
pub struct RenderUpdateSchedule;

/// Should only be used for maintenance before a frame starts.
#[derive(ScheduleLabel, Eq, PartialEq, Copy, Clone, Hash, Debug)]
pub struct PreFrameSchedule;

/// Should only be used for maintenance after a frame ends.
#[derive(ScheduleLabel, Eq, PartialEq, Copy, Clone, Hash, Debug)]
pub struct PostFrameSchedule;

pub fn create_startup_schedule() -> Schedule {
    let mut schedule = Schedule::new(StartupSchedule);

    schedule.add_systems((
        input::MouseMotion::init,
        input::KeyboardInput::init,
        input::MouseInput::init,
        camera::Camera::init,
        render_state::LastFrameInstant::insert,
    ));

    schedule
}

pub fn create_update_schedule() -> Schedule {
    let mut schedule = Schedule::new(UpdateSchedule);

    schedule.add_systems(camera::Camera::update);

    schedule
}

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
            camera::CameraUniform::init,
            camera::CameraBuffer::init,
            render::world::SolidTerrainRenderContext::init,
            render::post::FullscreenQuad::init,
            render::post::FinalRenderContext::init,
        )
            .chain(),
    );

    schedule
}

pub fn create_render_update_schedule() -> Schedule {
    let mut schedule = Schedule::new(RenderUpdateSchedule);

    schedule.add_systems(
        (
            camera::CameraUniform::update,
            camera::CameraBuffer::update,
            render::world::SolidTerrainRenderContext::update,
            render::post::FinalRenderContext::update,
        )
            .chain(),
    );

    schedule
}

pub fn create_pre_frame_schedule() -> Schedule {
    let mut schedule = Schedule::new(PreFrameSchedule);

    schedule.add_systems(input::receive_input_events);

    schedule
}

pub fn create_post_frame_schedule() -> Schedule {
    let mut schedule = Schedule::new(PostFrameSchedule);

    schedule.add_systems((input::KeyboardInput::update, input::MouseInput::update));

    schedule
}
