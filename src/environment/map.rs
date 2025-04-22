use std::{f32::consts::PI, fs::File, io::Read, ops::BitOr, path::Path, sync::Arc};

use bevy::{asset::RenderAssetUsages, image::{ImageAddressMode, ImageFilterMode, ImageFormatSetting, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor}, pbr::ExtendedMaterial, prelude::*, render::render_resource::{AddressMode, Extent3d, FilterMode, SamplerDescriptor, TextureDimension, TextureFormat, TextureUsages}};
use bitflags::bitflags;
use image::{ImageBuffer, ImageReader, Rgba};

use crate::{material::TerrainMaterialExtension};

/// a heigthmap loaded from a raw file
#[derive(Resource, Clone)]
pub struct HeightMap(Arc<[f32]>, usize); // the arc makes it cheap to clone
impl HeightMap {
    pub fn from_file<T: AsRef<Path>>(path: T, size: usize) -> std::io::Result<Self> {
        let mut file = File::open(path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        let data: &[f32] = bytemuck::cast_slice(data.as_slice());
        Ok(HeightMap(data.to_owned().into_boxed_slice().into(), size))
    }
    /// sample the height at a specified coordinate
    pub fn height_at(&self, x: f32, y: f32) -> f32 {
        let width = self.1 as f32;
        let x = (x * width) as usize;
        let y = (y * width) as usize;
        self.0[y * self.1 + x]
    }
    pub fn height_at_checked(&self, x: f32, y: f32) -> Option<f32> {
        if (x < 0.0 || x > 1.0) { return None; }
        if (y < 0.0 || y > 1.0) { return None; }
        let width = self.1 as f32;
        let x = (x * width) as usize;
        let y = (y * width) as usize;
        self.0.get(y * self.1 + x).copied()
    }
    pub fn size(&self) -> f32 {
        self.1 as f32
    }
}

#[derive(Resource)]
pub struct MapData{
    pub size: f32,
    biome_map: ImageBuffer<Rgba<u8>, Vec<u8>>
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Biome {
    Plains,
    Forest,
    Taiga,
    Mountains
}
impl Biome {
    pub fn from_u8(num: u8) -> Option<Self>{
        let out = match num {
            0 => Self::Plains,
            1 => Self::Forest,
            2 => Self::Taiga,
            3 => Self::Mountains,
            _ => return None
        };
        Some(out)
    }
    pub const fn to_bitflag(&self) -> u8 {
        match self {
            Self::Plains => 0b1,
            Self::Forest => 0b10,
            Self::Taiga => 0b100,
            Self::Mountains => 0b1000,
        }
    }
    pub fn to_collection(&self) -> BiomeCollection {
        match self {
            Self::Plains => BiomeCollection::Plains,
            Self::Forest => BiomeCollection::Forest,
            Self::Taiga => BiomeCollection::Taiga,
            Self::Mountains => BiomeCollection::Mountains
        }
    }
}
bitflags! {
    #[derive(Component, Copy, Clone)]
    pub struct BiomeCollection: u8 {
        const Plains = Biome::Plains.to_bitflag();
        const Forest = Biome::Forest.to_bitflag();
        const Taiga = Biome::Taiga.to_bitflag();
        const Mountains = Biome::Mountains.to_bitflag();
    }
}
impl BitOr<Biome> for BiomeCollection {
    type Output = Self;
    fn bitor(self, rhs: Biome) -> Self::Output {
        self | rhs.to_collection()
    }
}
impl BitOr for Biome {
    type Output = BiomeCollection;
    fn bitor(self, rhs: Self) -> Self::Output {
        self.to_collection() | rhs.to_collection()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BiomeBlend{
    pub a: Biome,
    pub b: Biome,
    pub blend: f32
}
impl BiomeBlend {
    pub fn get_primary_biome(&self) -> Biome {
        if self.blend > 0.5 {
            self.a
        } else {
            self.b
        }
    }
    pub fn get_secondary_biome(&self) -> Biome {
        if self.blend > 0.5 {
            self.b
        } else {
            self.a
        }
    }
}
impl MapData {
    pub fn new(size: f32, biome_map: ImageBuffer<Rgba<u8>, Vec<u8>>) -> Self {
        Self {
            size,
            biome_map
        }
    }
    pub fn biome_at_point(&self, x: f32, y: f32) -> Option<BiomeBlend> {
        let (x,y) = (
            x / self.size * self.biome_map.width() as f32,
            y / self.size * self.biome_map.height() as f32
        );
        let Rgba([r,g,b,a]) = self.biome_map.get_pixel_checked(x as u32, y as u32)?;
        Some(BiomeBlend {
            a: Biome::from_u8(*r).unwrap(),
            b: Biome::from_u8(*b).unwrap(),
            blend: *b as f32 / 255.0
        })
    }
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
    let ground = asset_server.load::<Mesh>("011/mesh3.glb#Mesh0/Primitive0");
    let world_transform = Transform::default()
        .with_rotation(
           Quat::from_rotation_x(PI/2.0)
        )
        .with_translation(Vec3::new(world_size/2.0, 0.0, world_size/2.0))
        .with_scale(Vec3::new(world_size,world_size,2.0*world_size));

    let img = ImageReader::open("assets/test.png").unwrap()
        .decode().unwrap()
        .to_rgba8();
    let biome_map = MapData::new(world_size, img.clone());
    commands.insert_resource(biome_map);
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
                textures: asset_server.load("012/texture.ktx2"),
                normals: asset_server.load("012/normals.ktx2"),
                parallax: asset_server.load("012/displacment.ktx2"),
                splatmap: images.add(bevy_image),
                filtered_splatmap: asset_server.load("test.png")
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