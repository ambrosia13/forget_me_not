pub mod block;
pub mod chunk;
pub mod event;
pub mod render;
pub mod schedule;
pub mod texture;
pub mod vertex;

use crate::render_state::{
    CommandEncoderResource, RenderState, ResizeEvent, SurfaceTextureResource,
};
use bevy_ecs::event::Events;
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

fn init_world() -> World {
    let mut world = World::new();
    world.insert_resource(Events::<ResizeEvent>::default());

    world
}

pub async fn run() {
    let (event_loop, window) = init_window();
    let mut world = init_world();

    let mut render_update_schedule = schedule::create_render_update_schedule();
    let mut render_init_schedule = schedule::create_render_init_schedule();

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

                    // Send an event so all other systems can resize accordingly
                    world.send_event(ResizeEvent(*size));
                }
                WindowEvent::RedrawRequested => {
                    // We want another frame after this one
                    world.resource::<RenderState>().window.request_redraw();

                    // Initialize data needed for the next frame, handle possible errors
                    match crate::render_state::begin_frame(&mut world) {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                            // Reconfigure the surface
                            let mut render_state = world.resource_mut::<RenderState>();

                            let size = render_state.size;
                            render_state.resize(size);
                        }
                        Err(wgpu::SurfaceError::Timeout) => {
                            log::warn!("Surface timeout");
                        }
                        Err(wgpu::SurfaceError::OutOfMemory) => {
                            log::error!("Out of memory, exiting");
                            control_flow.exit();
                        }
                    }

                    // Run every system in the render update schedule
                    render_update_schedule.run(&mut world);

                    // Clean up data from the frame we just finished
                    crate::render_state::finish_frame(&mut world);
                }
                _ => {}
            },
            _ => {}
        })
        .unwrap();
}
