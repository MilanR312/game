use bevy::prelude::*;
use bevy::animation::{AnimationPlayer, graph::{AnimationGraph, AnimationGraphHandle, AnimationNodeIndex}};
use bevy_rapier3d::prelude::*;
use rand::Rng;

use super::map::{HeightMap, MapData};
use crate::mask::{Mask, BlueNoise};

#[derive(Component)]
pub struct SeaCreature;

#[derive(Component)]
pub struct SeaCreatureAnimationPlayer;

#[derive(Component)]
pub struct SeaCreatureAnimationDelay {
    pub timer: Timer,
}

#[derive(Resource, Clone)]
pub struct SeaCreatureAssets {
    pub scene: Handle<Scene>,
    pub animation_graph: Handle<AnimationGraph>,
    pub animation_nodes: [AnimationNodeIndex; 2],
}

#[derive(Component)]
pub struct AreaSpeed(pub Vec3);

#[derive(Component)]
pub struct JellyfishDrift {
    pub direction: Vec3,
    pub speed: f32,
}

#[derive(Component)]
pub struct VerticalOscillation {
    pub amplitude: f32,
    pub frequency: f32,
    pub phase: f32,
}

#[derive(Component)]
pub struct BasePosition(pub Vec3);

#[derive(Resource)]
pub struct SeaMask(Mask);

pub fn load_sea_mask(mut commands: Commands) {
    let mask = Mask::from_png("assets/011/SandMask.png").expect("Failed to load SandMask.png");
    commands.insert_resource(SeaMask(mask));
}

pub fn load_sea_creature_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    let scene = asset_server.load("sealife/jellyfish.glb#Scene0");
    let idle = asset_server.load("sealife/jellyfish.glb#Animation0"); // idle
    let swim = asset_server.load("sealife/jellyfish.glb#Animation1"); // swimming

    let mut graph = AnimationGraph::new();
    let idle_node = graph.add_clip(idle, 1.0, graph.root);
    let swim_node = graph.add_clip(swim, 1.0, graph.root);

    let handle = graphs.add(graph);

    commands.insert_resource(SeaCreatureAssets {
        scene,
        animation_graph: handle,
        animation_nodes: [idle_node, swim_node],
    });
    info!("Loaded sea creature assets");
}

pub fn spawn_sea_creatures(
    mut commands: Commands,
    sea_assets: Res<SeaCreatureAssets>,
    height_map: Res<HeightMap>,
    map_data: Res<MapData>,
    mask: Res<SeaMask>,
    asset_server: Res<AssetServer>,
) {
    let mut rng = rand::thread_rng();
    let mut generator = BlueNoise::new(0.01, mask.0.clone());
    let water_level = 2.0;

    let mut count = 0;
    let mut attempted = 0;
    let scale = 0.2;

    for [u, v] in generator {
        attempted += 1;

        let Some(height) = height_map.height_at_checked(u, v) else {
            warn!("Skipping: no height at ({u}, {v})");
            continue;
        };

        let x = u * map_data.size;
        let z = v * map_data.size;
        let base_y = height * map_data.size * 2.0;
        let depth = water_level - base_y;

        if depth < 0.4 {
            warn!("Skipping: too shallow at ({x:.2}, {z:.2}), depth={depth:.2}");
            continue;
        }

        let max_offset = (depth - 0.2).clamp(0.3, 1.2);
        let final_y = (base_y + rng.gen_range(0.1..max_offset)).min(1.8);

        let amplitude = rng.gen_range(0.05..0.2);
        let frequency = rng.gen_range(0.2..0.6);
        let phase = rng.gen_range(0.0..(2.0 * std::f32::consts::PI));

        let pos = Vec3::new(x, final_y, z);

        let horizontal = Vec3::new(
            rng.gen_range(-1.0..1.0),
            0.0,
            rng.gen_range(-1.0..1.0),
        );
        let vertical = Vec3::Y * rng.gen_range(-0.2..0.2);
        let direction = (horizontal + vertical).normalize_or_zero();
        let speed = rng.gen_range(0.03..0.06);

        let mut entity = commands.spawn(SceneBundle {
            scene: SceneRoot(sea_assets.scene.clone()),
            transform: Transform::from_translation(pos).with_scale(Vec3::splat(scale)),
            ..default()
        });

        entity.insert((
            SeaCreature,
            JellyfishDrift { direction, speed },
            AreaSpeed(Vec3::ZERO),
            VerticalOscillation { amplitude, frequency, phase },
            BasePosition(pos),
            RigidBody::Dynamic,
            Velocity::default(),
            LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z,
            Damping {
                linear_damping: 0.1,
                angular_damping: 0.1,
            },
            Sleeping::default(),
            AnimationPlayer::default(),
            AnimationGraphHandle(sea_assets.animation_graph.clone()),
            SeaCreatureAnimationPlayer,
            SeaCreatureAnimationDelay {
                timer: Timer::from_seconds(rng.gen_range(0.0..2.0), TimerMode::Once),
            },
            Visibility::Visible,
        ));

        let entity_id = entity.id();
        info!("Spawned jellyfish entity: {entity_id:?} at pos=({x:.2}, {final_y:.2}, {z:.2})");

        count += 1;
        if count >= 50 {
            break;
        }
    }

    info!("Spawned {count} jellyfish out of {attempted} attempts");
}


pub fn start_sea_animations(
    mut commands: Commands,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
) {
    let mut rng = rand::thread_rng();

    for (entity, _player) in players.iter_mut() {
        let delay = rng.gen_range(0.0..2.0);
        let timer = Timer::from_seconds(delay, TimerMode::Once);
        commands.entity(entity).insert(SeaCreatureAnimationDelay { timer });
    }
}

pub fn update_sea_animation_delay(
    time: Res<Time>,
    sea_assets: Res<SeaCreatureAssets>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut SeaCreatureAnimationDelay, &mut AnimationPlayer), With<SeaCreatureAnimationPlayer>>,
) {
    for (entity, mut delay, mut player) in query.iter_mut() {
        delay.timer.tick(time.delta());

        if delay.timer.finished() {
            info!("[Sea] Timer finished for entity {entity:?}, initializing animation.");

            commands.entity(entity).insert(AnimationGraphHandle(sea_assets.animation_graph.clone()));
            info!("[Sea] AnimationGraphHandle assigned to entity {entity:?}");
            info!("[Animation] Starting swim animation for {:?}", entity);      
            let node = sea_assets.animation_nodes[0]; // swimming
            player.play(node).repeat();
            info!("[Sea] Swim animation (node {node:?}) started on entity {entity:?}");

            commands.entity(entity).remove::<SeaCreatureAnimationDelay>();
            info!("[Sea] SeaAnimationDelay removed from entity {entity:?}");
        }
    }
}

pub fn jellyfish_vertical_oscillation(
    time: Res<Time>,
    mut query: Query<(&VerticalOscillation, &BasePosition, &mut Transform)>,
) {
    for (osc, base, mut transform) in query.iter_mut() {
        let t = time.elapsed().as_secs_f32();
        let offset = osc.amplitude * (2.0 * std::f32::consts::PI * osc.frequency * t + osc.phase).sin();
        let local_up = transform.rotation.mul_vec3(Vec3::Y);
        transform.translation = base.0 + local_up * offset;
    }
}

pub fn jellyfish_bounded_drift(
    height_map: Res<HeightMap>,
    map_data: Res<MapData>,
    mut set: ParamSet<(
        Query<(&mut JellyfishDrift, &mut Velocity, &Transform, &AreaSpeed), With<SeaCreature>>,
        Query<&Transform, With<SeaCreature>>,
    )>,
) {
    let all_positions: Vec<Vec3> = set.p1().iter().map(|t| t.translation).collect();

    for (mut drift, mut velocity, transform, area_speed) in set.p0().iter_mut() {
        let pos = transform.translation;
        let u = pos.x / map_data.size;
        let v = pos.z / map_data.size;

        if !(0.0..=1.0).contains(&u) || !(0.0..=1.0).contains(&v) {
            continue;
        }

        let Some(raw) = height_map.height_at_checked(u, v) else { continue; };
        let seabed = raw * map_data.size * 2.0;
        let min_y = seabed + 0.3;
        let max_y = 2.0 - 0.2;

        let align = area_speed.0 - drift.direction;
        drift.direction += align * 0.05;

        let separation = all_positions.iter()
            .filter(|x| **x != pos)
            .filter(|x| x.distance_squared(pos) < 0.4 * 0.4)
            .fold(Vec3::ZERO, |acc, x| acc + (pos - *x));
        drift.direction += separation.normalize_or_zero() * 0.1;

        drift.direction = drift.direction.normalize_or_zero();

        if pos.y < min_y && drift.direction.y < 0.0 {
            drift.direction.y *= -1.0;
        }
        if pos.y > max_y && drift.direction.y > 0.0 {
            drift.direction.y *= -1.0;
        }

        velocity.linvel = drift.direction * drift.speed + Vec3::Y * velocity.linvel.y;
    }
}

pub fn update_jellyfish_rotation(
    mut query: Query<(&Velocity, &mut Transform), With<SeaCreature>>,
) {
    for (velocity, mut transform) in query.iter_mut() {
        let linvel = velocity.linvel;
        let flat_velocity = Vec3::new(linvel.x, 0.0, linvel.z);

        if flat_velocity.length_squared() > 0.01 {
            let forward = flat_velocity.normalize();
            let desired_rot = Quat::from_rotation_y(forward.x.atan2(forward.z) + std::f32::consts::FRAC_PI_2);
            transform.rotation = transform.rotation.slerp(desired_rot, 0.1);
        }
    }
}

pub fn calculate_area_speed(
    speeds: Query<(&JellyfishDrift, &Transform), With<SeaCreature>>,
    mut areaspeed: Query<(Entity, &mut AreaSpeed), With<SeaCreature>>,
) {
    const ROI: f32 = 1.5;
    for (entity, mut areaspeed) in areaspeed.iter_mut() {
        let (_, own_pos) = speeds.get(entity).unwrap();
        let (group_velocity, count) = speeds.iter()
            .filter(|(_, transform)| transform.translation.distance_squared(own_pos.translation) < ROI * ROI)
            .fold((Vec3::ZERO, 0.0), |acc, (drift, _)| {
                (acc.0 + drift.direction, acc.1 + 1.0)
            });

        areaspeed.0 = (group_velocity / count).normalize_or_zero();
    }
}

pub struct SeaLifePlugin;

impl Plugin for SeaLifePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, (
                load_sea_mask,
                load_sea_creature_assets.after(load_sea_mask),
                spawn_sea_creatures.after(load_sea_creature_assets),
            ))
            .add_systems(Update, (
                start_sea_animations,
                update_sea_animation_delay,
                calculate_area_speed,
                jellyfish_vertical_oscillation,
                update_jellyfish_rotation,
                jellyfish_bounded_drift,
            ));
    }
}
