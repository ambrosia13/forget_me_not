pub mod camera;
pub mod command;
pub mod event;
pub mod input;
pub mod material;
pub mod object;
pub mod render;
pub mod schedule;
pub mod vertex;

use crate::game::input::MouseMotion;
use crate::render_state::{LastFrameInstant, RenderState, WindowResizeEvent};
use bevy_ecs::prelude::World;
use command::{GameCommand, GameCommands, GameCommandsResource};
use glam::Vec3;
use material::Material;
use std::sync::Arc;
use winit::event::{DeviceEvent, Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::{CursorGrabMode, Window, WindowBuilder};

fn init_window() -> (EventLoop<()>, Arc<Window>) {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    // Set the cursor grab mode to one that is supported by the system.
    window
        .set_cursor_grab(CursorGrabMode::Confined)
        .or_else(|_| window.set_cursor_grab(CursorGrabMode::Locked))
        .unwrap();
    window.set_cursor_visible(false);

    window.set_maximized(true);

    let window = Arc::new(window);

    (event_loop, window)
}

fn init_world() -> World {
    World::new()
}

fn init_commands() -> Arc<GameCommands> {
    let game_commands = Arc::new(GameCommands::new());
    let game_commands_copy = game_commands.clone();

    std::thread::spawn(move || {
        let mut material = Some(Material {
            ty: material::MaterialType::Lambertian,
            albedo: Vec3::ONE,
            emission: Vec3::ZERO,
            roughness: 0.0,
        });

        loop {
            let mut input = String::new();

            std::io::stdin().read_line(&mut input).unwrap();

            let cmd = GameCommand::parse(&input, &mut material);

            game_commands.push(match cmd {
                Some(cmd) => cmd,
                None => {
                    println!("Unrecognized command {}", input);
                    continue;
                }
            });

            std::thread::sleep(std::time::Duration::from_secs_f32(0.1));
        }
    });

    game_commands_copy
}

pub async fn run() {
    let (event_loop, window) = init_window();
    let mut world = init_world();
    let game_commands = init_commands();

    GameCommandsResource::init(game_commands, &mut world);

    let mut startup_schedule = schedule::create_startup_schedule();
    let mut update_schedule = schedule::create_update_schedule();
    let mut event_init_schedule = schedule::create_event_init_schedule();
    let mut event_update_schedule = schedule::create_event_update_schedule();
    let mut render_init_schedule = schedule::create_render_init_schedule();
    let mut render_update_schedule = schedule::create_render_update_schedule();
    let mut pre_frame_schedule = schedule::create_pre_frame_schedule();
    let mut post_frame_schedule = schedule::create_post_frame_schedule();

    let render_state = RenderState::new(window.clone()).await;
    world.insert_resource(render_state);

    // Run systems that run on startup, after initializing render state
    render_init_schedule.run(&mut world);

    // Initialize all event types
    event_init_schedule.run(&mut world);

    // Run all startup systems
    startup_schedule.run(&mut world);

    event_loop
        .run(move |event, control_flow| match event {
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => {
                let mouse_motion = MouseMotion {
                    delta_x: delta.0,
                    delta_y: delta.1,
                };

                world.insert_resource(mouse_motion);
            }
            Event::WindowEvent { event, window_id }
                if window_id == world.resource::<RenderState>().window().id() =>
            {
                match event {
                    // Redirect keyboard input events to the ECS
                    WindowEvent::KeyboardInput {
                        device_id,
                        event,
                        is_synthetic,
                    } => {
                        world.send_event(input::KeyboardInputEvent {
                            device_id,
                            event,
                            is_synthetic,
                        });
                    }
                    // Redirect mouse input events to the ECS
                    WindowEvent::MouseInput {
                        device_id,
                        state,
                        button,
                    } => {
                        world.send_event(input::MouseInputEvent {
                            device_id,
                            state,
                            button,
                        });
                    }
                    WindowEvent::CloseRequested => {
                        control_flow.exit();
                    }
                    WindowEvent::Resized(size) => {
                        // Update the render state with the new size
                        world.resource_mut::<RenderState>().resize(size);

                        // Send an event so all other systems can resize accordingly
                        world.send_event(WindowResizeEvent(size));
                    }
                    WindowEvent::RedrawRequested => {
                        // We want another frame after this one
                        world.resource::<RenderState>().window.request_redraw();

                        // Run pre-frame systems that need to run before any logic
                        // for the current frame is run.
                        pre_frame_schedule.run(&mut world);

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

                        // Run all update systems
                        update_schedule.run(&mut world);

                        // Clear events so they don't pile up
                        event_update_schedule.run(&mut world);

                        // Clean up data from the frame we just finished
                        crate::render_state::finish_frame(&mut world);

                        // Run post-frame systems, that expect that all work for this frame has finished,
                        // so we can clean up some state.
                        post_frame_schedule.run(&mut world);

                        // Remove mouse motion because the frame is done
                        world.remove_resource::<MouseMotion>();

                        // Save the time that this frame happened
                        world.insert_resource(LastFrameInstant(std::time::Instant::now()));
                    }
                    _ => {}
                }
            }
            _ => {}
        })
        .unwrap();
}
