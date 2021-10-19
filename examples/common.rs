use oblivion::{GraphicsContext, Render};
use winit::event::{Event, VirtualKeyCode, WindowEvent};

pub trait Example {
    fn setup(ctx: &mut GraphicsContext) -> Self;
    fn draw(&self, render: &mut Render);
}

pub fn run<E: Example + 'static>() {
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::Window::new(&event_loop).expect("Unable to create window");

    let mut ctx = GraphicsContext::new(
        &window,
        window.inner_size().width,
        window.inner_size().height,
        true,
    );

    let ex = E::setup(&mut ctx);

    let mut prev = std::time::Instant::now();
    let mut frame_count = 0;

    event_loop.run(move |event, _window, control_flow| match event {
        Event::WindowEvent {
            window_id: _,
            event,
        } => match event {
            WindowEvent::KeyboardInput {
                device_id: _,
                input,
                is_synthetic: _,
            } => {
                if let Some(key_code) = input.virtual_keycode {
                    match key_code {
                        VirtualKeyCode::Escape => {
                            *control_flow = winit::event_loop::ControlFlow::Exit
                        }
                        _ => {}
                    }
                }
            }
            WindowEvent::CloseRequested => *control_flow = winit::event_loop::ControlFlow::Exit,
            _ => {}
        },
        Event::MainEventsCleared => {
            if prev.elapsed().as_secs_f32() >= 1.0 {
                println!("FPS: {}", frame_count);
                prev = std::time::Instant::now();
                frame_count = 0;
            }
            let mut render = Render::new();
            ex.draw(&mut render);
            ctx.submit_render(render);
            frame_count += 1;
        }
        Event::LoopDestroyed => { /* on quit */ }
        _ => {}
    });
}

#[allow(dead_code)]
fn main() {}
