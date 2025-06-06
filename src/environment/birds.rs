use bevy::prelude::*;
use bevy::animation::{AnimationPlayer, graph::{AnimationGraph, AnimationGraphHandle, AnimationNodeIndex}};
use bevy_rapier3d::prelude::{RigidBody, Velocity};
use rand::{rng, Rng};
use bevy::time::TimerMode;

use super::map::{HeightMap, MapData};

#[derive(Component)]
pub struct Bird;

#[derive(Component)]
pub struct BirdAnimationPlayer;

#[derive(Component)]
pub struct BirdAnimationDelay {
    timer: Timer,
}

#[derive(Resource, Clone)]
pub struct BirdAssets {
    pub scene_bird: Handle<Scene>,
    pub animation_graph_bird: Handle<AnimationGraph>,
    pub animation_nodes_bird: [AnimationNodeIndex; 1],
}

#[derive(Component)]
pub struct AreaSpeed(Vec3);

#[derive(Component)]
pub struct Flock;

pub fn load_bird_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    let scene_bird = asset_server.load("bird/bird.glb#Scene0");
    let fly_bird = asset_server.load("bird/bird.glb#Animation0");

    let mut graph_bird = AnimationGraph::new();

    let fly_node = graph_bird.add_clip(fly_bird, 1.0, graph_bird.root);

    let graph_handle_bird = graphs.add(graph_bird);

    commands.insert_resource(BirdAssets {
        scene_bird,
        animation_graph_bird: graph_handle_bird,
        animation_nodes_bird: [fly_node],
    });
}

pub fn spawn_birds(
    mut commands: Commands,
    bird_assets: Res<BirdAssets>,
    //mut meshes: ResMut<Assets<Mesh>>,
    //mut materials: ResMut<Assets<StandardMaterial>>,
    height_map: Res<HeightMap>,
    map_data: Res<MapData>,
) {
    commands.insert_resource(BirdRules::default());

    let mut rng = rand::rng();

    // Spawn the flock entity
    commands.spawn((
        Flock,
        Transform::from_xyz(
            0.5 * map_data.size,
            height_map.height_at(0.5, 0.5) * map_data.size + 10.0,
            0.5 * map_data.size,
        ),
        //Mesh3d(meshes.add(Sphere::new(1.0))),
        //MeshMaterial3d(materials.add(Color::linear_rgb(0.0, 1.0, 0.0))),
    ));

    // Spawn birds
    for _ in 0..100 {
        let x = rng.random_range(-5.0..5.0) + 50.0;
        let z = rng.random_range(-5.0..5.0) + 50.0;
        let y = height_map.height_at(x / map_data.size, z / map_data.size) * map_data.size + 10.0;

        commands.spawn((
            Bird,
            AreaSpeed(Vec3::ZERO),
            Velocity {
                angvel: Vec3::ZERO,
                linvel: Vec3::new(
                    {
                        let this = &mut rng;
                        let range = -1.0..1.0;
                        this.random_range(range)
                    },
                    0.0,
                    {
                        let this = &mut rng;
                        let range = -1.0..1.0;
                        this.random_range(range)
                    },
                ).normalize_or_zero(),
            },
            RigidBody::KinematicVelocityBased,
            Transform::from_xyz(x, y, z).with_scale(Vec3::splat(0.1)),
            GlobalTransform::default(),
            Visibility::Visible,
        ))
        .with_children(|parent| {
            // Scene
            parent.spawn((
                SceneRoot(bird_assets.scene_bird.clone()),
                Transform::default(),
                GlobalTransform::default(),
            ));

            // Separate animation player
            parent.spawn((
                AnimationPlayer::default(),
                AnimationGraphHandle(bird_assets.animation_graph_bird.clone()),
                BirdAnimationPlayer,
                BirdAnimationDelay {
                    timer: Timer::from_seconds(0.2, TimerMode::Once),
                },
            ));
        });
    }
}

pub fn update_bird_animation_delay(
    time: Res<Time>,
    bird_assets: Res<BirdAssets>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut BirdAnimationDelay, &mut AnimationPlayer)>,
) {
    for (entity, mut delay, mut player) in query.iter_mut() {
        delay.timer.tick(time.delta());

        if delay.timer.finished() {
            info!("[Bird] Timer finished for entity {entity:?}, initializing animation.");

            commands.entity(entity).insert(AnimationGraphHandle(bird_assets.animation_graph_bird.clone()));
            info!("[Bird] AnimationGraphHandle assigned to entity {entity:?}");

            let node = bird_assets.animation_nodes_bird[0];
            player.play(node).repeat();
            info!("[Bird] Fly animation (node {node:?}) started on entity {entity:?}");

            commands.entity(entity).remove::<BirdAnimationDelay>();
            info!("[Bird] BirdAnimationDelay removed from entity {entity:?}");
        }
    }
}

pub fn start_bird_animations(
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
        commands.entity(entity).insert(BirdAnimationDelay { timer : delay_timer });
    }
}

pub fn update_bird_rotation(
    mut birds: Query<(&Velocity, &mut Transform), With<Bird>>,
) {
    for (velocity, mut transform) in birds.iter_mut() {
        let linvel = velocity.linvel;

        if linvel.length_squared() > 0.01 {
            let forward = linvel.normalize();
            let base_rotation = Quat::from_rotation_arc(Vec3::Z, forward);
            let correction = Quat::from_rotation_y(std::f32::consts::PI);
            transform.rotation = base_rotation * correction;
        }
    }
}

const ROI: f32 = 1.5;
const ROA: f32 = 0.7;
// calculate the area speed of the birds
pub fn calculate_area_speed(
    speeds: Query<(&Velocity, &Transform), With<Bird>>, // for every bird we query the speed and position
    mut areaspeed: Query<(Entity, &mut AreaSpeed), With<Bird>> // and also query a mutable reference to the speed and an entity key
){
    for (entity, mut areaspeed) in areaspeed.iter_mut() { // iterate over all the speeds and bird keys
        let (_, own_pos) = speeds.get(entity).unwrap(); // get the speed and location for the bird key

        let (group_velocity, count) = speeds.iter() // for every bird in the query
            .filter(|x| x.1.translation.distance_squared(own_pos.translation) < ROI * ROI)// if they are within the region of influence
            .fold((Vec3::splat(0.0), 0.0), |acc, ell|{ // calculate the average speed
                (acc.0 + ell.0.linvel, acc.1 + 1.0)
            });
        
        *areaspeed = AreaSpeed((group_velocity / count).normalize_or_zero()); // and save it
    }
}

#[derive(Resource)]
pub struct BirdRules{
    //flock_pos: Vec3,
    seperation: f32,
    allignment: f32,
    cohesion: f32,
    horizontal_flock_cohesion: f32,
    vertical_flock_cohesion: f32
}
impl Default for BirdRules {
    fn default() -> Self {
        Self {
            //flock_pos: Vec3::new(0.0, 5.0, 0.0),
            seperation: 0.05,
            allignment: 0.05,
            cohesion: 0.0005,
            horizontal_flock_cohesion: 0.0001,
            vertical_flock_cohesion: 0.001,
        }
    }
}

pub fn bird_boids(
    mut query: Query<(Entity, &mut Velocity, &AreaSpeed), With<Bird>>, // Query the key, speed, and average speed
    flocks: Query<&Transform, With<Flock>>,
    ui_state: Res<BirdRules>,
    transforms: Query<&Transform, With<Bird>>, // and a query for the position
) {
    for (entity, mut velocity, areaspeed) in query.iter_mut() {
        let position = transforms.get(entity).unwrap(); // get the current birds position

        // seperation rule
        let seperation = transforms.iter() // for every position
            .filter(|x| *x != position) // except our bird
            .filter(|x| x.translation.distance_squared(position.translation) < ROA * ROA) // and if its within the region of avoidance
            .fold(Vec3::splat(0.0), |acc, point| { // calculate the influence
                acc + ( position.translation - point.translation )
            });
        velocity.linvel += seperation * ui_state.seperation; 
        

        // allignment
        let allignment = areaspeed.0 - velocity.linvel; // use the precalculated area speed
        velocity.linvel += allignment * ui_state.allignment;
        
        // cohesion step
        let (pos_sum, count) = transforms.iter()
            .filter(|x| x.translation.distance_squared(position.translation) < ROI * ROI) 
            .fold((Vec3::splat(0.0), 0.0), |acc, ell|{
                (acc.0 + ell.translation, acc.1+1.0)
            });
        let average_pos = pos_sum / count;
        let cohesion = (average_pos - position.translation).normalize_or_zero();
        velocity.linvel += cohesion * ui_state.cohesion;

        let closest_flock = flocks.iter()
            .min_by(|a,b| {
                let a = a.translation.distance_squared(position.translation);
                let b = b.translation.distance_squared(position.translation);
                a.total_cmp(&b)
            })
            .map(|x| x.translation)
            .expect("there must always be a flock registered");
        // get a vector point to the flock
        let flock_cohesion = closest_flock - position.translation;
        velocity.linvel += flock_cohesion * Vec3::new(ui_state.horizontal_flock_cohesion, ui_state.vertical_flock_cohesion, ui_state.horizontal_flock_cohesion);

        velocity.linvel = velocity.linvel.normalize() * 1.5;
    }
}

pub struct BirdPlugin;

impl Plugin for BirdPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(BirdRules::default())
            .add_systems(Startup, (
                load_bird_assets,
                spawn_birds.after(load_bird_assets),
            ))
            .add_systems(Update, (
                start_bird_animations,
                update_bird_animation_delay,
            ))
            .add_systems(Update, (
                update_bird_rotation,
                calculate_area_speed,
                bird_boids,
            ).chain());
    }
}
