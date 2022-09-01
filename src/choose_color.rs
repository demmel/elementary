use bevy::{math::vec3, prelude::*};
use rand::prelude::*;

pub fn choose_colors(n: usize) -> Vec<Color> {
    let mut points = get_initial_points(n);

    if points.len() > 1 {
        let mut prev_score = 0.0;
        loop {
            tick_points(&mut points);
            let score = mean_minimum_distance(&points);
            let change = (score - prev_score).abs();
            if change < 1e-5 {
                break;
            }
            prev_score = score;
        }
    }

    points
        .into_iter()
        .map(|c| Color::rgb(c.x, c.y, c.z))
        .collect()
}

fn get_initial_points(n: usize) -> Vec<Vec3> {
    let mut points = Vec::with_capacity(n);
    let mut rng = thread_rng();

    for _ in 0..n {
        points.push(vec3(rng.gen(), rng.gen(), rng.gen()));
    }

    points
}

fn tick_points(points: &mut [Vec3]) {
    let closest: Vec<_> = points
        .iter()
        .enumerate()
        .map(|(i, p1)| {
            let mut min = (Vec3::X, f32::INFINITY);
            for (j, p2) in points.iter().enumerate() {
                if i == j {
                    continue;
                }
                let d = *p1 - *p2;
                let dl = d.length();
                if dl < min.1 {
                    min = if dl > 0.0 {
                        (d.normalize(), dl)
                    } else {
                        let mut rng = thread_rng();
                        (
                            vec3(
                                rng.gen::<f32>() - 0.5,
                                rng.gen::<f32>() - 0.5,
                                rng.gen::<f32>() - 0.5,
                            )
                            .normalize(),
                            0.000001,
                        )
                    }
                }
            }
            min
        })
        .collect();

    let mut min = f32::INFINITY;
    let mut max = 0.0;
    for (_, d) in closest.iter() {
        min = d.min(min);
        max = d.max(max);
    }

    let bounds = bounds();
    let forces_iter = closest.into_iter().map(|(direction, magnitude)| {
        direction * (1.0 - (magnitude - min) / (max - min + f32::EPSILON))
    });
    for (p, mut force) in points.iter_mut().zip(forces_iter) {
        for bound in &bounds.0 {
            let v = bound.distance(*p);
            force += bound.0 * (1.0 - 1.0 / (1.0 + std::f32::consts::E.powf(-2048.0 * v + 5.0)));
        }
        *p += 1e-3 * force;
    }
}

fn mean_minimum_distance(points: &[Vec3]) -> f32 {
    points
        .iter()
        .enumerate()
        .map(|(i, p1)| {
            let mut min = f32::INFINITY;
            for (j, p2) in points.iter().enumerate() {
                if i == j {
                    continue;
                }
                let d = (*p1 - *p2).length();
                min = min.min(d)
            }
            min
        })
        .sum::<f32>()
        / points.len() as f32
}

struct Bound(Vec3, f32);

impl Bound {
    fn distance(&self, v: Vec3) -> f32 {
        v.dot(self.0) - self.1
    }
}

struct Bounds(Vec<Bound>);

fn bounds() -> Bounds {
    Bounds(vec![
        Bound(-Vec3::X, -1.0),
        Bound(-Vec3::Y, -1.0),
        Bound(-Vec3::Z, -1.0),
        Bound(Vec3::X, 0.0),
        Bound(Vec3::Y, 0.0),
        Bound(Vec3::Z, 0.0),
        Bound(luma().normalize(), 0.1),
    ])
}

const fn luma() -> Vec3 {
    Vec3::new(0.299, 0.587, 0.114)
}
