mod schedules;

use crate::render_state::RenderState;
use bevy_ecs::prelude::{Schedule, World};
use std::sync::Arc;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

fn init_window() -> (EventLoop<()>, Arc<Window>) {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let window = Arc::new(window);

    (event_loop, window)
}

fn create_render_init_schedule() -> Schedule {
    Schedule::new(schedules::RenderInitSchedule)
}

fn create_render_update_schedule() -> Schedule {
    let mut schedule = Schedule::new(schedules::RenderUpdateSchedule);

    schedule.add_systems(|| log::info!("Running render update schedule"));

    schedule
}

pub async fn run() {
    let (event_loop, window) = init_window();

    let mut world = World::new();
    let mut render_update_schedule = create_render_update_schedule();
    let mut render_init_schedule = create_render_init_schedule();

    let render_state = RenderState::new(window.clone()).await;
    world.insert_resource(render_state);

    // Run systems that run on startup, after initializing render state
    render_init_schedule.run(&mut world);

    event_loop
        .run(move |event, control_flow| match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == world.resource::<RenderState>().window().id() => match event {
                WindowEvent::CloseRequested => {
                    control_flow.exit();
                }
                WindowEvent::Resized(size) => {
                    // Update the render state with the new size
                    world.resource_mut::<RenderState>().resize(*size);
                }
                WindowEvent::RedrawRequested => {
                    // We want another frame after this one
                    world.resource::<RenderState>().window.request_redraw();

                    // Run every system in the render update schedule
                    render_update_schedule.run(&mut world);

                    // match render_state.render() {
                    //     Ok(_) => {}
                    //     Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                    //         let size = render_state.size;
                    //         render_state.resize(size);
                    //     }
                    //     Err(wgpu::SurfaceError::Timeout) => {
                    //         log::warn!("Surface timeout");
                    //     }
                    //     Err(wgpu::SurfaceError::OutOfMemory) => {
                    //         log::error!("Out of memory, exiting");
                    //         control_flow.exit();
                    //     }
                    // }
                }
                _ => {}
            },
            _ => {}
        })
        .unwrap();
}
