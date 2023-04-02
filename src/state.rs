use egui_winit::winit;
use winit::dpi::PhysicalPosition;
use winit::window::Window;

use crate::cell::{Cell, CellState};
use crate::gpu::Gpu;
use crate::gui::GuiCtx;

// TODO: handle automatically
pub const GRID_LINE_SIZE: usize = 100;
pub const GRID_COLUMN_SIZE: usize = 100;
pub const TICK_PER_SEC: u32 = 12;

pub const INITIAL_SCALE_FACTOR: f32 = 10.0;
pub const INITIAL_OFFSET: f32 = 5.0;

pub struct State {
    gpu: Gpu,
    window: Window,
    ctx: GuiCtx,

    pub cells: [Cell; GRID_LINE_SIZE * GRID_COLUMN_SIZE],

    mouse_pos: PhysicalPosition<f64>,
    mouse_left_pressed: bool,
    mouse_right_pressed: bool,

    gui_state: crate::gui::State,
}

impl State {
    pub async fn new(window: Window, event_loop: &winit::event_loop::EventLoop<()>) -> Self {
        let mut cells = [Cell::default(); GRID_LINE_SIZE * GRID_COLUMN_SIZE];
        for i in 0..GRID_LINE_SIZE {
            for j in 0..GRID_COLUMN_SIZE {
                let idx = Self::cell_idx(i as u32, j as u32);
                cells[idx].x = i as u32;
                cells[idx].y = j as u32;
            }
        }

        let gpu = Gpu::new(&window, &cells[..], INITIAL_SCALE_FACTOR, INITIAL_OFFSET).await;
        let ctx = GuiCtx::new(event_loop, gpu.device(), gpu.surface_config(), &window);

        log::info!("state initialized");

        Self {
            gpu,
            window,
            ctx,
            cells,
            mouse_pos: PhysicalPosition::<f64>::new(0.0, 0.0),
            mouse_left_pressed: false,
            mouse_right_pressed: false,

            gui_state: crate::gui::State {
                running: false,
                clear_color_r: 0.01,
                clear_color_g: 0.01,
                clear_color_b: 0.02,
                cell_scale_factor: INITIAL_SCALE_FACTOR,
                cell_offset: INITIAL_OFFSET,
            },
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

    pub fn resize(&mut self, dimensions: winit::dpi::PhysicalSize<u32>) {
        self.gpu.resize(dimensions);
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
        self.cells = cells;
    }

    pub fn update(&mut self) {
        self.gpu.update_cells(
            &self.cells,
            self.gui_state.cell_scale_factor,
            self.gui_state.cell_offset,
        );

        if self.gui_state.running {
            let start = std::time::Instant::now();

            self.tick();

            let delta = start.elapsed();
            if delta < State::tick_interval() {
                std::thread::sleep(State::tick_interval() - delta);
            }
        }

        let output = self.ctx.build_ui(&mut self.gui_state, &self.window);

        let clear_color = wgpu::Color {
            a: 1.0,
            r: self.gui_state.clear_color_r,
            g: self.gui_state.clear_color_g,
            b: self.gui_state.clear_color_b,
        };

        self.gpu.render(&mut self.ctx, output, clear_color);
    }

    pub fn cell_index_from_pos(
        pos: &PhysicalPosition<f64>,
        scale_factor: f32,
        offset: f32,
    ) -> usize {
        let gs = scale_factor as f64 + offset as f64;
        let i = pos.x - (offset as f64) - (scale_factor as f64 / 2.0);
        let j = pos.y - (offset as f64) - (scale_factor as f64 / 2.0);

        let i = (i / gs).floor() as usize;
        let j = (j / gs).floor() as usize;

        Self::cell_idx(i as u32, j as u32)
    }

    // TODO: better input handling
    pub fn input(&mut self, event: &winit::event::WindowEvent) {
        use winit::event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent};

        if self.ctx.on_event(event) {
            return;
        }

        if !self.gui_state.running && self.mouse_left_pressed || self.mouse_right_pressed {
            let idx = Self::cell_index_from_pos(
                &self.mouse_pos,
                self.gui_state.cell_scale_factor,
                self.gui_state.cell_offset,
            );
            if idx < self.cells.len() {
                if self.mouse_left_pressed {
                    self.cells[idx].state = CellState::Alive;
                } else if self.mouse_right_pressed {
                    self.cells[idx].state = CellState::Dead;
                }
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
                }
                Some(k) if *k == VirtualKeyCode::S && !self.gui_state.running => {
                    self.gui_state.running = true
                }
                Some(k) if *k == VirtualKeyCode::P && self.gui_state.running => {
                    self.gui_state.running = false
                }
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
    }

    pub fn pulsars(&mut self) {
        alive_state_at!(self => 68, 14);
        alive_state_at!(self => 68, 15);
        alive_state_at!(self => 68, 16);
        alive_state_at!(self => 67, 17);
        alive_state_at!(self => 66, 17);
        alive_state_at!(self => 65, 17);
        alive_state_at!(self => 67, 12);
        alive_state_at!(self => 66, 12);
        alive_state_at!(self => 65, 12);
        alive_state_at!(self => 63, 16);
        alive_state_at!(self => 63, 15);
        alive_state_at!(self => 63, 14);

        alive_state_at!(self => 70, 16);
        alive_state_at!(self => 70, 15);
        alive_state_at!(self => 70, 14);
        alive_state_at!(self => 65, 19);
        alive_state_at!(self => 66, 19);
        alive_state_at!(self => 67, 19);
        alive_state_at!(self => 68, 20);
        alive_state_at!(self => 68, 21);
        alive_state_at!(self => 68, 22);
        alive_state_at!(self => 71, 17);
        alive_state_at!(self => 72, 17);
        alive_state_at!(self => 73, 17);

        alive_state_at!(self => 71, 19);
        alive_state_at!(self => 72, 19);
        alive_state_at!(self => 73, 19);
        alive_state_at!(self => 70, 20);
        alive_state_at!(self => 70, 21);
        alive_state_at!(self => 70, 22);
        alive_state_at!(self => 67, 24);
        alive_state_at!(self => 66, 24);
        alive_state_at!(self => 65, 24);
        alive_state_at!(self => 63, 20);
        alive_state_at!(self => 63, 21);
        alive_state_at!(self => 63, 22);

        alive_state_at!(self => 71, 24);
        alive_state_at!(self => 72, 24);
        alive_state_at!(self => 73, 24);
        alive_state_at!(self => 75, 20);
        alive_state_at!(self => 75, 21);
        alive_state_at!(self => 75, 22);
        alive_state_at!(self => 75, 16);
        alive_state_at!(self => 75, 15);
        alive_state_at!(self => 75, 14);
        alive_state_at!(self => 71, 12);
        alive_state_at!(self => 72, 12);
        alive_state_at!(self => 73, 12);
    }
}

pub fn init(window: Window, event_loop: &winit::event_loop::EventLoop<()>) -> State {
    pollster::block_on(State::new(window, event_loop))
}
