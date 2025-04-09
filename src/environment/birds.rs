use bevy::prelude::*;
use bevy_rapier3d::prelude::{RigidBody, Velocity};
use rand::{rng, Rng};

use super::map::{HeightMap, MapData};

// marker component for a bird
#[derive(Component)]
pub struct Bird;
// a component that keeps track of the average speed and direction of all birds near a bird
// this is done due to rust aliasing guarantees that dont allow us to calculate this in the boid function
// so we add another function that precalculates this and the boid function then reads it
#[derive(Component)]
pub struct AreaSpeed(Vec3);

// a marker for a flock, flocks are used to keep the birds close to the island. All birds always have some force pulling them to the closest flock
// flocks are fixed position items
#[derive(Component)]
pub struct Flock;


pub fn spawn_birds(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    height_map: Res<HeightMap>,
    map_data: Res<MapData>
) {
    // insert the bird rules, these are things like cohesion factors, they are stored as a resource so it can be changed on the fly
    commands.insert_resource(BirdRules::default());

    let mat = materials.add(Color::linear_rgb(0.0, 0.0, 0.0)); // currently the birds are just black cubes, need to add a mesh here
    let mesh = meshes.add(Cuboid::from_length(0.5));
    let mut rng = rng();
    for j in 0..1 {
        // spawn 1 flock at the centre of the island
        commands.spawn((
            Flock,
            Transform::from_xyz(0.5 * map_data.size, height_map.height_at(0.5, 0.5) * map_data.size + 10.0, 0.5 * map_data.size),
            Mesh3d(meshes.add(Sphere::new(1.0))),
            MeshMaterial3d(materials.add(Color::linear_rgb(0.0, 1.0, 0.0)))
        ));
    }
    for i in 0..100 {
        // spawn 100 birds randomly
        let x = rng.random_range(-5.0 .. 5.0) + 50.0;
        let z = rng.random_range(-5.0 .. 5.0) + 50.0;
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(mat.clone()),
            Transform::from_xyz(
                x, 
                height_map.height_at(x / map_data.size, z / map_data.size) * map_data.size + 10.0, 
                z
            ),
            /*Exhaustion {
                exhaustion: 0.0,
                per_second: 33.0
            },
            Thinker::build()
                .label("exhaustion")
                .picker(FirstToScore{ threshold: 0.8 })
                .when(Tired, Rest {
                    per_second: 50.0,
                    until: 20.0
                })
                */
            Bird,
            AreaSpeed(Vec3::ZERO),
            Velocity {
                angvel: Vec3::ZERO,
                linvel: Vec3::new(
                    rng.random_range(-1.0 .. 1.0), 
                    0.0, 
                    rng.random_range(-1.0 .. 1.0)
                ).normalize_or_zero()
            },
            RigidBody::KinematicVelocityBased // tell the physics sym this is a velocity based item
        ));
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

// TODOS:
// add boid flying with the AI manager so they can land
// make the centering cloud an obloid spheroid instead of a regular sphere
//   this would allow us to make birds live in a semi equal plane whilst allowing bigger horizontal dispertion


#[derive(Resource)]
pub struct BirdRules{
    flock_pos: Vec3,
    seperation: f32,
    allignment: f32,
    cohesion: f32,
    horizontal_flock_cohesion: f32,
    vertical_flock_cohesion: f32
}
impl Default for BirdRules {
    fn default() -> Self {
        Self {
            flock_pos: Vec3::new(0.0, 5.0, 0.0),
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