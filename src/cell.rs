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

impl CellInstance {
    pub fn from_cell(cell: &Cell, scale_factor: f32, offset: f32) -> Self {
        #[rustfmt::skip]
        let model = [
            scale_factor, 0.0,          0.0, ((scale_factor + offset) * cell.x as f32) + scale_factor / 2.0 + 6.0, // 4.0 => little offset to center the grid
            0.0,          scale_factor, 0.0, ((scale_factor + offset) * cell.y as f32) + scale_factor / 2.0 + 6.0,
            0.0,          0.0,          1.0, 0.0,
            0.0,          0.0,          0.0, 1.0,
        ];

        Self {
            model: transpose(model),
            state: cell.state as u32,
        }
    }
}
