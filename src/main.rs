mod cell;
mod gpu;
mod math;
mod state;

use state::State;
use winit::event::{Event, WindowEvent};
use winit::event_loop::ControlFlow;

const WIN_WIDTH: u32 = 1280;
const WIN_HEIGHT: u32 = 720;

fn setup_window() -> (winit::event_loop::EventLoop<()>, winit::window::Window) {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .with_module_level("wgpu_core", log::LevelFilter::Warn)
        .with_module_level("wgpu_hal", log::LevelFilter::Warn)
        .init()
        .unwrap();

    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title("Conway's Game of Life")
        .with_inner_size(winit::dpi::PhysicalSize::new(WIN_WIDTH, WIN_HEIGHT))
        .with_resizable(false)
        .build(&event_loop)
        .unwrap();

    log::info!("winit window initialized");

    (event_loop, window)
}

fn handle_win_event(event: &WindowEvent, state: &mut State, control_flow: &mut ControlFlow) {
    match event {
        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => state.resize(**new_inner_size),
        _ => state.input(event),
    }
}

fn run(event_loop: winit::event_loop::EventLoop<()>, mut state: State) {
    state.gosper_glider_gun();
    state.blinkers();
    state.pulsars();

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { ref event, .. } => handle_win_event(event, &mut state, control_flow),
        Event::RedrawRequested(_) => state.update(),
        Event::MainEventsCleared => state.window().request_redraw(),
        _ => (),
    })
}

fn main() {
    let (event_loop, window) = setup_window();
    let state = state::init(window);

    run(event_loop, state);
}
