use bevy::prelude::*;
use bevy::animation::{AnimationPlayer, graph::{AnimationGraph, AnimationGraphHandle, AnimationNodeIndex}};
use rand::{rng, Rng};
use bevy_rapier3d::prelude::*;

use crate::environment::grass::BushMask;
use super::map::{HeightMap, MapData};
use crate::mask::BlueNoise;

#[derive(Component)]
pub struct Fox;

#[derive(Component)]
pub struct FoxAnimationPlayer;

#[derive(Component)]
pub struct FoxAnimationDelay {
    pub timer: Timer,
}

#[derive(Resource, Clone)]
pub struct FoxAssets {
    pub scene_fox: Handle<Scene>,
    pub animation_graph_fox: Handle<AnimationGraph>,
    pub animation_nodes_fox: [AnimationNodeIndex; 3], // [idle, walk, run]
}

#[derive(Component)]
pub struct WanderTarget(pub Vec3);

pub fn load_fox_assets(
    mut commands: Commands,
    asset_server_fox: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    let scene_fox = asset_server_fox.load("fox/Fox.glb#Scene0");
    let idle_fox = asset_server_fox.load("fox/Fox.glb#Animation0");
    let walk_fox = asset_server_fox.load("fox/Fox.glb#Animation1");
    let run_fox  = asset_server_fox.load("fox/Fox.glb#Animation2");

    let mut graph_fox = AnimationGraph::new();

    let idle_node = graph_fox.add_clip(idle_fox, 1.0, graph_fox.root);
    let walk_node = graph_fox.add_clip(walk_fox, 1.0, graph_fox.root);
    let run_node  = graph_fox.add_clip(run_fox, 1.0, graph_fox.root);


    let graph_handle_fox = graphs.add(graph_fox);

    commands.insert_resource(FoxAssets {
        scene_fox,
        animation_graph_fox: graph_handle_fox,
        animation_nodes_fox: [idle_node, walk_node, run_node],
    });
}

pub fn spawn_foxes(
    mut commands: Commands,
    fox_assets: Res<FoxAssets>,
    height_map: Res<HeightMap>,
    map_data: Res<MapData>,
    mask: Res<BushMask>,
) {
    let noise = BlueNoise::new(0.02, mask.0.clone());
    let mut rng = rng();

    for [u, v] in noise.take(15) {
        let Some(height) = height_map.height_at_checked(u, v) else { continue; };

        let x = u * map_data.size;
        let z = v * map_data.size;
        let terrain_y = height * map_data.size * 2.0;
        let scale = 0.005;
        let capsule_half_height = 0.1;
        let spawn_y = terrain_y + capsule_half_height / scale;

        let pos = Vec3::new(x, spawn_y, z);
        let wander_target = Vec3::new(
            x + {
                let this = &mut rng;
                let range = -5.0..5.0;
                this.random_range(range)
            },
            terrain_y,
            z + {
                let this = &mut rng;
                let range = -5.0..5.0;
                this.random_range(range)
            },
        );

        commands.spawn((
            Fox,
            WanderTarget(wander_target),
            RigidBody::Dynamic,
            Velocity::default(),
            Collider::capsule_y(capsule_half_height, 0.06),
            LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Y | LockedAxes::ROTATION_LOCKED_Z,
            Friction::coefficient(0.3),
            Ccd::enabled(),
            Damping { linear_damping: 0.1, angular_damping: 0.1 },
            Sleeping::default(),
            Transform::from_translation(pos),
            GlobalTransform::default(),
            Visibility::Visible,
        ))
        .with_children(|parent| {
            
            // Spawn Scene
            parent.spawn((
                SceneRoot(fox_assets.scene_fox.clone()),
                Transform {
                    translation: Vec3::ZERO,
                    rotation: Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2),
                    scale: Vec3::splat(scale),
                },
                GlobalTransform::default(),
                Visibility::Visible,
            ));

            // Separate animation player
            parent.spawn((
                AnimationPlayer::default(),
                AnimationGraphHandle(fox_assets.animation_graph_fox.clone()),
                FoxAnimationPlayer,
                FoxAnimationDelay {
                    timer: Timer::from_seconds(0.2, TimerMode::Once),
                },
            ));
        });
    }
}

pub fn update_fox_animation_delay(
    time: Res<Time>,
    fox_assets: Res<FoxAssets>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut FoxAnimationDelay, &mut AnimationPlayer)>,
) {
    for (entity, mut delay, mut player) in query.iter_mut() {
        delay.timer.tick(time.delta());

        if delay.timer.finished() {
            info!("[Fox] Timer finished for entity {entity:?}, initializing animation.");

            commands.entity(entity).insert(AnimationGraphHandle(fox_assets.animation_graph_fox.clone()));
            info!("[Fox] AnimationGraphHandle assigned to entity {entity:?}");

            let node = fox_assets.animation_nodes_fox[2];
            player.play(node).repeat();
            info!("[Fox] Walk animation (node {node:?}) started on entity {entity:?}");

            commands.entity(entity).remove::<FoxAnimationDelay>();
            info!("[Fox] FoxAnimationDelay removed from entity {entity:?}");
        }
    }
}


pub fn start_fox_animations(
    mut commands: Commands,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
) {
    for (entity, _player) in players.iter_mut() {
        let timer = Timer::from_seconds(0.2, TimerMode::Once);
        commands.entity(entity).insert(FoxAnimationDelay { timer });
    }
}

pub fn fox_wander_system(
    mut foxes: Query<(&Transform, &mut WanderTarget, &mut Velocity), With<Fox>>,
) {
    let mut rng = rng();
    let min_y = 2.0;

    for (transform, mut target, mut velocity) in foxes.iter_mut() {
        let current = transform.translation;
        let goal = Vec3::new(target.0.x, current.y, target.0.z);
        let dir = (goal - current).normalize_or_zero();

        // Base Speed
        let base_speed = 1.2;
        velocity.linvel = dir * base_speed + Vec3::Y * velocity.linvel.y;

        // New goal
        if current.distance_squared(goal) < 0.6 {
            let offset = Vec3::new(
                {
                    let this = &mut rng;
                    let range = 3.0..6.0;
                    this.random_range(range)
                },
                0.0,
                {
                    let this = &mut rng;
                    let range = 3.0..6.0;
                    this.random_range(range)
                },
            ) * Vec3::new(
                if {
                    let this = &mut rng;
                    this.random_bool(0.5)
                } { 1.0 } else { -1.0 },
                0.0,
                if {
                    let this = &mut rng;
                    this.random_bool(0.5)
                } { 1.0 } else { -1.0 },
            );

            let mut new_target = current + offset;

            // Limit new y (sealevel)
            if new_target.y < min_y {
                new_target.y = min_y;
            }

            target.0 = new_target;
        }
    }
}

pub fn update_fox_rotation(
    mut query: Query<(&Velocity, &mut Transform), With<Fox>>,
) {
    for (velocity, mut transform) in query.iter_mut() {
        let linvel = velocity.linvel;
        let flat_velocity = Vec3::new(linvel.x, 0.0, linvel.z);

        if flat_velocity.length_squared() > 0.01 {
            let forward = flat_velocity.normalize();
            let desired_dir = Quat::from_rotation_y(forward.x.atan2(forward.z) + std::f32::consts::FRAC_PI_2);
            transform.rotation = transform.rotation.slerp(desired_dir, 0.1);
        }
    }
}



pub struct FoxPlugin;

impl Plugin for FoxPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, (
                load_fox_assets.after(super::map::load_map),
                spawn_foxes.after(super::grass::init_mask).after(load_fox_assets),
            ))
            .add_systems(Update, (
                fox_wander_system,
                update_fox_rotation,
                update_fox_animation_delay,
                start_fox_animations,
            ));
    }
}
