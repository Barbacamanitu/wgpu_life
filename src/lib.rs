use winit::{
    event::*,
    event_loop::{EventLoop, ControlFlow},
    window::{Window, WindowBuilder},
};
use futures::executor::block_on;
mod renderer;
use renderer::Renderer;
pub struct Game {}

impl Game {
    pub fn run() {
    let event_loop = EventLoop::new();
    let window: Window = WindowBuilder::new()
        .build(&event_loop)
        .unwrap();

    let mut renderer = block_on(Renderer::new(&window));

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => if !renderer.input(event) { 
                match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput {
                        input,
                        ..
                    } => {
                        match input {
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            } => *control_flow = ControlFlow::Exit,
                            _ => {}
                        }
                    },                
                WindowEvent::Resized(physical_size) => {
                    renderer.resize(*physical_size);
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    // new_inner_size is &mut so we have to dereference it twice
                    renderer.resize(**new_inner_size);
                },
                _ => {}
            }
        },
        Event::RedrawRequested(_) => {
            renderer.update();
            renderer.render();
        }
        Event::MainEventsCleared => {
            // RedrawRequested will only trigger once, unless we manually
            // request it.
            window.request_redraw();
        }
            _ => {}
            
        }
    });
    }
}