use winit::window::Window;

use crate::gpu::Gpu;

pub struct State {
    gpu: Gpu,
    window: Window,
}

impl State {
    pub async fn new(window: Window) -> Self {
        let gpu = Gpu::new(&window).await;
        Self { gpu, window }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn update(&mut self) {
        self.gpu.render();
    }
}

pub fn init(window: Window) -> State {
    pollster::block_on(State::new(window))
}
