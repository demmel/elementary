use std::os::windows::thread;

use bevy::{
    math::{vec3, Vec3Swizzles},
    prelude::{shape::UVSphere, *},
    render::render_resource::Face,
    time::FixedTimestep,
};
use rand::prelude::*;

pub fn choose_colors(n: usize) -> Vec<Color> {
    let mut points = get_initial_points(n);

    loop {
        tick_points(&mut points);
    }

    points
        .into_iter()
        .map(|c| Color::rgb(c.x, c.y, c.z))
        .collect()
}

fn get_initial_points(n: usize) -> Vec<Vec3> {
    let mut points = Vec::with_capacity(n);
    let mut rng = thread_rng();

    for i in 0..n {
        points.push(vec3(rng.gen(), rng.gen(), rng.gen()));
    }

    points
}

fn tick_points(points: &mut [Vec3]) -> f32 {
    let closest: Vec<_> = points
        .iter()
        .enumerate()
        .map(|(i, p1)| {
            let mut min = (Vec3::X, f32::INFINITY);
            for (j, p2) in points.iter().enumerate() {
                if (i == j) {
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
    let mean = closest
        .iter()
        .map(|(_, d)| {
            min = d.min(min);
            max = d.max(max);
            d
        })
        .sum::<f32>()
        / closest.len() as f32;

    let forces: Vec<_> = closest
        .into_iter()
        .map(|(direction, magnitude)| {
            direction * (1.0 - (magnitude - min) / (max - min + f32::EPSILON))
        })
        .collect();

    let bounds = bounds();
    for (p, mut force) in points.iter_mut().zip(forces.into_iter()) {
        for bound in &bounds.0 {
            let v = bound.distance(*p);
            force += bound.0 * (1.0 - 1.0 / (1.0 + std::f32::consts::E.powf(-2048.0 * v + 5.0)));
        }
        *p += 1e-2 * force;
    }
}

pub struct ChooseColorVisualization;

impl Plugin for ChooseColorVisualization {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_world).add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(1.0 / 30.0))
                .with_system(adjust_points)
                .with_system(adjust_colors.after(adjust_points)),
        );
    }
}

#[derive(Component)]
struct Point;

fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ambient_light: ResMut<AmbientLight>,
) {
    *ambient_light = AmbientLight {
        color: Color::WHITE,
        brightness: 0.0,
    };

    let sphere_mesh = meshes.add(
        UVSphere {
            radius: 0.01,
            ..default()
        }
        .into(),
    );

    let points = get_initial_points(1000);

    for p in points {
        commands
            .spawn_bundle(PbrBundle {
                mesh: sphere_mesh.clone(),
                material: materials.add(StandardMaterial {
                    base_color: Color::BLACK,
                    emissive: Color::rgb(p.x, p.y, p.z),
                    ..default()
                }),
                transform: Transform::from_translation(p),
                ..default()
            })
            .insert(Point);
    }
}

fn adjust_points(mut points: Query<(Entity, &mut Transform), With<Point>>) {
    let bounds = bounds();
    let n_points = points.iter().count();

    let closest: Vec<_> = points
        .iter()
        .map(|(e1, t1)| {
            let mut min = (Vec3::X, f32::INFINITY);
            for (e2, t2) in points.iter() {
                if (e1 == e2) {
                    continue;
                }
                let d = t1.translation - t2.translation;
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
    let mean = closest
        .iter()
        .map(|(_, d)| {
            min = d.min(min);
            max = d.max(max);
            d
        })
        .sum::<f32>()
        / closest.len() as f32;

    let forces: Vec<_> = closest
        .into_iter()
        .map(|(direction, magnitude)| {
            direction * (1.0 - (magnitude - min) / (max - min + f32::EPSILON))
        })
        .collect();

    for ((_, mut t), mut force) in points.iter_mut().zip(forces.into_iter()) {
        for bound in &bounds.0 {
            let v = bound.distance(t.translation);
            force += bound.0 * (1.0 - 1.0 / (1.0 + std::f32::consts::E.powf(-2048.0 * v + 5.0)));
        }
        t.translation += 1e-2 * force;
    }
}

fn adjust_colors(
    mut materials: ResMut<Assets<StandardMaterial>>,
    points: Query<(&Transform, &Handle<StandardMaterial>), With<Point>>,
) {
    for (t, m) in points.iter() {
        let m = materials.get_mut(m).unwrap();
        m.emissive = Color::rgb(t.translation.x, t.translation.y, t.translation.z);
    }
}

struct Bound(Vec3, f32);

impl Bound {
    fn distance(&self, v: Vec3) -> f32 {
        v.dot(self.0) - self.1
    }

    fn is_in_bound(&self, v: Vec3) -> bool {
        self.distance(v) > 0.0
    }
}

struct Bounds(Vec<Bound>);

impl Bounds {
    fn is_in_bounds(&self, v: Vec3) -> bool {
        self.0.iter().all(|b| b.is_in_bound(v))
    }
}

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
