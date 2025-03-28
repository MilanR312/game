use std::{fs::File, io::Read, path::Path};

use bevy::prelude::*;

/// a heigthmap loaded from a raw filel
#[derive(Resource)]
pub struct HeightMap(Vec<f32>, usize);
impl HeightMap {
    pub fn from_file<T: AsRef<Path>>(path: T, size: usize) -> std::io::Result<Self> {
        let mut file = File::open(path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        let data: &[f32] = bytemuck::cast_slice(data.as_slice());
        Ok(HeightMap(data.to_owned(), size))
    }
    /// sample the height at a specified coordinate
    pub fn height_at(&self, x: f32, y: f32) -> f32 {
        let width = self.1 as f32;
        let x = (x * width) as usize;
        let y = (y * width) as usize;
        self.0[y * self.1 + x]
    }
}

#[derive(Resource)]
pub struct MapData{
    pub size: f32
}

pub fn load_map(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
){
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

    let heightmap = HeightMap::from_file("assets/002/Output.r32", 1024).unwrap();
    commands.insert_resource(heightmap);
    let world_size = 100.0;

    let ground = asset_server.load::<Mesh>("011/Mesher.obj");
    let texture = asset_server.load("011/Combine.png");
    //let mountain = asset_server.load::<Mesh>("005/Mountain.obj");
    let world_transform = Transform::default()
        .with_translation(Vec3::new(world_size/2.0, 0.0, world_size/2.0))
        .with_scale(Vec3::new(world_size,world_size*2.0,world_size));
    commands.spawn((
        Mesh3d(ground),
        MeshMaterial3d(materials.add(StandardMaterial{
            base_color_texture: Some(texture),
            perceptual_roughness: 0.9,
            metallic: 0.0,
            reflectance: 0.0,
            ..Default::default()
        })),
        world_transform.clone()
    ));
    /*commands.spawn((
        Mesh3d(mountain),
        MeshMaterial3d(materials.add(Color::linear_rgb(0.3, 0.3, 0.3))),
        world_transform.clone()
    ));
    commands.insert_resource(MapData{
        size: world_size
    });*/
}