mod choose_color;

use std::f32::consts::PI;

use bevy::{
    prelude::{shape::UVSphere, *},
    utils::HashMap,
    window::{PresentMode, WindowMode},
};
use bevy_flycam::PlayerPlugin;
use choose_color::ChooseColorVisualization;

#[cfg(feature = "editor")]
use ::{
    bevy::diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin},
    bevy_editor_pls::prelude::*,
};

fn main() {
    let mut app = App::new();

    app.insert_resource(WindowDescriptor {
        title: "Elementary".to_string(),
        mode: WindowMode::BorderlessFullscreen,
        ..default()
    })
    .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
    .add_plugins(DefaultPlugins)
    .add_plugin(PlayerPlugin)
    .add_plugin(ChooseColorVisualization);
    // .add_startup_system(setup_world);

    #[cfg(feature = "editor")]
    app.add_plugin(EditorPlugin)
        .add_plugin(FrameTimeDiagnosticsPlugin)
        .add_plugin(EntityCountDiagnosticsPlugin);

    app.run();
}

fn choose_colors(n: usize) -> Vec<Color> {
    let mut colors = Vec::with_capacity(n);

    for i in 0..n {
        colors.push(
            Vec3::new(
                (i % 256) as f32,
                ((127 * i) % 256) as f32,
                ((255 * i) / 256) as f32,
            ) / 255.0,
        );
    }

    let mut prior_d = 0.0;
    loop {
        let mut d = 0.0;
        let mut forces = vec![Vec3::ZERO; n];
        for i in 0..colors.len() {
            let mut c1 = colors[i];

            for j in (i + 1)..colors.len() {
                let c2 = colors[j];
                let c1_c2_d = c1.distance(c2);
                forces[i] += (c1 - c2) / c1_c2_d.powi(2);
                forces[j] += (c2 - c1) / c1_c2_d.powi(2);
                d += c1_c2_d;
            }

            forces[i] /= (n - 1) as f32;

            forces[i] += -Vec3::X;
            forces[i] += -Vec3::Y;
            forces[i] += -Vec3::Z;
            let lum = Vec3::new(0.299, 0.587, 0.114);
            forces[i] += lum / lum.length();

            forces[i] /= 5.0;

            colors[i] += forces[i];

            // colors[i] = match (c1.x > 1.0, c1.y > 1.0, c1.z > 1.0, l < 0.1) {
            //     (true, true, true, false) => Vec3::ONE,
            //     (true, true, false, false) => Vec3::new(1.0, 1.0, c1.z),
            //     (true, false, true, false) => Vec3::new(1.0, c1.y, 1.0),
            //     (true, false, false, false) => Vec3::new(1.0, c1.y, c1.z),
            //     (false, true, true, false) => Vec3::new(c1.x, 1.0, 1.0),
            //     (false, true, false, false) => Vec3::new(c1.x, 1.0, c1.z),
            //     (false, false, true, false) => Vec3::new(c1.x, c1.y, 1.0),
            //     (false, false, false, true) => todo!(),
            //     (false, false, false, false) => c1,
            //     (true, true, false, true)
            //     | (true, false, true, true)
            //     | (true, false, false, true)
            //     | (false, true, true, true)
            //     | (false, true, false, true)
            //     | (false, false, true, true)
            //     | (true, true, true, true) => panic!("Impossible"),
            // }
        }

        d /= (n * (n - 1) / 2) as f32;

        println!("{}", d - prior_d);

        prior_d = d;

        let out_of_bounds = colors.iter().filter(|c| {
            c.x > 1.0 || c.y > 1.0 || c.z > 1.0 || c.dot(Vec3::new(0.299, 0.587, 0.114)) < 0.1
        });
        for color in out_of_bounds {
            println!("{color:?}");
        }
    }

    colors
        .into_iter()
        .map(|c| Color::rgb(c.x, c.y, c.z))
        .collect()
}

fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let sphere_mesh = meshes.add(
        UVSphere {
            radius: 0.01,
            ..default()
        }
        .into(),
    );

    let num_particles = 1000;

    let colors = choose_colors(num_particles);

    for (i, color) in colors.iter().enumerate() {
        let r = 5.0;
        let theta = 2.0 * PI * (i as f32 / num_particles as f32);
        commands.spawn_bundle(PbrBundle {
            mesh: sphere_mesh.clone(),
            material: materials.add(StandardMaterial {
                emissive: color.clone(),
                ..default()
            }),
            transform: Transform::from_translation(Vec3::new(
                r * theta.cos(),
                0.0,
                r * theta.sin(),
            )),
            ..default()
        });
    }
}

#[derive(Component)]
struct ParticleKindHandle(usize);

struct ParticleRules(HashMap<ParticleKindHandle, HashMap<ParticleKindHandle, f32>>);

#[derive(Bundle)]
struct Particle {
    kind: ParticleKindHandle,
}
