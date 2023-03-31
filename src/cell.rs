use crate::math::transpose;

#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum CellState {
    Dead = 0,
    Alive,
}

impl CellState {
    pub fn is_alive(&self) -> bool {
        match self {
            CellState::Alive => true,
            _ => false,
        }
    }
}

impl std::ops::Not for CellState {
    type Output = Self;
    fn not(self) -> Self::Output {
        match self {
            CellState::Alive => CellState::Dead,
            CellState::Dead => CellState::Alive,
        }
    }
}

pub const SCALE_FACTOR: f32 = 10.0;
pub const GRID_OFFSET: f32 = 5.0;
pub const GS: f64 = SCALE_FACTOR as f64 + GRID_OFFSET as f64;

#[derive(Clone, Copy, Debug)]
pub struct Cell {
    pub x: u32,
    pub y: u32,
    pub state: CellState,
}

impl std::default::Default for Cell {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            state: CellState::Dead,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CellInstance {
    pub model: [f32; 16],
    pub state: u32,
}

impl std::convert::From<&Cell> for CellInstance {
    fn from(value: &Cell) -> Self {
        #[rustfmt::skip]
        let model = [
            SCALE_FACTOR, 0.0,          0.0, ((SCALE_FACTOR+GRID_OFFSET) * value.x as f32) + SCALE_FACTOR / 2.0 + 5.0, // 5.0 => little offset to center the grid
            0.0,          SCALE_FACTOR, 0.0, ((SCALE_FACTOR+GRID_OFFSET) * value.y as f32) + SCALE_FACTOR / 2.0,
            0.0,          0.0,          1.0, 0.0,
            0.0,          0.0,          0.0, 1.0,
        ];

        Self {
            model: transpose(model),
            state: value.state as u32,
        }
    }
}
