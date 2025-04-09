use bevy::{prelude::*, render::mesh::PlaneMeshBuilder};
use bevy_water::{material::{StandardWaterMaterial, WaterMaterial}, WaterSettings};

use crate::mask::{BlueNoise, Mask};

use super::map::HeightMap;

// spawns an ocean mesh at a specified height, uses a prebuilt material from bevy_water to actually give nice oceans
pub fn spawn_ocean(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardWaterMaterial>>,
    mut materials2: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>
) {
    let world_size = 100.0;
    let plane = meshes.add(PlaneMeshBuilder::from_length(world_size * 10.0).subdivisions(18).build());
    let mat = materials.add(StandardWaterMaterial {
        base: default(),
        extension: WaterMaterial {
          //amplitude: 0.0001,
          coord_scale: Vec2::splat(256.0 * 4.0), // change ocean scale to make waves more realistic
          ..default()
        },
      });
    let plane2 = Plane3d::new(Vec3::Y, Vec2::splat(world_size) * 10.0);
    for delta in [-1.0, 1.0]{
        commands.spawn((
            Mesh3d(plane.clone()),
            MeshMaterial3d(mat.clone()),
            Transform::from_xyz(world_size / 2.0 + delta * world_size / 2.0, 2.0, world_size / 2.0)
        ));
    }
    commands.spawn(( // the map is a limited size but if we look into the ocean this would look weird since its semi transparent
        Mesh3d(meshes.add(plane2)), // this adds a gigantic plane so that the map seems to continue even if it stops having any detail
        MeshMaterial3d(materials2.add(Color::WHITE)),
        Transform::from_xyz(0.0, 0.1, 0.0)
    ));
}

// big TODO
pub fn spawn_rivers(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    height_map: Res<HeightMap>
){
    /*let mask = Mask::from_file("assets/002/RiverDepth.r32", 1024).unwrap();
    // just spawn a bunch of cubes
    let cube = meshes.add(Cuboid::from_length(10.0));
    let mat = materials.add(Color::linear_rgb(0.0, 0.0, 1.0));
    let mut cubes_spawned = 0;
    for [x,y] in BlueNoise::new(0.005, &mask){
        commands.spawn((
            Mesh3d(cube.clone()),
            MeshMaterial3d(mat.clone()),
            Transform::from_xyz(
                x, 
                height_map.height_at(x, y), 
                y
            )
        ));
        cubes_spawned += 1;
    }
    println!("spawned {cubes_spawned} pseudo river tiles");*/
}