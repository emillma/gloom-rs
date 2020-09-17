#[allow(dead_code)]
pub fn get_triangles(n: u32) -> Vec<f32> {
    let mut vertices: Vec<f32> = Vec::new();
    for i in 1..n + 1 {
        let i2 = i as f32;
        let size = 0.1;
        let offset = 0.9 - size * i2 * (i2 + 1.) / 2.;
        let triangle = vec![
            -size * i2,
            offset,
            0.,
            size * i2,
            offset,
            0.,
            0.,
            offset + size * i2,
            0.,
        ];
        vertices.extend_from_slice(&triangle);
    }
    return vertices;
}
