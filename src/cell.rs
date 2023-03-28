use crate::math::transpose;

#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum CellState {
    Dead = 0,
    Alive,
}

const SCALE_FACTOR: f32 = 43.0;
const GRID_OFFSET: f32 = 10.0;

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
            state: CellState::Alive,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CellInstance {
    pub model: [f32; 16],
}

impl std::convert::From<&Cell> for CellInstance {
    fn from(value: &Cell) -> Self {
        #[rustfmt::skip]
        let model = [
            SCALE_FACTOR, 0.0,          0.0, ((SCALE_FACTOR+GRID_OFFSET) * value.x as f32) + SCALE_FACTOR / 2.0 + 14.0,
            0.0,          SCALE_FACTOR, 0.0, ((SCALE_FACTOR+GRID_OFFSET) * value.y as f32) + SCALE_FACTOR / 2.0 + 8.0,
            0.0,          0.0,          1.0, 0.0,
            0.0,          0.0,          0.0, 1.0,
        ];

        Self {
            model: transpose(model),
        }
    }
}
