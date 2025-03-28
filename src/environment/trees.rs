use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, ComputeTaskPool, TaskPool},
};
use rand::{Rng, SeedableRng};

use crate::mask::{BlueNoise, Mask};

use super::map::HeightMap;

#[derive(Resource)]
struct TreePositions {
    positions: Vec<Transform>,
}

pub fn spawn_trees(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    height_map: Res<HeightMap>,
) {
    // this dumb small piece of code took so much work :(
    // the assets i got were gltfs and adding so many of them bloated the ecs so it started lagging
    // i had to learn blender baking and texture merging to get this to work :(

    let obj = asset_server.load("tree/tree3.obj");
    let texture = asset_server.load("tree/OutputTexture.png");
    let material = materials.add(StandardMaterial {
        base_color_texture: Some(texture),
        alpha_mode: AlphaMode::Mask(0.5),
        ..Default::default()
    });

    let mask = Mask::from_file("assets/002/Oakmap.r32", 1024).expect("failed to find mask");
    for [x, y] in BlueNoise::new(0.002, &mask).take(5000) {
        let mut local_rng = rand::prelude::SmallRng::seed_from_u64((x * y + y) as u64);
        let height = height_map.height_at(x, y);
        let rand_transform = Transform::from_xyz(x * 100.0, height * 100.0, y * 100.0)
            .with_rotation(Quat::from_rotation_y(local_rng.random_range(0.0..6.24)))
            .with_scale(
                Vec3::splat(0.02)
                    * Vec3::new(
                        local_rng.random_range(0.9..1.1),
                        local_rng.random_range(0.9..1.1),
                        local_rng.random_range(0.9..1.1),
                    ),
            );

        commands.spawn((
            Mesh3d(obj.clone()),
            MeshMaterial3d(material.clone()),
            rand_transform,
        ));
    }
}
