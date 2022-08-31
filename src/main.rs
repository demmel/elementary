mod choose_color;

use std::f32::consts::PI;

use bevy::{
    prelude::{shape::UVSphere, *},
    time::FixedTimestep,
    utils::HashMap,
    window::{PresentMode, WindowMode},
};
use bevy_flycam::PlayerPlugin;
use bevy_rapier3d::prelude::*;
use choose_color::choose_colors;
use rand::{thread_rng, Rng};

#[cfg(feature = "editor")]
use ::{
    bevy::diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin},
    bevy_editor_pls::prelude::*,
};

const NUM_KINDS: usize = 4;
const NUM_PARTICLES: usize = 3000;
const PARTICLE_SIZE: f32 = 0.01;
const PARTICLE_FORCE_MAX: f32 = 0.001;

fn main() {
    let mut app = App::new();

    app.insert_resource(WindowDescriptor {
        title: "Elementary".to_string(),
        mode: WindowMode::BorderlessFullscreen,
        ..default()
    })
    .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
    .add_plugins(DefaultPlugins)
    .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
    .insert_resource(RapierConfiguration {
        gravity: Vect::ZERO,
        timestep_mode: TimestepMode::Variable {
            max_dt: 1.0 / 240.0,
            time_scale: 0.01,
            substeps: 1,
        },
        ..default()
    })
    .add_plugin(PlayerPlugin)
    .insert_resource(ParticleSystem::rand(&mut thread_rng(), NUM_KINDS))
    .add_startup_system(setup_world)
    .add_system(update_forces);

    #[cfg(feature = "editor")]
    app.add_plugin(EditorPlugin)
        .add_plugin(FrameTimeDiagnosticsPlugin)
        .add_plugin(EntityCountDiagnosticsPlugin);

    app.run();
}

fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ambient_light: ResMut<AmbientLight>,
    particle_system: Res<ParticleSystem>,
) {
    println!("{particle_system:?}");

    *ambient_light = AmbientLight {
        color: Color::WHITE,
        brightness: 0.0,
    };

    let sphere_mesh = meshes.add(
        UVSphere {
            radius: PARTICLE_SIZE,
            ..default()
        }
        .into(),
    );

    let kinds: Vec<_> = particle_system.kinds().collect();
    let color_materials: Vec<_> = choose_colors(kinds.len())
        .into_iter()
        .map(|color| {
            materials.add(StandardMaterial {
                emissive: color.clone(),
                ..default()
            })
        })
        .collect();

    let mut rng = thread_rng();
    for _ in 0..NUM_PARTICLES {
        let kind_i = rng.gen_range(0..kinds.len());
        let kind = kinds[kind_i];

        commands
            .spawn_bundle(PbrBundle {
                mesh: sphere_mesh.clone(),
                material: color_materials[kind_i].clone(),
                transform: Transform::from_translation(Vec3::new(
                    1.0 * (rng.gen::<f32>() - 0.5),
                    1.0 * (rng.gen::<f32>() - 0.5),
                    1.0 * (rng.gen::<f32>() - 0.5),
                )),
                ..default()
            })
            .insert(kind)
            .insert(RigidBody::Dynamic)
            .insert(Collider::ball(PARTICLE_SIZE))
            .insert(Restitution::coefficient(0.0))
            .insert(ExternalForce::default());
    }
}

fn update_forces(
    particle_system: Res<ParticleSystem>,
    mut particles_with_velocity: Query<(
        Entity,
        &ParticleKindHandle,
        &Transform,
        &mut ExternalForce,
    )>,
    particles: Query<(Entity, &ParticleKindHandle, &Transform)>,
) {
    for (e1, k1, t1, mut f) in particles_with_velocity.iter_mut() {
        let mut force = Vec3::ZERO;
        for (e2, k2, t2) in particles.iter() {
            if e1 == e2 {
                continue;
            }
            let d = (t2.translation - t1.translation);
            force += particle_system.rule(*k1, *k2) * d.normalize_or_zero()
                / (d.length_squared() + f32::EPSILON);
        }
        *f = ExternalForce { force, ..default() }
    }
}

#[derive(Clone, Copy, Component)]
struct ParticleKindHandle(usize);

#[derive(Debug)]
struct ParticleSystem {
    num_kinds: usize,
    rules: Vec<f32>,
}

impl ParticleSystem {
    pub fn rand<R: Rng>(rng: &mut R, num_kinds: usize) -> Self {
        Self {
            num_kinds,
            rules: (0..(num_kinds * num_kinds))
                .map(|_| 2.0 * PARTICLE_FORCE_MAX * (rng.gen::<f32>() - 0.5))
                .collect(),
        }
    }

    pub fn kinds(&self) -> impl Iterator<Item = ParticleKindHandle> {
        (0..self.num_kinds).map(|pk| ParticleKindHandle(pk))
    }

    pub fn rule(&self, pk1: ParticleKindHandle, pk2: ParticleKindHandle) -> f32 {
        self.rules[self.index(pk1, pk2)]
    }

    pub fn rule_mut(&mut self, pk1: ParticleKindHandle, pk2: ParticleKindHandle) -> &mut f32 {
        let index = self.index(pk1, pk2);
        &mut self.rules[index]
    }

    fn index(&self, pk1: ParticleKindHandle, pk2: ParticleKindHandle) -> usize {
        pk1.0 * self.num_kinds + pk2.0
    }
}
