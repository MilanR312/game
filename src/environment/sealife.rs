use bevy::prelude::*;
use bevy::animation::{
    graph::{AnimationGraph, AnimationGraphHandle, AnimationNodeIndex},
    AnimationPlayer,
};
use rand::{rng, Rng};
use super::map::{HeightMap, MapData};
use crate::mask::{Mask, BlueNoise};

/// Marker component for jellyfish entities.
#[derive(Component)]
pub struct SeaCreature;

#[derive(Component)]
pub struct SeaCreatureAnimationPlayer;

/// Stores the directional influence of nearby creatures.
#[derive(Component)]
pub struct AreaSpeed(pub Vec3);

/// Temporary component for triggering delayed animation playback.
#[derive(Component)]
pub struct SeaCreatureAnimationDelay {
    pub timer: Timer,
}

/// Controls directional drift for jellyfish movement.
#[derive(Component)]
pub struct JellyfishDrift {
    pub direction: Vec3,
    pub speed: f32,
}

/// Jellyfish animation state.
#[derive(Component)]
pub enum JellyfishState {
    Idle,
    Swimming,
}

/// Original base position
#[derive(Component)]
pub struct BasePosition(pub Vec3);

/// Shared asset data for sea creatures.
#[derive(Resource, Clone)]
pub struct SeaCreatureAssets {
    pub scene: Handle<Scene>,
    pub animation_graph: Handle<AnimationGraph>,
    pub animation_nodes: [AnimationNodeIndex; 2], // [idle, swim]
}

/// SandMask-based noise mask for spawn positioning.
#[derive(Resource)]
pub struct SeaMask(pub Mask);

/// Load the sand mask used for spawn noise filtering.
pub fn load_sea_mask(mut commands: Commands) {
    let mask = Mask::from_png("assets/011/SealifeMask.png").expect("Failed to load mask");
    commands.insert_resource(SeaMask(mask));
}

/// Load jellyfish model and animations.
pub fn load_sea_creature_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    let scene = asset_server.load("sealife/jellyfish.glb#Scene0");
    let idle = asset_server.load("sealife/jellyfish.glb#Animation0");
    let swim = asset_server.load("sealife/jellyfish.glb#Animation1");

    let mut graph = AnimationGraph::new();
    let idle_node = graph.add_clip(idle, 1.0, graph.root);
    let swim_node = graph.add_clip(swim, 1.0, graph.root);

    let handle = graphs.add(graph);

    commands.insert_resource(SeaCreatureAssets {
        scene,
        animation_graph: handle,
        animation_nodes: [idle_node, swim_node],
    });
}

/// Spawn jellyfish using blue noise and randomized motion parameters.
pub fn spawn_sea_creatures(
    mut commands: Commands,
    sea_assets: Res<SeaCreatureAssets>,
    height_map: Res<HeightMap>,
    map_data: Res<MapData>,
    mask: Res<SeaMask>,
) {
    let mut rng = rand::thread_rng();
    let mut generator = BlueNoise::new(0.01, mask.0.clone());
    let water_level = 2.0;

    let mut spawned = 0;
    let mut attempts = 0;

    while spawned < 100 && attempts < 500 {
        attempts += 1;

        if let Some([u, v]) = generator.next() {
            let Some(height) = height_map.height_at_checked(u, v) else { continue };
            let x = u * map_data.size;
            let z = v * map_data.size;
            let base_y = height * map_data.size * 2.0;
            let depth = water_level - base_y;

            if depth < 0.4 {
                continue;
            }

            let final_y = base_y + rng.gen_range(0.1..(depth - 0.2).clamp(0.3, 1.2)).min(1.8);
            let pos = Vec3::new(x, final_y, z);

            let direction = Vec3::new(
                rng.gen_range(-1.0..1.0),
                rng.gen_range(-0.2..0.2),
                rng.gen_range(-1.0..1.0),
            ).normalize_or_zero();

            let speed = rng.gen_range(0.08..0.12); 
            let amplitude = rng.gen_range(0.01..0.05);
            let frequency = rng.gen_range(0.2..0.6);
            let phase = rng.gen_range(0.0..(2.0 * std::f32::consts::PI));

            commands.spawn((
                SeaCreature,
                JellyfishDrift { direction, speed },
                AreaSpeed(Vec3::ZERO),
                BasePosition(pos),
                JellyfishState::Swimming, 
                Transform {
                    translation: pos,
                    rotation: Quat::IDENTITY,
                    scale: Vec3::splat(0.2),
                },
                GlobalTransform::default(),
                Visibility::Visible,
            )).with_children(|parent| {
                // model
                parent.spawn((
                    SceneRoot(sea_assets.scene.clone()),
                    Transform {
                    translation: Vec3::ZERO,
                    rotation: Quat::from_rotation_x(std::f32::consts::FRAC_PI_2), 
                    scale: Vec3::ONE,
                },
                    GlobalTransform::default(),
                ));

                // animation
                parent.spawn((
                    AnimationPlayer::default(),
                    AnimationGraphHandle(sea_assets.animation_graph.clone()),
                    SeaCreatureAnimationPlayer,
                    SeaCreatureAnimationDelay {
                        timer: Timer::from_seconds(rng.gen_range(0.0..2.0), TimerMode::Once),
                    },
                ));
            });

            spawned += 1;
        }
    }
}

pub fn update_sealife_animation_delay(
    time: Res<Time>,
    jelly_assets: Res<SeaCreatureAssets>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut SeaCreatureAnimationDelay, &mut AnimationPlayer)>,
) {
    for (entity, mut delay, mut player) in query.iter_mut() {
        delay.timer.tick(time.delta());

        if delay.timer.finished() {

            commands.entity(entity).insert(AnimationGraphHandle(jelly_assets.animation_graph.clone()));

            let node = jelly_assets.animation_nodes[1];
            player.play(node).repeat();

            commands.entity(entity).remove::<SeaCreatureAnimationDelay>();
        }
    }
}

pub fn start_sealife_animations(
    mut commands: Commands,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
) {
    let mut rng = rng();
    for (entity, _player) in players.iter_mut() {
        let delay_seconds = {
            let this = &mut rng;
            let range = 0.0..2.0;
            this.random_range(range)
        };
        let delay_timer = Timer::from_seconds(delay_seconds, TimerMode::Once);
        commands.entity(entity).insert(SeaCreatureAnimationDelay { timer : delay_timer });
    }
}

/// Rotate jellyfish toward direction of motion.
pub fn update_jellyfish_rotation(
    mut query: Query<(&JellyfishDrift, &JellyfishState, &mut Transform)>,
) {
    for (drift, state, mut transform) in query.iter_mut() {
        if let JellyfishState::Swimming = state {
            let dir = drift.direction.normalize_or_zero();
            if dir.length_squared() > 0.001 {
                let base = Quat::from_rotation_arc(Vec3::Z, dir);
                let correction = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2); // was rotation_z!
                transform.rotation = base * correction;
            }
        }
    }
}


/// Compute average direction from nearby creatures.
pub fn calculate_area_speed(
    speeds: Query<(&JellyfishDrift, &Transform), With<SeaCreature>>,
    mut areaspeed: Query<(Entity, &mut AreaSpeed), With<SeaCreature>>,
) {
    const ROI: f32 = 1.5;
    for (entity, mut areaspeed) in areaspeed.iter_mut() {
        let (_, own_pos) = speeds.get(entity).unwrap();
        let (group_velocity, count) = speeds.iter()
            .filter(|(_, t)| t.translation.distance_squared(own_pos.translation) < ROI * ROI)
            .fold((Vec3::ZERO, 0.0), |acc, (drift, _)| {
                (acc.0 + drift.direction, acc.1 + 1.0)
            });

        areaspeed.0 = (group_velocity / count).normalize_or_zero();
    }
}

pub fn jellyfish_drift_system(
    time: Res<Time>,
    height_map: Res<HeightMap>,
    map_data: Res<MapData>,
    mask: Res<SeaMask>,
    mut base_query: Query<&mut BasePosition>,
    mut query: Query<(Entity, &mut JellyfishDrift, &JellyfishState, &mut Transform, &AreaSpeed), With<SeaCreature>>,
) {
    for (entity, mut drift, state, mut transform, area_speed) in query.iter_mut() {
        if let JellyfishState::Swimming = state {
            let pos = transform.translation;
            let u = pos.x / map_data.size;
            let v = pos.z / map_data.size;

            if !(0.0..=1.0).contains(&u) || !(0.0..=1.0).contains(&v) {
                continue;
            }

            let Some(height) = height_map.height_at_checked(u, v) else { continue };
            let seabed = height * map_data.size * 2.0;
            let min_y = seabed + 0.3;
            let max_y = 2.0 - 0.2;

            let Some(mask_value) = mask.0.sample_at_checked(u, v) else { continue };
            if mask_value < 0.05 {
                drift.direction.x *= -1.0;
                drift.direction.z *= -1.0;
            }

            let sample_left = mask.0.sample_at_checked((u - 0.01).clamp(0.0, 1.0), v).unwrap_or(0.0);
            let sample_right = mask.0.sample_at_checked((u + 0.01).clamp(0.0, 1.0), v).unwrap_or(0.0);
            let sample_down = mask.0.sample_at_checked(u, (v - 0.01).clamp(0.0, 1.0)).unwrap_or(0.0);
            let sample_up = mask.0.sample_at_checked(u, (v + 0.01).clamp(0.0, 1.0)).unwrap_or(0.0);

            let mask_gradient = Vec3::new(sample_right - sample_left, 0.0, sample_up - sample_down);
            drift.direction += mask_gradient.normalize_or_zero() * 0.05;

            let current_dir = drift.direction;
            let align = area_speed.0 - current_dir;
            drift.direction += align * 0.05;            
            drift.direction = drift.direction.normalize_or_zero();

            if pos.y < min_y && drift.direction.y < 0.0 {
                drift.direction.y *= -1.0;
            }
            if pos.y > max_y && drift.direction.y > 0.0 {
                drift.direction.y *= -1.0;
            }

            let forward = transform.rotation * Vec3::Z;
            if let Ok(mut base) = base_query.get_mut(entity) {
                base.0 = transform.translation;
            }
            transform.translation += forward * drift.speed * time.delta_secs();
        }
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
                start_sealife_animations,
                update_sealife_animation_delay,
            ))
            .add_systems(Update, (
                calculate_area_speed,
                jellyfish_drift_system,
                update_jellyfish_rotation,
            ).chain());
    }
}
