mod choose_color;

use std::f32::consts::PI;

use bevy::{
    prelude::{shape::UVSphere, *},
    utils::HashMap,
    window::{PresentMode, WindowMode},
};
use bevy_flycam::PlayerPlugin;
use choose_color::choose_colors;

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
    .add_startup_system(setup_world);

    #[cfg(feature = "editor")]
    app.add_plugin(EditorPlugin)
        .add_plugin(FrameTimeDiagnosticsPlugin)
        .add_plugin(EntityCountDiagnosticsPlugin);

    app.run();
}

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
