use std::{f32::consts::PI, fs::File, io::Read, path::Path};

use bevy::{asset::RenderAssetUsages, image::{ImageAddressMode, ImageFilterMode, ImageFormatSetting, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor}, pbr::ExtendedMaterial, prelude::*, render::render_resource::{AddressMode, Extent3d, FilterMode, SamplerDescriptor, TextureDimension, TextureFormat, TextureUsages}};
use image::ImageReader;

use crate::material::TerrainMaterialExtension;

/// a heigthmap loaded from a raw file
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
    mut materials2: ResMut<Assets<ExtendedMaterial<StandardMaterial, TerrainMaterialExtension>>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>
){
    // show the 3 axis as red green and blue lines in the world
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
    // load the heightmap and save it as a resource
    let heightmap = HeightMap::from_file("assets/002/Output.r32", 1024).unwrap();
    commands.insert_resource(heightmap);
    let world_size = 100.0;

    // get the first mesh of a glb file, the map is stored as a glb since it needs tangents for the material. Obj files dont support tangents
    let ground = asset_server.load::<Mesh>("011/mesh.glb#Mesh0/Primitive0");
    let world_transform = Transform::default()
        .with_rotation(
           Quat::from_rotation_x(PI/2.0)
        )
        .with_translation(Vec3::new(world_size/2.0, 0.0, world_size/2.0))
        .with_scale(Vec3::new(world_size,world_size,2.0*world_size));

    let img = ImageReader::open("assets/test.png").unwrap()
        .decode().unwrap()
        .to_rgba8();
    let (width, height) = img.dimensions();
    let data = img.into_raw();

    let mut bevy_image = Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8Uint,
        RenderAssetUsages::default()
    );

    // Set the sampler to non-filtering (nearest neighbor)
    bevy_image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        label: Some("non_filtering_sampler".into()),
        address_mode_u: ImageAddressMode::ClampToEdge,
        address_mode_v: ImageAddressMode::ClampToEdge,
        address_mode_w: ImageAddressMode::ClampToEdge,
        mag_filter: ImageFilterMode::Nearest,
        min_filter: ImageFilterMode::Nearest,
        mipmap_filter: ImageFilterMode::Nearest,
        ..Default::default()
    });

    commands.spawn((
        Mesh3d(ground),
        MeshMaterial3d(materials2.add(ExtendedMaterial {
            base: StandardMaterial {
                ..Default::default()
            },
            extension: TerrainMaterialExtension {
                textures: asset_server.load("textures/textures/texture.ktx2"),
                normals: asset_server.load("textures/textures/normals.ktx2"),
                parallax: asset_server.load("textures/textures/displacment.ktx2"),
                splatmap: images.add(bevy_image)
            }
        })),
        world_transform.clone()
    ));
    commands.spawn(( // testing light
        PointLight{
            intensity: 100000000.0,
            ..Default::default()
        },
        Transform::from_xyz(50.0, 0.0, 0.0)
    ));
}