//disable snake_case naming convention
#![allow(non_snake_case)]
#![allow(dead_code)]
mod code;
mod ecs;

use self::code::traits::*;
use self::ecs::components::*;
use self::ecs::resources::*;

use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
    utils::hashbrown::HashMap,
};
use bevy_prototype_debug_lines::{DebugLinesPlugin, DebugShapes};
use rand::Rng;
use rand_distr::{Distribution, UnitCircle};

use dashmap::{DashMap};

#[derive(Component)]
struct FpsText;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_plugins(DefaultPlugins)
        .add_plugin(DebugLinesPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .insert_resource(SparseSpatialHash::<Entity> {
            cell_size: 35.,
            grid: HashMap::new(),
        })
        //.insert_resource(SpatialHash::<Vec3>::new(100000, 100000, 1000, Vec3::ZERO))
        .insert_resource(BoidWorld {
            width: 100000.,
            height: 100000.,
        })
        .add_startup_system(setup.in_base_set(StartupSet::PreStartup))
        .add_startup_system(
            register_boids
                .in_base_set(StartupSet::PostStartup)
                .after(setup),
        )
        //.add_startup_system(update_windmap)
        .add_system(camera_input)
        .add_system(kinematic_simulation)
        .add_system(update_spatial_hash.after(kinematic_simulation))
        .add_system(boid_get_neighbors)
        .add_system(boid_flock)
        .add_system(show_fps)
        .run();
}

fn debug_draw_boids(
    mut shapes: ResMut<DebugShapes>,
    query_boids: Query<(&Transform, &Boid)>,
    query_neighbors: Query<(Entity, &Transform), With<Boid>>,
) {
    for (transform, boid) in query_boids.iter() {
        for neighbor in boid.neighbours.iter() {
            let (_n_entity, n_transform) = query_neighbors.get(*neighbor).unwrap();
            shapes
                .line()
                .start(transform.translation)
                .end(n_transform.translation)
                .color(Color::RED);
        }
    }
}

fn debug_draw_shash(
    mut shapes: ResMut<DebugShapes>,
    spatial_hash: ResMut<SparseSpatialHash<Entity>>,
) {
    spatial_hash.debug_display(&mut shapes);
}

fn show_fps(mut query: Query<&mut Text, With<FpsText>>, diagnostics: Res<Diagnostics>) {
    for mut text in query.iter_mut() {
        if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                // Update the value of the second section
                text.sections[1].value = format!("{value:.2}");
            }
        }
    }
}

fn debug_draw_kinematics(mut shapes: ResMut<DebugShapes>, query: Query<&Kinematic>) {
    for kinematic in query.iter() {
        kinematic.debug_display(&mut shapes);
    }
}

//Camera movement
fn camera_input(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Transform, &mut OrthographicProjection)>,
) {
    for (mut transform, mut ortho) in query.iter_mut() {
        //Todo: Get/make a better input system (Perhaps something similar to unity's legacy input system?)
        let mut direction = Vec3::new(0., 0., 0.);
        if keyboard_input.pressed(KeyCode::W) {
            direction += Vec3::new(0., 1., 0.);
        }
        if keyboard_input.pressed(KeyCode::S) {
            direction += Vec3::new(0., -1., 0.);
        }
        if keyboard_input.pressed(KeyCode::A) {
            direction += Vec3::new(-1., 0., 0.);
        }
        if keyboard_input.pressed(KeyCode::D) {
            direction += Vec3::new(1., 0., 0.);
        }
        //zoom in and out with q and e
        if keyboard_input.pressed(KeyCode::Q) {
            direction += Vec3::new(0., 0., 1.);
        }
        if keyboard_input.pressed(KeyCode::E) {
            direction += Vec3::new(0., 0., -1.);
        }
        transform.translation += Vec3::new(direction.x, direction.y, 0.0) * 10.;
        ortho.scale += direction.z * 0.01;
    }
}

fn register_boids(
    mut spatial_hash: ResMut<SparseSpatialHash<Entity>>,
    query: Query<(Entity, &Transform, &Boid)>,
) {
    for (entity, transform, _boid) in query.iter() {
        spatial_hash.insert(entity, transform.translation);
    }
}

fn update_spatial_hash(
    mut spatial_hash: ResMut<SparseSpatialHash<Entity>>,
    query: Query<(&Kinematic, Entity, &Transform), With<Boid>>,
) {
    query.for_each(|(kinematic, entity, transform)| {
        spatial_hash.update(entity, kinematic.last_position, transform.translation);
    });
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    let boid_sprite = asset_server.load("textures/boid.png");

    commands.spawn_batch((0..20000).map(move |_c| {
        let mut rng = rand::thread_rng();
        let r_point: [f32; 2] = UnitCircle.sample(&mut rng);
        let r_radius = rng.gen_range(-10000.0..10000.0);
        let random_position = Vec3::new(r_point[0] * r_radius, r_point[1] * r_radius, 0.0);

        (
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(1.0, 1.0, 1.0),
                    ..default()
                },
                transform: Transform {
                    scale: Vec3::splat(0.05),
                    translation: random_position,
                    ..default()
                },
                texture: boid_sprite.clone(),
                ..default()
            },
            Boid {
                neighbours: Vec::new(),
            },
            Kinematic {
                velocity: Vec3::new(
                    rng.gen_range(-200.0..200.0),
                    rng.gen_range(-200.0..200.0),
                    0.0,
                ),
                acceleration: Vec3::splat(0.0),
                last_position: Vec3::splat(0.0),
            },
            KinematicConstraint {
                max_speed: 200.0,
                max_force: 2.5,
            },
        )
    }));

    //Spawn boids

    commands.spawn((
        // Create a TextBundle that has a Text with a list of sections.
        TextBundle::from_sections([
            TextSection::new(
                "FPS: ",
                TextStyle {
                    font: asset_server.load("fonts/Ac437_IBM_BIOS.ttf"),
                    font_size: 24.0,
                    color: Color::WHITE,
                },
            ),
            TextSection::from_style(TextStyle {
                font: asset_server.load("fonts/Ac437_IBM_BIOS.ttf"),
                font_size: 24.0,
                color: Color::GOLD,
            }),
        ]),
        FpsText,
    ));
}

fn boid_flock(
    mut boid_transforms: Query<(
        Entity,
        &mut Kinematic,
        &KinematicConstraint,
        &Transform,
        &Boid,
    )>,
) {
    let steer_forces = DashMap::new();

    //Collect boid_transforms into a vector
    boid_transforms
    .par_iter()
    .for_each(|(entity, kinematic, constraint, transform, boid)| {
        let mut average_velocity = Vec3::splat(0.0);
        let mut average_position = Vec3::splat(0.0);
        let mut separation_force = Vec3::splat(0.0);
        let mut count = 0;
        for neighbor in &boid.neighbours {
            let n = boid_transforms.get(*neighbor).unwrap();
            if n.0 == entity {
                continue;
            }
            average_velocity += n.1.velocity;
            average_position += n.3.translation;

            let distance = transform.translation.distance(n.3.translation);
            if distance > 0.0 && distance < 50.0 {
                let mut diff = (transform.translation - n.3.translation).normalize();
                diff /= distance;
                separation_force += diff;
            }

            count += 1;
        }
        if count > 0 {
            //Average the forces.
            average_velocity /= count as f32;
            average_position /= count as f32;
            separation_force /= count as f32;

            average_velocity = average_velocity.normalize() * constraint.max_speed;
            average_position = (average_position - transform.translation).normalize();

            let desired_velocity =
                (average_position - kinematic.last_position).normalize() * constraint.max_speed;

            //Steering force = desired velocity - current velocity
            let steer_alignment =
                (average_velocity - kinematic.velocity).clamp_length(0.0, constraint.max_force);

            let steer_cohesion =
                (desired_velocity - kinematic.velocity).clamp_length(0.0, constraint.max_force);
            if steer_cohesion.length() > 0.0 {
                steer_forces.insert(entity, steer_cohesion);
            }
            if steer_alignment.length() > 0.0 {
                steer_forces.insert(entity, steer_alignment);
            }
        }
        if separation_force.length() > 0.0 {
            separation_force = ((separation_force.normalize() * constraint.max_speed)
                - kinematic.velocity)
                .clamp_length(0.0, constraint.max_force);
            steer_forces.insert(entity, separation_force);
        }
    });
    {
        //For profiling
        let _apply_forces_span = info_span!("apply_flocking_forces",name="apply_flocking_forces").entered();
        //Loop through hasmap and apply forces to boids
        for (entity, steer) in steer_forces {
            let mut kinematic = boid_transforms.get_mut(entity).unwrap();
            kinematic.1.acceleration += steer;
        }
    }
}

fn boid_get_neighbors(
    mut boid_transforms: Query<(&Transform, &mut Boid)>,
    boids: Query<Entity, With<Boid>>,
    spatial_hash: ResMut<SparseSpatialHash<Entity>>,
) {
        //Provides about 2-3x speedup with no obvious issues
    boid_transforms
        .par_iter_mut()
        .for_each_mut(|(transform, mut boid)| {
            boid.neighbours.clear();
            let neighbors = spatial_hash.get_neighbors(transform.translation);
            for neighbor_cell in neighbors {
                for neighbor in neighbor_cell {
                    let n_entity = boids.get(neighbor).unwrap();

                    boid.neighbours.push(n_entity);
                }
            }
        });
}

fn kinematic_simulation(
    mut kinematic_bodies: Query<(&mut Kinematic, &mut Transform)>,
    time: Res<Time>,
    world: Res<BoidWorld>,
) {
    kinematic_bodies
        .par_iter_mut()
        .for_each_mut(|(mut kinematic, mut transform)| {
            let acceleration = kinematic.acceleration;

            kinematic.last_position = transform.translation;
            kinematic.velocity += acceleration;
            transform.rotation =
                Quat::from_rotation_z(kinematic.velocity.y.atan2(kinematic.velocity.x));

            transform.translation += kinematic.velocity * time.delta_seconds();

            kinematic.acceleration = Vec3::splat(0.0);
            //wrap around world
            if transform.translation.x > world.width / 2.0 {
                transform.translation.x = -world.width / 2.0;
            } else if transform.translation.x < -world.width / 2.0 {
                transform.translation.x = world.width / 2.0;
            }

            if transform.translation.y > world.height / 2.0 {
                transform.translation.y = -world.height / 2.0;
            } else if transform.translation.y < -world.height / 2.0 {
                transform.translation.y = world.height / 2.0;
            }
        });
}
