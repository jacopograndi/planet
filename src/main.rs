use std::collections::HashMap;
use std::time::Duration;

use rand::prelude::*;

use bevy::sprite::MaterialMesh2dBundle;
use bevy::{prelude::*, window::WindowResized};

use bevy_rapier2d::{pipeline::CollisionEvent::*, prelude::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(movement)
        .add_system(move_enemies)
        .add_system(shooting)
        .add_system(bullet_clean)
        .add_system(enemy_clean)
        .add_system(window_resized_event)
        .add_system(spawn_enemies)
        .add_system_to_stage(CoreStage::PostUpdate, collision_resolve)
        .init_resource::<AssetHandles>()
        .insert_resource(RapierConfiguration {
            gravity: Vec2::new(0.0, 0.0),
            ..default()
        })
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default().with_physics_scale(100.0))
        .run();
}

// dynamic asset storage

#[derive(Eq, Hash, PartialEq)]
enum MeshName {
    Circle,
    Triangle,
    Capsule,
}

#[derive(Eq, Hash, PartialEq)]
enum MaterialName {
    Sky,
    Planet,
    Player,
    Enemy,
}

#[derive(Default)]
struct AssetHandles {
    meshes: HashMap<MeshName, Handle<Mesh>>,
    materials: HashMap<MaterialName, Handle<ColorMaterial>>,
}

// game components

#[derive(Component)]
struct Planet {
    size: f32,
    hp: f32,
}

#[derive(Component)]
struct Player {
    speed: f32,
    timer: Timer,
}

#[derive(Component)]
struct Bullet {
    lifetime: Timer,
    damage: f32,
    has_hit: u8,
}

#[derive(Component)]
struct Spawner {
    spawntimer: Timer,
    size: f32,
}

#[derive(Component)]
struct Enemy {
    speed: f32,
    has_hit: u8,
    damage: f32,
    hp: f32,
}

fn window_resized_event(windows: Res<Windows>, mut projection: Query<&mut OrthographicProjection>) {
    let window = windows.primary();
    let viewsize = Vec2::new(window.width(), window.height());
    let min = if viewsize.x < viewsize.y {
        viewsize.x
    } else {
        viewsize.y
    };
    let scale = if min < 1024.0 { 1024.0 / min } else { 1.0 };
    projection.single_mut().scale = scale;
}

fn setup(
    mut commands: Commands,
    mut handles: ResMut<AssetHandles>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut camera_bundle = Camera2dBundle::new_with_far(100.0);
    commands.spawn_bundle(camera_bundle);

    handles.meshes.insert(
        MeshName::Circle,
        meshes.add(Mesh::from(shape::Circle::default())),
    );
    handles.meshes.insert(
        MeshName::Triangle,
        meshes.add(Mesh::from(shape::RegularPolygon::new(8.0, 3))),
    );
    handles.meshes.insert(
        MeshName::Capsule,
        meshes.add(Mesh::from(shape::Capsule::default())),
    );

    handles.materials.insert(
        MaterialName::Planet,
        materials.add(ColorMaterial::from(Color::PURPLE)),
    );
    handles.materials.insert(
        MaterialName::Sky,
        materials.add(ColorMaterial::from(Color::BLACK)),
    );
    handles.materials.insert(
        MaterialName::Player,
        materials.add(ColorMaterial::from(Color::BLUE)),
    );
    handles.materials.insert(
        MaterialName::Enemy,
        materials.add(ColorMaterial::from(Color::RED)),
    );

    commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: handles
                .meshes
                .get(&MeshName::Circle)
                .unwrap()
                .clone_weak()
                .into(),
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 0.0),
                scale: Vec3::new(1024.0, 1024.0, 1.0),
                ..default()
            },
            material: handles
                .materials
                .get(&MaterialName::Sky)
                .unwrap()
                .clone_weak(),
            ..default()
        })
        .insert(Spawner {
            spawntimer: Timer::new(Duration::from_millis(2000), false),
            size: 1024.0,
        });

    commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: handles
                .meshes
                .get(&MeshName::Circle)
                .unwrap()
                .clone_weak()
                .into(),
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 1.0),
                scale: Vec3::new(192.0, 192.0, 1.0),
                ..default()
            },
            material: handles
                .materials
                .get(&MaterialName::Planet)
                .unwrap()
                .clone_weak(),
            ..default()
        })
        .insert(Collider::ball(0.5))
        .insert(CollisionGroups::new(0b100, 0b111))
        .insert(Planet {
            size: 192.0,
            hp: 100.0,
        });

    commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: handles
                .meshes
                .get(&MeshName::Triangle)
                .unwrap()
                .clone_weak()
                .into(),
            transform: Transform {
                translation: Vec3::new(0.0, 92.0 + 16.0, 2.0),
                scale: Vec3::new(1.0, 1.0, 1.0),
                ..default()
            },
            material: handles
                .materials
                .get(&MaterialName::Player)
                .unwrap()
                .clone_weak(),
            ..default()
        })
        .insert(Player {
            speed: 300.0,
            timer: Timer::new(Duration::from_millis(200), false),
        });
}

fn spawn_enemies(
    time: Res<Time>,
    mut commands: Commands,
    handles: ResMut<AssetHandles>,
    mut spawner_query: Query<(&mut Spawner, &Transform)>,
) {
    let mut rng = thread_rng();
    for (mut spawner, transform) in &mut spawner_query {
        spawner.spawntimer.tick(time.delta());
        if spawner.spawntimer.finished() {
            spawner.spawntimer.reset();

            let angle: f32 = rng.gen_range(0.0..(2.0 * std::f32::consts::PI));
            let pos = Vec3::new(
                f32::cos(angle) * (spawner.size * 0.5),
                f32::sin(angle) * (spawner.size * 0.5),
                3.0,
            ) + transform.translation;
            let acc = Vec2::new(-pos.y, pos.x).normalize();

            commands
                .spawn_bundle(MaterialMesh2dBundle {
                    mesh: handles
                        .meshes
                        .get(&MeshName::Capsule)
                        .unwrap()
                        .clone_weak()
                        .into(),
                    transform: Transform {
                        translation: pos,
                        rotation: Quat::from_rotation_z(angle),
                        scale: Vec3::new(20.0, 20.0, 1.0),
                        ..default()
                    },
                    material: handles
                        .materials
                        .get(&MaterialName::Enemy)
                        .unwrap()
                        .clone_weak(),
                    ..default()
                })
                .insert(RigidBody::Dynamic)
                .insert(Restitution::coefficient(0.0))
                .insert(Collider::capsule(
                    Vec2::new(0.0, -0.5),
                    Vec2::new(0.0, 0.5),
                    0.5,
                ))
                .insert(Damping {
                    linear_damping: 1.0,
                    angular_damping: 10.0,
                })
                .insert(Velocity::linear(acc * 120.0))
                .insert(CollisionGroups::new(0b001, 0b111))
                .insert(ActiveEvents::COLLISION_EVENTS)
                .insert(Enemy {
                    speed: 2.0,
                    has_hit: 0,
                    damage: 1.0,
                    hp: 100.0,
                });
        }
    }
}

fn shooting(
    time: Res<Time>,
    mut commands: Commands,
    handles: ResMut<AssetHandles>,
    mut player_query: Query<(&mut Player, &Transform)>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    let shooting = keyboard_input.pressed(KeyCode::S);
    let (mut player, player_trans) = player_query.single_mut();

    player.timer.tick(time.delta());
    if shooting && player.timer.finished() {
        player.timer.reset();

        let acc = player_trans.translation.normalize();
        let acc = Vec2::new(acc.x, acc.y);
        let mut angle = Vec2::angle_between(
            Vec2::X,
            Vec2::new(player_trans.translation.x, player_trans.translation.y),
        );
        if angle.is_nan() {
            angle = 0.0;
        }

        commands
            .spawn_bundle(MaterialMesh2dBundle {
                mesh: handles
                    .meshes
                    .get(&MeshName::Circle)
                    .unwrap()
                    .clone_weak()
                    .into(),
                transform: Transform {
                    translation: player_trans.translation,
                    rotation: Quat::from_rotation_z(angle),
                    scale: Vec3::new(5.0, 5.0, 1.0),
                    ..default()
                },
                material: handles
                    .materials
                    .get(&MaterialName::Player)
                    .unwrap()
                    .clone_weak(),
                ..default()
            })
            .insert(RigidBody::Dynamic)
            .insert(Restitution::coefficient(0.0))
            .insert(Collider::ball(0.5))
            .insert(LockedAxes::ROTATION_LOCKED)
            .insert(Damping {
                linear_damping: 0.2,
                angular_damping: 10.0,
            })
            .insert(Ccd::enabled())
            .insert(ActiveEvents::COLLISION_EVENTS)
            .insert(CollisionGroups::new(0b010, 0b001))
            .insert(Velocity::linear(acc * 500.0))
            .insert(ColliderMassProperties::Density(5.0))
            .insert(Bullet {
                lifetime: Timer::new(Duration::from_millis(1000), false),
                damage: 25.0,
                has_hit: 0,
            });
    }
}

fn bullet_clean(
    mut commands: Commands,
    time: Res<Time>,
    mut bullet_query: Query<(Entity, &mut Bullet)>,
) {
    for (entity, mut bullet) in &mut bullet_query {
        bullet.lifetime.tick(time.delta());
        if bullet.lifetime.finished() || bullet.has_hit == 2 {
            commands.entity(entity).despawn();
        }
        if bullet.has_hit > 0 {
            bullet.has_hit += 1
        }
    }
}

fn collision_resolve(
    mut collision_events: EventReader<CollisionEvent>,
    mut bullet_query: Query<&mut Bullet>,
    mut enemy_query: Query<&mut Enemy>,
    mut planet_query: Query<&mut Planet>,
) {
    for collision_event in collision_events.iter() {
        if let Started(ent, oth, _) = collision_event {
            if let Ok(mut bullet) = bullet_query.get_mut(*ent) {
                if bullet.has_hit == 0 {
                    if let Ok(mut enemy) = enemy_query.get_mut(*oth) {
                        enemy.hp -= bullet.damage;
                    }
                    bullet.has_hit = 1;
                }
            }
            if let Ok(mut bullet) = bullet_query.get_mut(*oth) {
                if bullet.has_hit == 0 {
                    if let Ok(mut enemy) = enemy_query.get_mut(*ent) {
                        enemy.hp -= bullet.damage;
                    }
                    bullet.has_hit = 1;
                }
            }
            if let Ok(mut enemy) = enemy_query.get_mut(*ent) {
                if enemy.has_hit == 0 {
                    if let Ok(mut planet) = planet_query.get_mut(*oth) {
                        planet.hp -= enemy.damage;
                        enemy.has_hit = 1;
                    }
                }
            }
            if let Ok(mut enemy) = enemy_query.get_mut(*oth) {
                if enemy.has_hit == 0 {
                    if let Ok(mut planet) = planet_query.get_mut(*ent) {
                        planet.hp -= enemy.damage;
                        enemy.has_hit = 1;
                    }
                }
            }
        }
    }
}

fn enemy_clean(mut commands: Commands, life_query: Query<(Entity, &Enemy)>) {
    for (entity, enemy) in &life_query {
        if enemy.hp <= 0.0 || enemy.has_hit > 0 {
            commands.entity(entity).despawn();
        }
    }
}

fn movement(
    time: Res<Time>,
    mut player_query: Query<(&mut Player, &mut Transform), (With<Player>, Without<Planet>)>,
    planet_query: Query<(&Planet, &Transform), (With<Planet>, Without<Player>)>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    let direction = if keyboard_input.pressed(KeyCode::A) {
        1.0
    } else if keyboard_input.pressed(KeyCode::D) {
        -1.0
    } else {
        0.0
    };

    let (player, mut player_trans) = player_query.single_mut();
    let (planet, _planet_trans) = planet_query.single();

    let mut angle_past = Vec2::angle_between(
        Vec2::X,
        Vec2::new(player_trans.translation.x, player_trans.translation.y),
    );
    if angle_past.is_nan() {
        angle_past = 0.0;
    }

    let angle = angle_past + direction * player.speed * (1.0 / planet.size) * time.delta_seconds();

    player_trans.translation = Vec3::new(
        f32::cos(angle) * (planet.size * 0.5 + 4.0),
        f32::sin(angle) * (planet.size * 0.5 + 4.0),
        player_trans.translation.z,
    );
    player_trans.rotation = Quat::from_rotation_z(angle - std::f32::consts::FRAC_PI_2);
}

fn move_enemies(
    time: Res<Time>,
    mut enemies_query: Query<(&mut Enemy, &mut Transform, &mut Velocity)>,
) {
    for (mut enemy, mut enemy_tr, mut rb_vel) in &mut enemies_query {
        if enemy.speed > 0.0 {
            enemy.speed -= time.delta_seconds() * 0.1;
        }

        let delta = Vec2::new(enemy_tr.translation.x, enemy_tr.translation.y);
        let tan = delta.normalize();
        let norm = tan.perp() * enemy.speed;
        rb_vel.linvel -= tan - norm;

        let mut angle = Vec2::angle_between(
            Vec2::X,
            Vec2::new(enemy_tr.translation.x, enemy_tr.translation.y),
        );
        if angle.is_nan() {
            angle = 0.0;
        }
        enemy_tr.rotation = Quat::from_rotation_z(angle);
    }
}
