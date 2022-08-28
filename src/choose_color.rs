use bevy::{
    math::Vec3Swizzles,
    prelude::{shape::UVSphere, *},
    time::FixedTimestep,
};

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

fn is_point_in_bounds(p: Vec3) -> bool {
    !(p.x > 1.0 || p.y > 1.0 || p.z > 1.0 || p.dot(luma()) < 0.1)
}

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

    for p in points {
        commands
            .spawn_bundle(PbrBundle {
                mesh: sphere_mesh.clone(),
                material: if !is_point_in_bounds(p) {
                    materials.red.clone()
                } else {
                    materials.green.clone()
                },
                transform: Transform::from_translation(p),
                ..default()
            })
            .insert(Point);
    }
}

fn adjust_points(
    materials: Res<Materials>,
    mut points: Query<(Entity, &mut Transform, &mut Handle<StandardMaterial>), With<Point>>,
) {
    let planes = [
        (-Vec3::X, -1.0),
        (-Vec3::Y, -1.0),
        (-Vec3::Z, -1.0),
        (luma().normalize(), 0.1),
    ];

    let n_points = points.iter().count();
    let forces: Vec<_> = points
        .iter()
        .map(|(e1, t1, _)| {
            let mut force = Vec3::ZERO;
            for (e2, t2, _) in points.iter() {
                if (e1 == e2) {
                    continue;
                }
                force += ((t1.translation - t2.translation)
                    / (t1.translation.distance_squared(t2.translation) + f32::EPSILON));
            }

            for (normal, offset) in planes {
                let v = t1.translation.dot(normal) - offset;
                force += (n_points - 1) as f32
                    * if v > 0.0 {
                        normal / (v.powi(2) + f32::EPSILON)
                    } else {
                        normal / f32::EPSILON
                    }
            }

            force.normalize()
        })
        .collect();

    for ((_, mut t, mut m), force) in points.iter_mut().zip(forces.into_iter()) {
        t.translation += 1e-3 * force;
        *m = if !is_point_in_bounds(t.translation) {
            materials.red.clone()
        } else {
            materials.green.clone()
        }
    }
}

const fn luma() -> Vec3 {
    Vec3::new(0.299, 0.587, 0.114)
}
