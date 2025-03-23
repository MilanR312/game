
use std::{fs::File, io::Read, path::Path};

use bevy::{asset::{io::Reader, AssetLoader, LoadContext, LoadState, RenderAssetUsages}, diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin, SystemInformationDiagnosticsPlugin}, gltf::GltfMesh, image::ImageLoader, pbr::{experimental::meshlet::MeshletPlugin, wireframe::{Wireframe, WireframeConfig, WireframePlugin}}, prelude::*, render::{batching::no_gpu_preprocessing::BatchedInstanceBuffer, diagnostic::RenderDiagnosticsPlugin, mesh::PrimitiveTopology, settings::{RenderCreation, WgpuFeatures, WgpuSettings}, RenderPlugin}, utils::hashbrown::HashMap};
use bevy_obj::ObjPlugin;
use iyes_perf_ui::{entries::{PerfUiFixedTimeEntries, PerfUiFramerateEntries, PerfUiSystemEntries}, prelude::{PerfUiAllEntries, PerfUiDefaultEntries}, PerfUiPlugin};
use noise::NoiseFn;
use rand::{rng, rngs::ThreadRng, Rng};
use serde::{Deserialize, Serialize};
use smooth_bevy_cameras::{controllers::unreal::{UnrealCameraBundle, UnrealCameraController, UnrealCameraPlugin}, LookTransform, LookTransformPlugin};
use spade::{DelaunayTriangulation, Triangulation};

use bevy_rapier3d::prelude::*;
use thiserror::Error;

#[derive(Component)]
struct Bird;
#[derive(Component)]
struct AreaSpeed(Vec3);


fn spawn_birds(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    let bird_size = 0.05;
    let animal_mesh = meshes.add(Cuboid::new(bird_size, bird_size, bird_size));
    let animal_material = materials.add(Color::WHITE);
    let mut rng = rng();
    let animal_count = 100;
    for _ in 0..animal_count {
        commands.spawn((
            MeshMaterial3d(animal_material.clone()),
            Mesh3d(animal_mesh.clone()),
            Transform::from_xyz( // spawn dier op random plaats
                rng.random_range(-1.0 .. 1.0), 
                rng.random_range(-1.0 .. 1.0), 
                rng.random_range(-1.0 .. 1.0)
            ),
            Velocity{ // met een random snelheid
                linvel: Vec3::new(
                    rng.random_range(-1.0 .. 1.0), 
                    rng.random_range(-1.0 .. 1.0), 
                    rng.random_range(-1.0 .. 1.0)
                ).normalize(),
                angvel: Vec3::splat(0.0)
            },
            AreaSpeed(Vec3::splat(0.0)),
            RigidBody::KinematicVelocityBased, // en zeg tegen rapier dat we het met snelheid controlleren
            Bird // en dat het een vogel is tegen bevy
        ));
    }
}


const ROI: f32 = 0.3;
const ROA: f32 = 0.1;

// door mutability redenen doen we eerst een apparte berekening waar we de gemiddelde snelheid per groep berekenen
// dit maakt de code wat simpeler
fn calculate_area_speed(
    speeds: Query<(&Velocity, &Transform), With<Bird>>, // we vragen voor elke vogel de snelheid en positie
    mut areaspeed: Query<(Entity, &mut AreaSpeed), With<Bird>> // en appart ook elke gemiddelde snelheid + een sleutel dat elk object uniek identificieerd
){
    for (entity, mut areaspeed) in areaspeed.iter_mut() { // we gaan dan over elke gemiddelde snelheid (en ook vogel)
        let (_, own_pos) = speeds.get(entity).unwrap(); // en pakken dan de positie op in de andere query met de unieke sleutel

        let (group_velocity, count) = speeds.iter() // voor elke positie en snelheid
            .filter(|x| x.1.translation.distance_squared(own_pos.translation) < ROI * ROI)// behalve als ze buiten de region of influence liggen
            .fold((Vec3::splat(0.0), 0.0), |acc, ell|{ // berekenen we de som van de snelheiden + aantal snelheiden
                (acc.0 + ell.0.linvel, acc.1 + 1.0)
            });
        
        *areaspeed = AreaSpeed(group_velocity / count); // en steken we dit in de gemiddelde snelheid
    }
}
fn bird_boids(
    mut query: Query<(Entity, &mut Velocity, &AreaSpeed), With<Bird>>, // hier vragen we dan terug de sleutel op, de snelheid en de gemiddelde snelheid voor elke vogel
    transforms: Query<&Transform, With<Bird>>, // en in een apparte query de positie van elke vogel
) {
    for (entity, mut velocity, areaspeed) in query.iter_mut() {
        let position = transforms.get(entity).unwrap(); // we pakken de momentele vogel zn positie

        // seperation regel
        let closest = transforms.iter() // voor elke positie
            .filter(|x| *x != position) // behalve onze vogel zn positie
            .filter(|x| x.translation.distance_squared(position.translation) < ROA * ROA) // en elke vogel dat buiten de region of avoidance ligt
            .fold(Vec3::splat(0.0), |acc, point| { // berekenen we de som van de afstanden (dit kan ook met een for loop hoor :) )
                acc + ( position.translation - point.translation )
            });
        velocity.linvel += closest * 0.05; // en veranderen we de snelheid met deze seperation
        

        // allignment
        let delta = areaspeed.0 - velocity.linvel; // voor allignment gebruiken we de areaspeed dat we vorige stap berekend hebben
        velocity.linvel += delta * 0.05;
        
        // cohesion step
        let (pos_sum, count) = transforms.iter() // dan terug voor elke positie
            .filter(|x| x.translation.distance_squared(position.translation) < ROI * ROI) // behalve deze buiten de region of influence
            .fold((Vec3::splat(0.0), 0.0), |acc, ell|{ // nemen we de gemiddelde positie
                (acc.0 + ell.forward().as_vec3(), acc.1+1.0)
            });
        let average_pos = pos_sum / count;
        let cohesion = (average_pos - position.translation).normalize_or_zero();
        velocity.linvel += cohesion * 0.0005;
    }
}


#[derive(Resource)]
struct HeightMap(Vec<f32>, usize);
impl HeightMap {
    pub fn from_file<T: AsRef<Path>>(path: T, size: usize) -> std::io::Result<Self> {
        let mut file = File::open(path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        let data: &[f32] = bytemuck::cast_slice(data.as_slice());
        Ok(HeightMap(data.to_owned(), size))
    }
    pub fn height_at(&self, x: f32, y: f32) -> f32 {
        let width = self.1 as f32;
        let x = (x * width) as usize;
        let y = (y * width) as usize;
        self.0[y * self.1 + x]
    }
}

struct Mask(Vec<f32>, usize);
impl Mask {
    pub fn from_file<T: AsRef<Path>>(path: T, size: usize) -> std::io::Result<Self> {
        let mut file = File::open(path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        let data: &[f32] = bytemuck::cast_slice(data.as_slice());
        Ok(Self(data.to_owned(), size))
    }
    pub fn sample_at(&self, x: f32, y: f32) -> f32 {
        let width = self.1 as f32;
        let ix = (x * width) as usize;
        let iy = (y * width) as usize;
        debug_assert!(iy * self.1 + ix < self.0.len(), "index ({x},{y}) out of bounds ({ix}, {iy})->({})", iy * self.1 + ix);
        self.0[iy * self.1 + ix]
    }
}
pub struct BlueNoise<'a>(fast_poisson::Iter<2>, &'a Mask, ThreadRng);
impl<'a> BlueNoise<'a>{
    pub fn new(distance: f32, mask: &'a Mask) -> Self {
        let iter = fast_poisson::Poisson::<2>::new()
            .with_dimensions([1.0, 1.0], distance as f64);   
        Self(iter.iter(), mask, rng())
    }
}
impl Iterator for BlueNoise<'_> {
    type Item = [f32; 2];
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let x = self.0.next()?;
            let sample_point = self.1.sample_at(x[0] as f32, x[1] as f32);
            if self.2.random_range(0.00 .. 1.0) < sample_point {
                return Some([x[0] as f32, x[1] as f32])
            }
        }
    }
}

fn main() -> eyre::Result<()> {
    App::new().add_plugins(DefaultPlugins)

        //.add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(ObjPlugin)
        /*.add_plugins((
            FrameTimeDiagnosticsPlugin,
            EntityCountDiagnosticsPlugin,
            SystemInformationDiagnosticsPlugin,
            RenderDiagnosticsPlugin,
            PerfUiPlugin
        ))*/
        
        .add_systems(Startup, (setup, spawn_trees))
        .add_plugins(LookTransformPlugin)
        .add_plugins(UnrealCameraPlugin::default())
        //.add_systems(Update, (calculate_area_speed, bird_boids).chain())
        .run();
    Ok(())
}
fn spawn_trees(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let tree_scene = asset_server.load("tree/scene.gltf#Scene0");

    for i in 0..1000 {
        commands.spawn((
            SceneRoot(tree_scene.clone()),
            Transform::from_xyz(i as f32 * 2.0, 0.0, 0.0), 
        ));
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>
) {
    let mesh_handle = asset_server.load::<Mesh>("002/Mesher.obj");
    let heightmap = HeightMap::from_file("assets/002/Output.r32", 1024).unwrap();
    commands.insert_resource(heightmap);
    let world_size = 100.0;

    /*commands.spawn((
        /*PerfUiFramerateEntries::default(),
        PerfUiSystemEntries::default(),
        PerfUiFixedTimeEntries::default()*/
        PerfUiAllEntries::default()
    ));*/

    commands.spawn((
        Mesh3d(mesh_handle),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::default()
            .with_translation(Vec3::new(0.0, 0.0, world_size))
            .with_scale(Vec3::new(world_size,world_size*2.0,world_size))
    ));

    // display
    // display 3 lines to display axis
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(10.0, 0.5, 0.5))),
        MeshMaterial3d(materials.add(Color::linear_rgb(1.0, 0.0, 0.0))),
        Transform::from_xyz(5.0, 0.25, 0.25)
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.5, 10.0, 0.5))),
        MeshMaterial3d(materials.add(Color::linear_rgb(0.0, 1.0, 0.0))),
        Transform::from_xyz(0.25, 5.0, 0.25)
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.5, 0.5, 10.0))),
        MeshMaterial3d(materials.add(Color::linear_rgb(0.0, 0.0, 1.0))),
        Transform::from_xyz(0.25, 0.25, 5.0)
    ));

    commands.spawn((
        Camera3d::default(),
        UnrealCameraBundle::new(
            UnrealCameraController::default(), 
            Vec3::new(-1.0, 0.0, -1.0), Vec3::ZERO, Vec3::Y)
        //Transform::from_xyz(-5.0, 5.0, -5.0).looking_at(Vec3::ZERO, Vec3::Y)
    ));

    commands.spawn((
        PointLight::default(),
        Transform::from_xyz(50.0, 1000.0, 50.0)
    ));
    // generate 100 random cubes at the height in the heightmap
    //let mut rng = rng();
    //let cuboid_size = 1.0;
    //let mesh = meshes.add(Cuboid::new(cuboid_size, cuboid_size, cuboid_size));
    //let mat = materials.add(Color::linear_rgb(0.0, 1.0, 0.0));
   

    
}


