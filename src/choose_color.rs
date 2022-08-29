use std::os::windows::thread;

use bevy::{
    math::Vec3Swizzles,
    prelude::{shape::UVSphere, *},
    render::render_resource::Face,
    time::FixedTimestep,
};
use bevy_editor_pls::egui::color_picker::Alpha;

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

    for i in 0..n {
        points.push(
            Vec3::new(
                (i % 256) as f32,
                ((127 * i) % 256) as f32,
                ((255 * i) % 256) as f32,
            ) / 255.0,
        );
    }

    points
}

fn tick_points(points: &mut [Vec3]) {}

pub struct ChooseColorVisualization;

impl Plugin for ChooseColorVisualization {
    fn build(&self, app: &mut App) {
        app.init_resource::<Materials>()
            .add_startup_system(setup_world)
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(FixedTimestep::step(1.0 / 60.0))
                    .with_system(adjust_points),
            );
    }
}

struct Materials {
    red: Handle<StandardMaterial>,
    green: Handle<StandardMaterial>,
}

#[derive(Component)]
struct Point;

impl FromWorld for Materials {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world
            .get_resource_mut::<Assets<StandardMaterial>>()
            .unwrap();
        Materials {
            red: materials.add(StandardMaterial {
                emissive: Color::rgb(1.0, 0.0, 0.0).clone(),
                ..default()
            }),
            green: materials.add(StandardMaterial {
                emissive: Color::rgb(0.0, 1.0, 0.0).clone(),
                ..default()
            }),
        }
    }
}

fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut asset_materials: ResMut<Assets<StandardMaterial>>,
    mut materials: Res<Materials>,
) {
    let sphere_mesh = meshes.add(
        UVSphere {
            radius: 0.01,
            ..default()
        }
        .into(),
    );

    let points = get_initial_points(1500);
    let bounds = bounds();

    for p in points {
        commands
            .spawn_bundle(PbrBundle {
                mesh: sphere_mesh.clone(),
                material: if !bounds.is_in_bounds(p) {
                    materials.red.clone()
                } else {
                    materials.green.clone()
                },
                transform: Transform::from_translation(p),
                ..default()
            })
            .insert(Point);
    }

    let plane_material = asset_materials.add(StandardMaterial {
        base_color: Color::rgba(1.0, 1.0, 1.0, 0.1).clone(),
        double_sided: true,
        cull_mode: None,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    let plane_mesh = meshes.add(shape::Plane { size: 2. }.into());

    for bound in &bounds.0 {
        let mut transform = Transform::from_rotation(Quat::from_rotation_arc(Vec3::Y, bound.0));
        transform.translation += bound.1 * transform.local_y();
        commands.spawn_bundle(PbrBundle {
            mesh: plane_mesh.clone(),
            material: plane_material.clone(),
            transform,
            ..default()
        });
    }
}

fn adjust_points(
    materials: Res<Materials>,
    mut points: Query<(Entity, &mut Transform, &mut Handle<StandardMaterial>), With<Point>>,
) {
    let bounds = bounds();
    let n_points = points.iter().count();
    let forces: Vec<_> = points
        .iter()
        .map(|(e1, t1, _)| {
            let mut force = Vec3::ZERO;
            for (e2, t2, _) in points.iter() {
                if (e1 == e2) {
                    continue;
                }

                let d = t1.translation.distance(t2.translation);
                force += (t1.translation - t2.translation) / (d.powi(2) + f32::EPSILON);
            }

            for bound in &bounds.0 {
                let v = bound.distance(t1.translation);
                force += ((n_points - 1) / bounds.0.len()) as f32
                    * if v > 0.0 {
                        bound.0 / (v.powi(2) + f32::EPSILON)
                    } else {
                        bound.0 / f32::EPSILON
                    };
            }

            force.normalize()
        })
        .collect();

    for ((_, mut t, mut m), force) in points.iter_mut().zip(forces.into_iter()) {
        t.translation += 1e-4 * force;
        *m = if !bounds.is_in_bounds(t.translation) {
            materials.red.clone()
        } else {
            materials.green.clone()
        }
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
