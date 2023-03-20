pub struct State {
    window: winit::window::Window,
}

impl State {
    pub fn window(&self) -> &winit::window::Window {
        &self.window
    }
}

pub async fn init(window: winit::window::Window) -> State {
    State { window }
}
