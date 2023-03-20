mod gpu;
mod state;

use state::State;

const WIN_WIDTH: u32 = 1280;
const WIN_HEIGHT: u32 = 720;

fn main() {
    let (event_loop, window) = setup_window();
    let state = state::init(window);

    run(event_loop, state);
}

fn setup_window() -> (winit::event_loop::EventLoop<()>, winit::window::Window) {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .unwrap();

    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_inner_size(winit::dpi::LogicalSize::new(WIN_WIDTH, WIN_HEIGHT))
        .with_resizable(false)
        .build(&event_loop)
        .unwrap();

    log::info!("winit window initialized");

    (event_loop, window)
}

fn run(event_loop: winit::event_loop::EventLoop<()>, mut state: State) {
    use winit::event::{Event, WindowEvent};
    use winit::event_loop::ControlFlow;

    fn handle_win_event(event: &WindowEvent, _state: &mut State, control_flow: &mut ControlFlow) {
        match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            _ => (),
        }
    }

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { ref event, .. } => handle_win_event(event, &mut state, control_flow),
        Event::RedrawRequested(_) => state.update(),
        Event::MainEventsCleared => state.window().request_redraw(),
        _ => (),
    })
}
