pub fn transpose(m: [f32; 16]) -> [f32; 16] {
    #[rustfmt::skip]
    let m = [
        m[0], m[4], m[8],  m[12],
        m[1], m[5], m[9],  m[13],
        m[2], m[6], m[10], m[14],
        m[3], m[7], m[11], m[15],
    ];
    m
}

pub fn ortho_projection(dimensions: (u32, u32)) -> [f32; 16] {
    #[rustfmt::skip]
    let ortho_projection: [f32; 16] = [
        2.0 / dimensions.0 as f32, 0.0,                       0.0, -1.0,
        0.0,                      -2.0 / dimensions.1 as f32, 0.0,  1.0,
        0.0,                       0.0,                       0.0,  1.0,
        0.0,                       0.0,                       0.0,  1.0,
    ];
    transpose(ortho_projection)
}
