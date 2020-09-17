#[allow(dead_code)]
use std::f32::consts::PI;
use std::vec;

fn deg2rad(i: f32) -> f32 {
    return (i % 360.) * PI / 180.;
}
pub fn get_triangles(n: u32) -> (Vec<f32>, Vec<f32>) {
    let mut vertecies: Vec<f32> = Vec::new();
    let mut colors: Vec<f32> = Vec::new();
    for i in 0..n {
        let n = n as f32;
        let i = i as f32;

        let x: f32 = 0. - deg2rad(i * (360. / n)).cos() * 0.3;
        let y: f32 = 0. - deg2rad(i * (360. / n)).sin() * 0.3;
        let z: f32 = 0.;
        let i_frac = i / (n - 1.);
        let size = 0.75;

        for j in 0..3 {
            let j = j as f32;
            let j_frac = j / 2.;

            vertecies.extend_from_slice(&vec![
                x + deg2rad(10. + 120. * j + i * (360. / n)).cos() * size,
                y + deg2rad(10. + 120. * j + i * (360. / n)).sin() * size,
                z - i_frac * 0.1,
            ]);
            colors.extend_from_slice(&vec![
                (0.5 * (i == 1.) as i32 as f32) + 0.3,
                (0.5 * (i == 2.) as i32 as f32) + 0.3,
                (0.5 * (i == 0.) as i32 as f32) + 0.3,
                0.5,
            ]);
        }
    }
    return (vertecies, colors);
}
