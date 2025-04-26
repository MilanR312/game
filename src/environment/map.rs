use std::{f32::consts::PI, fs::File, io::{BufReader, Read}, ops::BitOr, path::Path, sync::Arc};

use bevy::{asset::{io::{AssetSourceId, AssetSources, AssetWriter}, saver::{AssetSaver, SavedAsset}, ErasedLoadedAsset, LoadState, LoadedAsset, RenderAssetUsages}, gltf::{GltfMesh, GltfNode}, image::{ImageAddressMode, ImageFilterMode, ImageFormatSetting, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor}, pbr::{experimental::meshlet::{self, MeshToMeshletMeshConversionError, MeshletMesh, MeshletMesh3d, MeshletMeshSaver}, ExtendedMaterial}, prelude::*, render::{render_resource::{AddressMode, Extent3d, FilterMode, SamplerDescriptor, TextureDimension, TextureFormat, TextureUsages}, view::VisibilityRange}, tasks::block_on};
use bevy_asset_loader::asset_collection::AssetCollection;
use bitflags::bitflags;
use image::{GenericImageView, ImageBuffer, ImageReader, Rgba};

use crate::material::{TerrainMaterialExtension, TestMaterialExtension};

/// a heigthmap loaded from a raw file
#[derive(Resource, Clone)]
pub struct HeightMap(Arc<[f32]>, usize); // the arc makes it cheap to clone
impl HeightMap {
    pub fn from_file<T: AsRef<Path>>(path: T, size: usize) -> std::io::Result<Self> {
        let mut file = File::open(path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        println!("heightmap size == {} or {}", data.len(), data.len() as f64 / 4.0);
        let data: &[f32] = bytemuck::cast_slice(data.as_slice());
        Ok(HeightMap(data.to_owned().into_boxed_slice().into(), size))
    }
    pub fn from_png<T: AsRef<Path>>(path: T) -> std::io::Result<Self>{
        let reader = BufReader::new(File::open(path)?);
        let mut image = image::ImageReader::new(reader);
        image.set_format(image::ImageFormat::Png);
        let img = image.decode().unwrap();
        let size = img.width() as usize;
        assert_eq!(size, img.height() as usize);
        let raw_data: Arc<[f32]> = img.pixels().map(|(_, _, p)| (p.0[0]) as f32 / 255.0).collect();
        //let raw_data = img.into_bytes().into_iter().map(|x| x as f32 / 255.0);
        assert_eq!(raw_data.len(), size * size);
        Ok(Self {
            0: raw_data,
            1: size
        })
    }
    const SCALER: f32 = 1.04;
    /// sample the height at a specified coordinate
    pub fn height_at(&self, x: f32, y: f32) -> f32 {
        let width = self.1 as f32;
        let x = (x * width) as usize;
        let y = (y * width) as usize;
        self.0[y * self.1 + x] * Self::SCALER
    }
    pub fn height_at_checked(&self, x: f32, y: f32) -> Option<f32> {
        if (x < 0.0 || x > 1.0) { return None; }
        if (y < 0.0 || y > 1.0) { return None; }
        let width = self.1 as f32;
        let x = (x * width) as usize;
        let y = (y * width) as usize;
        self.0.get(y * self.1 + x).copied().map(|x| x * Self::SCALER)
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

#[derive(Clone, Copy, Component, PartialEq, Eq, Debug)]
pub enum MapLodType{
    Main,
    Lod1,
    Lod2,
    Lod3
}
#[derive(AssetCollection, Resource)]
pub struct Map{
    #[asset(path = "012/lod0.glb")]
    lod0: Handle<Gltf>,
    #[asset(path = "012/lod1.glb")]
    lod1: Handle<Gltf>,
    #[asset(path = "012/lod2.glb")]
    lod2: Handle<Gltf>,
    #[asset(path = "012/lod3.glb")]
    lod3: Handle<Gltf>,
}
#[derive(Component)]
pub enum MapLoadState{
    Gltf(Handle<Gltf>, MapLodType),
    Chunk(Handle<GltfNode>),
    MeshData(Handle<GltfMesh>)
}
#[derive(Resource)]
pub struct MapMaterial(Handle<ExtendedMaterial<StandardMaterial, TerrainMaterialExtension>>);


pub fn load_map(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut materials2: ResMut<Assets<ExtendedMaterial<StandardMaterial, TerrainMaterialExtension>>>,
    mut materials3: ResMut<Assets<ExtendedMaterial<StandardMaterial, TestMaterialExtension>>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mapdata: Res<Map>,
    mut images: ResMut<Assets<Image>>,
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
    let heightmap = HeightMap::from_file("assets/011/Output.r32", 4096).unwrap(); // cant do png bad resolution
    let world_size = 100.0;

    // get the first mesh of a glb file, the map is stored as a glb since it needs tangents for the material. Obj files dont support tangents
    //let ground = asset_server.load::<Mesh>("011/mesh3.glb#Mesh0/Primitive0");
    commands.spawn((
        MapLoadState::Gltf(mapdata.lod0.clone(), MapLodType::Main),
    ));
    commands.spawn((
        MapLoadState::Gltf(mapdata.lod1.clone(), MapLodType::Lod1),
    ));
    commands.spawn((
        MapLoadState::Gltf(mapdata.lod2.clone(), MapLodType::Lod2),
    ));
    commands.spawn((
        MapLoadState::Gltf(mapdata.lod0.clone(), MapLodType::Lod3),
    ));
    
    //let ground = asset_server.load::<Mesh>("012/LOD3.gltf#Mesh0/LOD4a0x0");
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
    let foo = MeshMaterial3d(
        materials3.add(ExtendedMaterial{
            base: StandardMaterial::default(),
            extension: TestMaterialExtension{ 
                normals: asset_server.load("testmat/cube_normal.png"),
                textures: asset_server.load("testmat/cube_color.png"),
                parallax: asset_server.load("testmat/cube_depth.png")
            }
        })
    );

    let handle = materials2.add(ExtendedMaterial {
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
    });

    commands.insert_resource(MapMaterial(handle));

    commands.insert_resource(heightmap);

}
pub fn generate_chunks(
    mut commands: Commands,
    mut q: Query<(Entity, &mut MapLoadState)>,
    gltfs: Res<Assets<Gltf>>,
    nodes: Res<Assets<GltfNode>>,
    gltf_meshes: Res<Assets<GltfMesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    size: Res<MapData>,
    map_material: Res<MapMaterial>
){
    for (entity, gltf) in q.iter_mut(){
        match &*gltf {
            MapLoadState::Gltf(map, x) => {
                let Some(gltf) = gltfs.get(map.id()) else {
                    continue;
                };
                println!("transformed mesh to chunks {x:?}");
                let visibility_range = match *x {
                    MapLodType::Main => VisibilityRange {
                        use_aabb: false,
                        start_margin: 0.0 .. 0.0,
                        end_margin: 15.0 .. 20.0
                    },
                    MapLodType::Lod1 => VisibilityRange {
                        use_aabb: false,
                        start_margin: 15.0 .. 20.0,
                        end_margin: 35.0 .. 40.0
                    },
                    MapLodType::Lod2 => VisibilityRange {
                        use_aabb: false,
                        start_margin: 35.0 .. 40.0,
                        end_margin: 60.0 .. 70.0
                    },
                    MapLodType::Lod3 => VisibilityRange {
                        use_aabb: false,
                        start_margin: 60.0 .. 70.0,
                        end_margin: 150.0 .. 500.0
                    },
                };
                for node in &gltf.nodes {
                    commands.spawn((
                        MapLoadState::Chunk(node.clone()),
                        visibility_range.clone()
                    ));
                }
                commands.entity(entity)
                    .despawn();
            },
            MapLoadState::Chunk(chunk) => {
                let Some(node) = nodes.get(chunk.id()) else {
                    continue;
                };
                let mesh = node.mesh.as_ref().unwrap();
                let transform = node.transform.clone();
                let mut transform = transform.with_scale(Vec3::splat(size.size).with_z(size.size * 2.0));
                transform.translation.y *= 2.0;
                transform.translation *= size.size;
                transform.translation += Vec3::splat(size.size / 2.0).with_y(0.0);

                commands.entity(entity)
                    .remove::<MapLoadState>()
                    .insert((
                        MapLoadState::MeshData(mesh.clone()),
                        transform
                    ));
            },
            MapLoadState::MeshData(x) => {
                let Some(mesh) = gltf_meshes.get(x.id()) else {
                    continue;
                };
                let mesh = mesh.primitives[0].mesh.clone();
                commands.entity(entity)
                    .remove::<MapLoadState>()
                    .insert((
                        Visibility::Visible,
                        Mesh3d(mesh),
                        MeshMaterial3d(map_material.0.clone())
                    ));
                
            }
            _ => {}
        }
    }
}
