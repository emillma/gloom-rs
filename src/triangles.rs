#[allow(dead_code)]
use std::f32::consts::PI;
use std::vec;

fn deg2rad(i: f32) -> f32 {
    return (i % 360.) * PI / 180.;
}
pub fn get_triangles(n: u32) -> Vec<f32> {
    let mut vertecies: Vec<f32> = Vec::new();
    let x: f32 = 0.;
    let y: f32 = 0.;
    let z: f32 = 0.;
    for i in 0..n {
        let i_frac = (i as f32) / ((n - 1) as f32);
        let size = 1.;
        for j in 0..3 {
            let j_frac = (j as f32) / 2.;
            vertecies.extend_from_slice(&vec![
                x + deg2rad(120. * (j as f32) + (i as f32) * (120. / (n as f32))).cos() * size,
                y + deg2rad(120. * (j as f32) + (i as f32) * (120. / (n as f32))).sin() * size,
                z - 0.1 * ((j == 0) as i32 as f32),
            ]);
            vertecies.extend_from_slice(&vec![
                ((i == 0) as i32 as f32) * ((j == 0) as i32 as f32),
                ((i == 1) as i32 as f32) * ((j == 0) as i32 as f32),
                ((i == 2) as i32 as f32) * ((j == 0) as i32 as f32),
            ]);
        }
    }
    println!("{:?}", vertecies);
    return vertecies;
}
