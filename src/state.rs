use winit::dpi::PhysicalPosition;
use winit::window::Window;

use crate::cell::{Cell, CellState, GRID_OFFSET, SCALE_FACTOR};
use crate::gpu::Gpu;

pub const GRID_LINE_SIZE: usize = 170;
pub const GRID_COLUMN_SIZE: usize = 95;
pub const TICK_PER_SEC: u32 = 15;

pub struct State {
    gpu: Gpu,
    window: Window,
    pub cells: [Cell; GRID_LINE_SIZE * GRID_COLUMN_SIZE],

    mouse_pos: PhysicalPosition<f64>,
    mouse_left_pressed: bool,
    mouse_right_pressed: bool,

    running: bool,
}

impl State {
    pub async fn new(window: Window) -> Self {
        let mut cells = [Cell::default(); GRID_LINE_SIZE * GRID_COLUMN_SIZE];
        for i in 0..GRID_LINE_SIZE {
            for j in 0..GRID_COLUMN_SIZE {
                let idx = Self::cell_idx(i as u32, j as u32);
                cells[idx].x = i as u32;
                cells[idx].y = j as u32;
            }
        }

        let gpu = Gpu::new(&window, &cells[..]).await;

        Self {
            gpu,
            window,
            cells,
            mouse_pos: PhysicalPosition::<f64>::new(0.0, 0.0),
            mouse_left_pressed: false,
            mouse_right_pressed: false,
            running: false,
        }
    }

    pub fn cell_idx(x: u32, y: u32) -> usize {
        (x + y * GRID_LINE_SIZE as u32) as usize
    }

    pub fn tick_interval() -> std::time::Duration {
        std::time::Duration::from_millis((1_000 / TICK_PER_SEC) as u64)
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    fn tick(&mut self) {
        macro_rules! valid_idx {
            ($cond:expr, $x:expr, $y:expr) => {
                if $cond {
                    Some(Self::cell_idx($x, $y))
                } else {
                    None
                }
            };
        }

        let cells = self.cells.map(|mut cell| {
            let x = cell.x;
            let y = cell.y;

            let has_left = x > 0;
            let has_top = y > 0;
            let has_right = x < GRID_LINE_SIZE as u32 - 1;
            let has_bottom = y < GRID_COLUMN_SIZE as u32 - 1;

            let neighbors = [
                valid_idx!(has_top, x, y - 1),
                valid_idx!(has_bottom, x, y + 1),
                valid_idx!(has_right, x + 1, y),
                valid_idx!(has_left, x - 1, y),
                valid_idx!(has_right && has_top, x + 1, y - 1),
                valid_idx!(has_right && has_bottom, x + 1, y + 1),
                valid_idx!(has_left && has_top, x - 1, y - 1),
                valid_idx!(has_left && has_bottom, x - 1, y + 1),
            ]
            .iter()
            .filter(|c| c.is_some() && self.cells[c.unwrap()].state.is_alive())
            .fold(0, |acc, _| acc + 1);

            match neighbors {
                2..=3 if cell.state.is_alive() => (),
                3 if !cell.state.is_alive() => cell.state = CellState::Alive,
                _ => cell.state = CellState::Dead,
            }

            cell
        });
        self.gpu.update_cells(&cells);
        self.cells = cells;
    }

    pub fn update(&mut self) {
        if self.running {
            let start = std::time::Instant::now();

            self.tick();

            let delta = start.elapsed();
            if delta < State::tick_interval() {
                std::thread::sleep(State::tick_interval() - delta);
            }
        }

        self.gpu.render();
    }

    pub fn cell_index_from_pos(pos: &PhysicalPosition<f64>) -> usize {
        let i = pos.x - (GRID_OFFSET as f64) - (SCALE_FACTOR as f64 / 2.0);
        let j = pos.y - (GRID_OFFSET as f64) - (SCALE_FACTOR as f64 / 2.0);

        let i = (i / crate::cell::GS).floor() as usize;
        let j = (j / crate::cell::GS).floor() as usize;

        Self::cell_idx(i as u32, j as u32)
    }

    pub fn input(&mut self, event: &winit::event::WindowEvent) {
        use winit::event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent};

        if !self.running && self.mouse_left_pressed || self.mouse_right_pressed {
            let idx = Self::cell_index_from_pos(&self.mouse_pos);
            if idx < self.cells.len() {
                if self.mouse_left_pressed {
                    self.cells[idx].state = CellState::Alive;
                } else if self.mouse_right_pressed {
                    self.cells[idx].state = CellState::Dead;
                }
                self.gpu.update_cells(&self.cells);
            }
        }

        match event {
            WindowEvent::CursorMoved { position, .. } => self.mouse_pos = *position,
            WindowEvent::MouseInput { state, button, .. } => match button {
                winit::event::MouseButton::Left => match state {
                    ElementState::Pressed => self.mouse_left_pressed = true,
                    ElementState::Released => self.mouse_left_pressed = false,
                },
                winit::event::MouseButton::Right => match state {
                    ElementState::Pressed => self.mouse_right_pressed = true,
                    ElementState::Released => self.mouse_right_pressed = false,
                },
                _ => (),
            },
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode,
                        ..
                    },
                ..
            } => match virtual_keycode {
                Some(k) if *k == VirtualKeyCode::R => {
                    for cell in self.cells.iter_mut() {
                        cell.state = CellState::Dead;
                    }
                    self.gpu.update_cells(&self.cells);
                }
                Some(k) if *k == VirtualKeyCode::S && !self.running => self.running = true,
                Some(k) if *k == VirtualKeyCode::P && self.running => self.running = false,
                _ => (),
            },
            _ => (),
        }
    }
}

macro_rules! alive_state_at {
    ($self:expr => $x:expr, $y:expr) => {
        $self.cells[Self::cell_idx($x, $y)].state = CellState::Alive;
    };
}

// Default Patterns
impl State {
    pub fn gosper_glider_gun(&mut self) {
        // Left Cube
        alive_state_at!(self => 1, 5);
        alive_state_at!(self => 2, 5);
        alive_state_at!(self => 1, 6);
        alive_state_at!(self => 2, 6);

        // Left "C" like structure
        alive_state_at!(self => 11, 5);
        alive_state_at!(self => 11, 6);
        alive_state_at!(self => 11, 7);
        alive_state_at!(self => 12, 4);
        alive_state_at!(self => 13, 3);
        alive_state_at!(self => 14, 3);
        alive_state_at!(self => 12, 8);
        alive_state_at!(self => 13, 9);
        alive_state_at!(self => 14, 9);

        alive_state_at!(self => 15, 6);

        alive_state_at!(self => 16, 4);
        alive_state_at!(self => 17, 5);
        alive_state_at!(self => 17, 6);
        alive_state_at!(self => 17, 7);
        alive_state_at!(self => 18, 6);
        alive_state_at!(self => 16, 8);

        alive_state_at!(self => 21, 5);
        alive_state_at!(self => 21, 4);
        alive_state_at!(self => 21, 3);
        alive_state_at!(self => 22, 5);
        alive_state_at!(self => 22, 4);
        alive_state_at!(self => 22, 3);
        alive_state_at!(self => 23, 2);
        alive_state_at!(self => 23, 6);

        alive_state_at!(self => 25, 1);
        alive_state_at!(self => 25, 2);
        alive_state_at!(self => 25, 6);
        alive_state_at!(self => 25, 7);

        alive_state_at!(self => 35, 3);
        alive_state_at!(self => 35, 4);
        alive_state_at!(self => 36, 3);
        alive_state_at!(self => 36, 4);

        self.gpu.update_cells(&self.cells);
    }

    pub fn blinkers(&mut self) {
        alive_state_at!(self => 7, 26);
        alive_state_at!(self => 7, 27);
        alive_state_at!(self => 7, 28);
        alive_state_at!(self => 6, 27);
        alive_state_at!(self => 8, 27);

        alive_state_at!(self => 14, 40);
        alive_state_at!(self => 14, 41);
        alive_state_at!(self => 14, 42);
        alive_state_at!(self => 13, 41);
        alive_state_at!(self => 15, 41);

        alive_state_at!(self => 24, 28);
        alive_state_at!(self => 24, 29);
        alive_state_at!(self => 24, 30);
        alive_state_at!(self => 23, 29);
        alive_state_at!(self => 25, 29);

        alive_state_at!(self => 29, 45);
        alive_state_at!(self => 29, 46);
        alive_state_at!(self => 29, 47);
        alive_state_at!(self => 28, 46);
        alive_state_at!(self => 30, 46);

        alive_state_at!(self => 19, 58);
        alive_state_at!(self => 19, 59);
        alive_state_at!(self => 19, 60);
        alive_state_at!(self => 18, 59);
        alive_state_at!(self => 20, 59);

        alive_state_at!(self => 7, 66);
        alive_state_at!(self => 7, 67);
        alive_state_at!(self => 7, 68);
        alive_state_at!(self => 6, 67);
        alive_state_at!(self => 8, 67);

        alive_state_at!(self => 5, 51);
        alive_state_at!(self => 5, 52);
        alive_state_at!(self => 5, 53);
        alive_state_at!(self => 4, 52);
        alive_state_at!(self => 6, 52);

        alive_state_at!(self => 30, 65);
        alive_state_at!(self => 30, 66);
        alive_state_at!(self => 30, 67);
        alive_state_at!(self => 29, 66);
        alive_state_at!(self => 31, 66);

        alive_state_at!(self => 18, 73);
        alive_state_at!(self => 18, 74);
        alive_state_at!(self => 18, 75);
        alive_state_at!(self => 17, 74);
        alive_state_at!(self => 19, 74);

        alive_state_at!(self => 37, 77);
        alive_state_at!(self => 37, 78);
        alive_state_at!(self => 37, 79);
        alive_state_at!(self => 36, 78);
        alive_state_at!(self => 38, 78);

        alive_state_at!(self => 26, 83);
        alive_state_at!(self => 26, 84);
        alive_state_at!(self => 26, 85);
        alive_state_at!(self => 25, 84);
        alive_state_at!(self => 27, 84);

        alive_state_at!(self => 7, 84);
        alive_state_at!(self => 7, 85);
        alive_state_at!(self => 7, 86);
        alive_state_at!(self => 6, 85);
        alive_state_at!(self => 8, 85);

        alive_state_at!(self => 42, 57);
        alive_state_at!(self => 43, 57);
        alive_state_at!(self => 44, 57);
        alive_state_at!(self => 43, 56);
        alive_state_at!(self => 43, 58);

        self.gpu.update_cells(&self.cells);
    }

    pub fn pulsars(&mut self) {
        alive_state_at!(self => 78, 14);
        alive_state_at!(self => 78, 15);
        alive_state_at!(self => 78, 16);
        alive_state_at!(self => 77, 17);
        alive_state_at!(self => 76, 17);
        alive_state_at!(self => 75, 17);
        alive_state_at!(self => 77, 12);
        alive_state_at!(self => 76, 12);
        alive_state_at!(self => 75, 12);
        alive_state_at!(self => 73, 16);
        alive_state_at!(self => 73, 15);
        alive_state_at!(self => 73, 14);

        alive_state_at!(self => 80, 16);
        alive_state_at!(self => 80, 15);
        alive_state_at!(self => 80, 14);
        alive_state_at!(self => 75, 19);
        alive_state_at!(self => 76, 19);
        alive_state_at!(self => 77, 19);
        alive_state_at!(self => 78, 20);
        alive_state_at!(self => 78, 21);
        alive_state_at!(self => 78, 22);
        alive_state_at!(self => 81, 17);
        alive_state_at!(self => 82, 17);
        alive_state_at!(self => 83, 17);

        alive_state_at!(self => 81, 19);
        alive_state_at!(self => 82, 19);
        alive_state_at!(self => 83, 19);
        alive_state_at!(self => 80, 20);
        alive_state_at!(self => 80, 21);
        alive_state_at!(self => 80, 22);
        alive_state_at!(self => 77, 24);
        alive_state_at!(self => 76, 24);
        alive_state_at!(self => 75, 24);
        alive_state_at!(self => 73, 20);
        alive_state_at!(self => 73, 21);
        alive_state_at!(self => 73, 22);

        alive_state_at!(self => 81, 24);
        alive_state_at!(self => 82, 24);
        alive_state_at!(self => 83, 24);
        alive_state_at!(self => 85, 20);
        alive_state_at!(self => 85, 21);
        alive_state_at!(self => 85, 22);
        alive_state_at!(self => 85, 16);
        alive_state_at!(self => 85, 15);
        alive_state_at!(self => 85, 14);
        alive_state_at!(self => 81, 12);
        alive_state_at!(self => 82, 12);
        alive_state_at!(self => 83, 12);

        self.gpu.update_cells(&self.cells);
    }
}

pub fn init(window: Window) -> State {
    pollster::block_on(State::new(window))
}
