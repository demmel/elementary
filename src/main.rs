mod barnes_hut;
mod choose_color;

use std::vec;

use barnes_hut::BarnesHutTree;
use bevy::{
    prelude::{shape::UVSphere, *},
    window::WindowMode,
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
const NUM_PARTICLES: usize = 2000;
const PARTICLE_SIZE: f32 = 0.01;
const PARTICLE_FORCE_MAX: f32 = 1e-5;
const BH_THETA: f32 = 1.0;

fn main() {
    let mut app = App::new();

    app.insert_resource(WindowDescriptor {
        title: "Elementary".to_string(),
        mode: WindowMode::Windowed,
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
    .init_resource::<ParticleTrees>()
    .add_startup_system(setup_world)
    .add_system(barnes_hut)
    .add_system(update_forces.after(barnes_hut));

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
                emissive: color,
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
                    rng.gen_range(-1.0..=1.0),
                    rng.gen_range(-1.0..=1.0),
                    rng.gen_range(-1.0..=1.0),
                )),
                ..default()
            })
            .insert(kind)
            .insert(RigidBody::Dynamic)
            .insert(Collider::ball(PARTICLE_SIZE))
            .insert(Restitution::coefficient(0.0))
            .insert(ExternalForce::default())
            .insert(ReadMassProperties::default());
    }
}

fn barnes_hut(
    particle_system: Res<ParticleSystem>,
    mut particle_trees: ResMut<ParticleTrees>,
    particles: Query<(Entity, &ParticleKindHandle, &Transform, &ReadMassProperties)>,
) {
    let bounds = particles.iter().fold(
        vec![
            (f32::INFINITY * Vec3::ONE, -f32::INFINITY * Vec3::ONE);
            particle_system.kinds().count()
        ],
        |mut bounds, (_, pk, t, _)| {
            let (min, max) = &mut bounds[pk.0];

            *min = min.min(t.translation);
            *max = max.max(t.translation);

            bounds
        },
    );

    *particle_trees = ParticleTrees(
        bounds
            .into_iter()
            .map(|(min, max)| BarnesHutTree::new(min, max))
            .collect(),
    );

    for (e, pk, t, m) in particles.iter() {
        particle_trees
            .tree_mut(*pk)
            .insert(e, t.translation, m.0.mass)
    }
}

fn update_forces(
    particle_system: Res<ParticleSystem>,
    particle_trees: Res<ParticleTrees>,
    mut particles: Query<(Entity, &ParticleKindHandle, &Transform, &mut ExternalForce)>,
) {
    for (e1, pk1, t1, mut f) in particles.iter_mut() {
        let mut force = Vec3::ZERO;

        for pk in particle_system.kinds() {
            let _span = info_span!("barnes_hut_force", name = "barnes_hut_force").entered();
            let rule = particle_system.rule(*pk1, pk);
            force += particle_trees.tree(pk).force(
                e1,
                t1.translation,
                rule.force,
                rule.distance_exp,
                BH_THETA,
            );
        }

        *f = ExternalForce { force, ..default() }
    }
}

#[derive(Debug, Default)]
struct ParticleTrees(Vec<BarnesHutTree<Entity>>);

impl ParticleTrees {
    fn tree(&self, pk: ParticleKindHandle) -> &BarnesHutTree<Entity> {
        &self.0[pk.0]
    }

    fn tree_mut(&mut self, pk: ParticleKindHandle) -> &mut BarnesHutTree<Entity> {
        &mut self.0[pk.0]
    }
}

#[derive(Clone, Copy, Component)]
struct ParticleKindHandle(usize);

#[derive(Debug)]
struct ParticleSystem {
    num_kinds: usize,
    rules: Vec<ParticleRule>,
}

#[derive(Debug)]
struct ParticleRule {
    force: f32,
    distance_exp: i32,
}

impl ParticleSystem {
    pub fn rand<R: Rng>(rng: &mut R, num_kinds: usize) -> Self {
        Self {
            num_kinds,
            rules: (0..(num_kinds * num_kinds))
                .map(|_| ParticleRule {
                    force: 2.0 * PARTICLE_FORCE_MAX * (rng.gen::<f32>() - 0.5),
                    distance_exp: rng.gen_range(-2..=1),
                })
                .collect(),
        }
    }

    pub fn kinds(&self) -> impl Iterator<Item = ParticleKindHandle> {
        (0..self.num_kinds).map(ParticleKindHandle)
    }

    pub fn rule(&self, pk1: ParticleKindHandle, pk2: ParticleKindHandle) -> &ParticleRule {
        &self.rules[self.index(pk1, pk2)]
    }

    fn index(&self, pk1: ParticleKindHandle, pk2: ParticleKindHandle) -> usize {
        pk1.0 * self.num_kinds + pk2.0
    }
}
