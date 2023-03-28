use winit::window::Window;

use crate::cell::Cell;
use crate::gpu::Gpu;

pub const GRID_LINE_SIZE: usize = 48;
pub const GRID_COLUMN_SIZE: usize = 27;

pub struct State {
    gpu: Gpu,
    window: Window,
    _cells: [Cell; GRID_LINE_SIZE * GRID_COLUMN_SIZE],
}

impl State {
    pub async fn new(window: Window) -> Self {
        let mut cells = [Cell::default(); GRID_LINE_SIZE * GRID_COLUMN_SIZE];
        for i in 0..GRID_LINE_SIZE {
            for j in 0..GRID_COLUMN_SIZE {
                cells[i + GRID_LINE_SIZE * j].x = i as u32;
                cells[i + GRID_LINE_SIZE * j].y = j as u32;
            }
        }

        let gpu = Gpu::new(&window, &cells[..]).await;

        Self {
            gpu,
            window,
            _cells: cells,
        }
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
