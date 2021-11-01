use oblivion::{GraphicsContext, Render};
use winit::{
    dpi::LogicalSize,
    event::{Event, VirtualKeyCode, WindowEvent},
};

pub trait Example {
    fn setup(ctx: &mut GraphicsContext) -> Self;
    fn update(&mut self, _ctx: &mut GraphicsContext) {}
    fn draw(&self, render: &mut Render);
}

pub fn run<E: Example + 'static>() {
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title("Oblivion")
        .with_inner_size(LogicalSize::new(1280.0, 720.0))
        .build(&event_loop)
        .expect("Unable to create window");

    let mut ctx = GraphicsContext::new(
        &window,
        [window.inner_size().width, window.inner_size().height],
        true,
    )
    .unwrap();

    let mut ex = E::setup(&mut ctx);

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
                window.set_title(&format!("Oblivion (FPS: {})", frame_count));
                prev = std::time::Instant::now();
                frame_count = 0;
            }
            ex.update(&mut ctx);
            let mut render = Render::new();
            ex.draw(&mut render);
            ctx.submit_render(render).unwrap();
            frame_count += 1;
        }
        Event::LoopDestroyed => { /* on quit */ }
        _ => {}
    });
}

#[allow(dead_code)]
fn main() {}
