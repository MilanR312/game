/*
Grass system from https://github.com/mikeam565/first-game/tree/main/assets
ported to bevy 15.3 and modified for our game
the amount of modifications is quite small

*/



use std::borrow::BorrowMut;
use std::f32::consts::FRAC_PI_2;

use bevy::ecs::system::{SystemState};
use bevy::ecs::world::CommandQueue;
use bevy::pbr::{ExtendedMaterial, MaterialExtension, MaterialExtensionKey, MaterialExtensionPipeline, MaterialPipeline, NotShadowCaster, NotShadowReceiver};
use bevy::render::globals::GlobalsBuffer;
use bevy::render::mesh::{Indices, MeshAabb, MeshVertexAttribute, MeshVertexBufferLayout};
use bevy::render::primitives::Aabb;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{AsBindGroup, ColorTargetState, ColorWrites, GpuArrayBuffer, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError, VertexFormat};
use bevy::render::view::VisibilityRange;
use bevy::tasks::{block_on, poll_once, AsyncComputeTaskPool, Task};
use bevy::{prelude::*, render::{render_resource::{PrimitiveTopology, Face}, mesh::{self, VertexAttributeValues}}, utils::HashMap};
use noise::{Perlin, NoiseFn};
use rand::{rng, thread_rng, Rng, SeedableRng};

use crate::mask::{BlueNoise, Mask};

use super::map::{HeightMap, MapData};

// Grass constants
const GRASS_SCALER: f32 = 0.1;
const GRASS_TILE_SIZE_1: f32 = 32. * GRASS_SCALER;
const GRASS_TILE_SIZE_2: f32 = 32.; // TODO: like terrain, this causes overlaps if bigger than SIZE_1
const NUM_GRASS_1: u32 = 128; // number of grass blades in one row of a tile
const NUM_GRASS_2: u32 = 32;
const GRASS_BLADE_VERTICES: u32 = 3;
const GRASS_WIDTH: f32 = 0.3 * GRASS_SCALER;
const GRASS_HEIGHT: f32 = 1.2 * GRASS_SCALER;
const GRASS_BASE_COLOR_1: [f32;4] = [0.102,0.153,0.,1.];
const GRASS_BASE_COLOR_2: [f32;4] = [8.0 / 255.0,12.0 / 255.0,2.0 / 255.0,1.];
pub const GRASS_SECOND_COLOR: [f32;4] = [28.0 / 255.0,38.0 / 255.0,14.0 / 255.0,1.];
const GRASS_SCALE_FACTOR: f32 = 1.0;
const GRASS_HEIGHT_VARIATION_FACTOR: f32 = 0.2 * GRASS_SCALER;
const GRASS_STRAIGHTNESS: f32 = 10.0; // for now, as opposed to a curve factor, just modifying denominator for curve calcs
const GRASS_SPACING: f32 = 0.3;
const GRASS_OFFSET: f32 = 0.2 * GRASS_SCALER;
const ENABLE_WIREFRAME: bool = false;
const WIND_STRENGTH: f32 = 1.;
const WIND_SPEED: f64 = 0.5;
const WIND_CONSISTENCY: f64 = 50.0; //
const WIND_LEAN: f32 = 0.0; // determines how already bent grass will be at 0 wind
const CURVE_POWER: f32 = 1.0; // the linearity / exponentiality of the application/bend of the wind
const DESPAWN_DISTANCE: f32 = 10.0;
const WIND_SIM_TRIGGER_DISTANCE: f32 = 3. * GRASS_TILE_SIZE_1;
const WIND_SIM_DISTANCE: f32 = WIND_SIM_TRIGGER_DISTANCE - GRASS_TILE_SIZE_1/2.;

const ATTRIBUTE_BASE_Y: MeshVertexAttribute = MeshVertexAttribute::new("BaseY", 988540917, VertexFormat::Float32);
const ATTRIBUTE_STARTING_POSITION: MeshVertexAttribute = MeshVertexAttribute::new("StartingPosition", 988540916, VertexFormat::Float32x3);
const ATTRIBUTE_WORLD_POSITION: MeshVertexAttribute = MeshVertexAttribute::new("WorldPosition", 988540915, VertexFormat::Float32x3);

const GRID_SIZE_HALF: i32 = 32;

#[derive(Component)]
pub struct ContainsPlayer(pub bool);

fn grass_material() -> StandardMaterial {
    StandardMaterial {
        base_color: Color::WHITE,
        double_sided: false,
        perceptual_roughness: 1.0,
        reflectance: 0.5,
        cull_mode: None,
        alpha_mode: AlphaMode::Opaque,
        opaque_render_method: bevy::pbr::OpaqueRendererMethod::Forward,
        unlit: false,
        ..default()
    }
}

// Grass Component
#[derive(Component)]
pub struct GrassData {
    initial_vertices: Vec<Vec3>,
    initial_positions: Vec<[f32;3]>,
}

#[derive(Component,Clone)]
pub struct Grass;

// Grass offsets component
#[derive(Component)]
pub struct GrassItem;

pub fn generate_grass_items(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
    height_map: Res<HeightMap>,
    grass: Res<BushMask>,
) {
    let material: Handle<StandardMaterial> = asset_server.load("bushes/bushes_cleaned.glb#Material0");

    let bushes: [Handle<Mesh>; 2] = [
        asset_server.load("bushes/bushes_cleaned.glb#Mesh0/Primitive0"),
        asset_server.load("bushes/bushes_cleaned.glb#Mesh1/Primitive0"),
    ];

    /*commands.spawn((
        SceneRoot(asset_server.load("bushes/bushes_cleaned.glb#Scene0")),
        Transform::from_scale(Vec3::splat(1.0)),
        GrassItem
    ));*/

    let cube = meshes.add(Cuboid::from_length(0.5));

    let mask = grass.0.clone();
    let noise = BlueNoise::new(0.005, mask);
    let mut rng = rng();
    let mut cnt = 0;
    for [x, y] in noise.take(200) {
        let mut local_rng = rand::prelude::SmallRng::seed_from_u64((x * y + y) as u64);
        let height = height_map.height_at(x, y);
        let bush_used = rng.random_range(0..bushes.len());
        let bush = &bushes[bush_used];
        let rand_transform = Transform::from_xyz(x * 100.0, height * 100.0, y * 100.0)
            .with_rotation(
                Quat::from_rotation_y(local_rng.random_range(0.0..6.24)) 
                *
                Quat::from_rotation_x(-FRAC_PI_2)
                
            )
            .with_scale(
                Vec3::splat(0.005)
                    * Vec3::new(
                        local_rng.random_range(0.9..1.1) ,
                        local_rng.random_range(0.9..1.1),
                        local_rng.random_range(0.9..1.1),
                    ),
            );
        warn!("spawn at {rand_transform:?}");
        commands.spawn((
            rand_transform,
            //MeshMaterial3d(materials.add(Color::WHITE)),
            MeshMaterial3d(material.clone()),
            Mesh3d(bush.clone()),
            GrassItem,
            VisibilityRange {
                use_aabb: false,
                start_margin: 0.0..0.0,
                end_margin: 20.0..22.0
            }
        ));
        cnt += 1;
    }
    info!("spawned {cnt} bushes");
}


pub fn generate_grass_mesh(
    spawn_x: f32,
    spawn_z: f32,
    density: u32,
    tile_size: f32,
    heightmap: &HeightMap,
    mask: &Mask,
    mapsize: f32
) -> (Mesh, GrassData) {
    let mut grass_offsets = vec![];
    let mut rng = thread_rng();
    let asset_usage = RenderAssetUsages::RENDER_WORLD;// | RenderAssetUsages::MAIN_WORLD;
    // let asset_usage = RenderAssetUsages::RENDER_WORLD;
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, asset_usage);
    let mut all_verts: Vec<Vec3> = vec![];
    let mut all_indices: Vec<u32> = vec![];
    let mut all_normals: Vec<Vec3> = vec![];
    let mut all_uvs: Vec<Vec3> = vec![];

    let mut all_colors: Vec<[f32; 4]> = vec![];
    let mut blade_number = 0;
    let height_perlin = Perlin::new(0);
    let start_x = - tile_size/2.;
    let start_z = - tile_size/2.;
    for i in 0..density {
        let x = start_x + i as f32 * tile_size / density as f32;
        for j in 0..density {
            let z = start_z + j as f32 * tile_size / density as f32;
            let rand1 = if GRASS_OFFSET!=0.0 {rng.gen_range(-GRASS_OFFSET..GRASS_OFFSET)} else {0.0};
            let rand2 = if GRASS_OFFSET!=0.0 {rng.gen_range(-GRASS_OFFSET..GRASS_OFFSET)} else {0.0};
            let x_offset = x + rand1;
            let z_offset = z + rand2;
            let t = 0.0;
            let x_sample = (spawn_x + x_offset) / (mapsize - t);
            let z_sample = (spawn_z + z_offset) / (mapsize - t);
            // TODO
            let Some(y) = heightmap.height_at_checked(
                x_sample,
                z_sample
            ) else {
                continue;
            };
            let Some(spawnable) = mask.sample_at_checked(
                x_sample,
                z_sample
            ) else {
                continue;
            };
            if spawnable < 0.5 {
                continue;
            }
            let y = y * mapsize;
            //let y = sample_terrain_height(&terrain_perlin, spawn_x + x_offset, spawn_z + z_offset) - 0.2; // minus small amount to avoid floating
            let blade_height = GRASS_HEIGHT + (height_perlin.get([(spawn_x + x_offset) as f64, (spawn_z + z_offset) as f64]) as f32 * GRASS_HEIGHT_VARIATION_FACTOR);
            //if y > terrain::HEIGHT_TEMPERATE_START && y < terrain::HEIGHT_TEMPERATE_END {
                let (mut verts, mut indices, mut normals, mut uvs) = generate_single_blade_verts(x_offset, y, z_offset, blade_number, blade_height);
                if verts.len() == 0 {
                }
                for v in &verts {
                    grass_offsets.push([spawn_x + x_offset,y,spawn_z + z_offset]);
                    let r_color_shift = 0.0;
                    //let r_color_shift = (terrain_perlin.get([(spawn_x + x_offset) as f64 / 100., (spawn_z + z_offset) as f64 / 100.]) * 0.01) as f32;
                    let mut color = color_gradient_y_based(v.y-y, GRASS_BASE_COLOR_2, GRASS_SECOND_COLOR);
                    color[0] += r_color_shift;
                    color[3] = 0.3;
                    all_colors.push(color);
                }
                all_verts.append(&mut verts);
                all_indices.append(&mut indices);
                all_normals.append(&mut normals);
                all_uvs.append(&mut uvs);
                blade_number += 1;
            //}
        }
    }
    generate_grass_geometry(&all_verts, all_indices, &mut mesh, &grass_offsets, all_colors, &all_normals, &all_uvs);

    (
        mesh,
        GrassData {
            initial_vertices: all_verts,
            initial_positions: grass_offsets
        }
    )
}

fn grass_data_to_base_data(grass_data: Vec<[f32; 3]>) -> [f32; (GRASS_BLADE_VERTICES * (NUM_GRASS_1*NUM_GRASS_1 + 2)) as usize] {
    let mut arr = [0.; (GRASS_BLADE_VERTICES * (NUM_GRASS_1*NUM_GRASS_1 + 2)) as usize];
    for (i, v) in grass_data.iter().enumerate() {
        arr[i] = v[1];
    }
    arr
}

pub fn generate_grass(
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ExtendedMaterial<StandardMaterial,GrassMaterialExtension>>>,
    spawn_x: f32,
    spawn_z: f32,
    density: u32,
    tile_size: f32,
    heightmap: &HeightMap,
    mask: &Mask,
    mapsize: f32
) -> (
    Handle<ExtendedMaterial<StandardMaterial, GrassMaterialExtension>>,
    Handle<Mesh>,
    Transform,
    Grass,
    GrassData    
)
{
    let (mesh, grass_data) = generate_grass_mesh(spawn_x, spawn_z, density, tile_size, heightmap, mask, mapsize);

    let grass_material_ext = GrassMaterialExtension {
    };
    
    let grass_material_std = grass_material();

    let grass_material = ExtendedMaterial {
        base: grass_material_std,
        extension: grass_material_ext
    };

    (
        materials.add(grass_material),
        meshes.add(mesh),
        Transform::from_xyz(spawn_x, 0.0, spawn_z),
        Grass {},
        grass_data,
    )

}

pub fn generate_single_blade_verts(x: f32, y: f32, z: f32, blade_number: u32, blade_height: f32) -> (Vec<Vec3>, Vec<u32>, Vec<Vec3>, Vec<Vec3>) {
    // For grass with 7 vertices, uncomment t3-6, and uncomment indices
    // vertex transforms
    let t1 = Vec3::new(x, y, z);
    let t2 = Vec3::new(x+GRASS_WIDTH, y, z);
    // let t3 = Transform::from_xyz(x, y+blade_height/3.0, z);
    // let t4 = Transform::from_xyz(x+GRASS_WIDTH, y+blade_height/3.0, z);
    // let t5 = Transform::from_xyz(x, y+2.0*blade_height/3.0, z);
    // let t6 = Transform::from_xyz(x + GRASS_WIDTH, y+2.0*blade_height/3.0, z);
    let t7 = Vec3::new(x+(GRASS_WIDTH/2.0), y+blade_height, z);
    
    // let mut transforms = vec![t1,t2,t3,t4,t5,t6,t7];
    // let mut transforms = vec![t1,t2,t5,t6,t7];
    let mut vertices = vec![t1,t2,t7];

    let normals = vertices.iter().map(|_| Vec3::new(0.0, 0.0, 1.0)).collect::<Vec<_>>();
    let uvs = vertices.iter().map(|vert| {
        let mut out = *vert;
        out.x = (out.x - x) / GRASS_WIDTH;
        out.y = (out.y - y) / blade_height;
        out.z = 0.0;
        out
    }).collect::<Vec<_>>();

    let blade_number_shift = blade_number*vertices.len() as u32;

    let rotation_point = Vec3::new(x, y, z);
    let mut rng = rng();
    let rotate_point = |point: Vec3, around: Vec3, angle: f32| {
        let mut transform = Transform::from_translation(point);
        transform.rotate_around(around, Quat::from_rotation_y(angle));
        transform.translation
    };


    

        
    // physical randomization of grass blades
    // rotate grass randomly around y
        
    let rot1 = rng.random_range(0..628) as f32 / 100.0;
    for pt in &mut vertices {
        *pt = rotate_point(*pt, rotation_point, rot1);
    }
    
    // curve the grass all one way
    let curve_rotation_point = Vec3::new(x + rng.random_range(0..2) as f32 / 10.0, y, z + rng.random_range(0..2) as f32 / 10.0);
    let rand_curve = (rng.random_range(101..110) / 100) as f32;
    for pt in vertices.iter_mut() {
        let mut transform = Transform::from_translation(*pt);
        transform.rotate_around(curve_rotation_point, Quat::from_rotation_z(rand_curve * ((transform.translation.y - y) / (GRASS_HEIGHT*GRASS_STRAIGHTNESS))));
        *pt = transform.translation;
    }
    

    // rotate grass again
    let rot1 = rng.random_range(0..628) as f32 / 100.0;
    for pt in &mut vertices {
        *pt = rotate_point(*pt, rotation_point, rot1);
    }
    

    let indices: Vec<u32> = vec![
        blade_number_shift+0, blade_number_shift+1, blade_number_shift+2,
        // blade_number_shift+2, blade_number_shift+1, blade_number_shift+3,
        // blade_number_shift+2, blade_number_shift+3, blade_number_shift+4,
        // blade_number_shift+4, blade_number_shift+3, blade_number_shift+5,
        // blade_number_shift+4, blade_number_shift+5, blade_number_shift+6,
    ];
    (vertices, indices, normals, uvs)
}


// todo: clean up


pub fn generate_grass_geometry(verts: &Vec<Vec3>, vec_indices: Vec<u32>, mesh: &mut Mesh, grass_offsets: &Vec<[f32; 3]>, colors: Vec<[f32; 4]>, all_normals: &Vec<Vec3>, all_uvs: &Vec<Vec3>) {
    let indices = mesh::Indices::U32(vec_indices);

    let vertices: Vec<([f32;3],[f32;3],[f32;2])> = verts.iter().map(|v| { (v.to_array(), [0., 0., 1.], [0.0,0.0])} ).collect();

    let mut positions = Vec::with_capacity(verts.capacity());
    let mut normals = Vec::with_capacity(verts.capacity());
    let bases: Vec<f32> = grass_offsets.iter().map(|x| x[1]).collect();
    // let mut uvs: Vec<[f32; 2]> = Vec::new();
    for (position, normal, uv) in vertices.iter() {
        positions.push(*position);
        normals.push(*normal);
        // uvs.push(*uv);
    }
    /*for normal in all_normals.iter() {
    }*/
    /*for color in &colors{
        assert_eq!(color[3], 1.0);
    }*/

    mesh.insert_indices(indices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions.clone());
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    // mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    // mesh.generate_tangents().unwrap();
    mesh.insert_attribute(ATTRIBUTE_BASE_Y, bases);
    mesh.insert_attribute(ATTRIBUTE_STARTING_POSITION, positions);
    mesh.insert_attribute(ATTRIBUTE_WORLD_POSITION, grass_offsets.clone());

}

#[derive(Component)]
struct GrassGrid(HashMap<(i32,i32), bool>);

fn update_grass(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial,GrassMaterialExtension>>>,
    mut materials2: ResMut<Assets<StandardMaterial>>,
    mut grass: Query<(Entity, &Mesh3d, &GrassData, &Transform, &ViewVisibility, &mut ContainsPlayer), With<Grass>>,
    mut grid: Query<&mut GrassGrid>,
    heightmap: Res<HeightMap>,
    mask: Res<BushMask>,
    //perlin: Res<PerlinNoiseEntity>,
    time: Res<Time>,
    player: Query<(Entity,&Transform),With<Camera>>,
    mapdata: Res<MapData>
) {
    let mapsize = mapdata.size;
    let (plyr_e, player_trans) = player.get_single().unwrap();
    let x = player_trans.translation.x;
    let z = player_trans.translation.z;
    if grass.is_empty() {
        let mut grass_grid = GrassGrid(HashMap::new());
        // generate grid of grass
        for i in -GRID_SIZE_HALF..=GRID_SIZE_HALF {
            for j in -GRID_SIZE_HALF..=GRID_SIZE_HALF {
                let a = x + i as f32 * GRASS_TILE_SIZE_1;
                let b = z + j as f32 * GRASS_TILE_SIZE_1;
                grass_grid.0.insert((a as i32, b as i32), true);
                let contains_player = (player_trans.translation.x - a).abs() < GRASS_TILE_SIZE_1/2. && (player_trans.translation.z - b).abs() < GRASS_TILE_SIZE_1/2.;
                let (mat, mesh, transform, main_grass, main_data) = generate_grass(&mut meshes, &mut materials, a, b, NUM_GRASS_1, GRASS_TILE_SIZE_1, &heightmap, &mask.0, mapsize);
                commands.spawn((
                    //MeshMaterial3d(materials2.add(Color::WHITE)),
                    MeshMaterial3d(mat),
                    //Mesh3d(meshes.add(Cuboid::from_length(10.0))),
                    Mesh3d(mesh),
                    //Transform::from_xyz(30.0, 5.0, 20.0),
                    transform,
                    main_grass,
                    main_data,
                    ContainsPlayer(contains_player)
                ));
            }
        }
        println!("test");
        commands.spawn(grass_grid);
    }
    else {
        let thread_pool = AsyncComputeTaskPool::get();
        let mut grass_grid = grid.get_single_mut().unwrap();
        let elapsed_time = time.elapsed_secs_f64();
        let mut grass_w_player: Option<Entity> = None;
        for (ent, mh, grass_data, grass_trans, visibility, mut contains_player) in grass.iter_mut() {
            let is_outside_draw_distance = (player_trans.translation.x - grass_trans.translation.x).abs() > DESPAWN_DISTANCE || (player_trans.translation.z - grass_trans.translation.z).abs() > DESPAWN_DISTANCE;
            // remove or add ContainsPlayer if applicable
            if (player_trans.translation.x - grass_trans.translation.x).abs() >= GRASS_TILE_SIZE_1/2. || (player_trans.translation.z - grass_trans.translation.z).abs() >= GRASS_TILE_SIZE_1/2. {
                if contains_player.0 {
                    *contains_player = ContainsPlayer(false);
                }
            } else {
                if !contains_player.0 {
                    *contains_player = ContainsPlayer(true);
                    // generate new grass
                    for i in -GRID_SIZE_HALF..=GRID_SIZE_HALF {
                        for j in -GRID_SIZE_HALF..=GRID_SIZE_HALF {
                            let a = grass_trans.translation.x + i as f32 * GRASS_TILE_SIZE_1;
                            let b = grass_trans.translation.z + j as f32 * GRASS_TILE_SIZE_1;
                                let is_outside_draw_distance = (player_trans.translation.x - a).abs() > DESPAWN_DISTANCE || (player_trans.translation.z - b).abs() > DESPAWN_DISTANCE;
                                if is_outside_draw_distance {
                                    continue;
                                }
                                if let false = *grass_grid.0.get(&(a as i32,b as i32)).unwrap_or(&false) {
                                grass_grid.0.insert((a as i32, b as i32), true);
                                // todo: async way
                                let transform = Transform::from_xyz(a,0.,b);
    
                                let task_entity = commands.spawn_empty().id();
                                let height_map = heightmap.clone();
                                let mask = mask.0.clone();
                                let task = thread_pool.spawn(async move {
                                    let mut command_queue = CommandQueue::default();
                                    let (mesh, grass_data) = generate_grass_mesh(a, b, NUM_GRASS_1, GRASS_TILE_SIZE_1, &height_map, &mask, mapsize);
    
                                    command_queue.push(move |world: &mut World| {
                                        let (grass_mesh_handle, grass_mat_handle) = {
                                            let mut system_state = SystemState::<(ResMut<Assets<Mesh>>, ResMut<Assets<ExtendedMaterial<StandardMaterial,GrassMaterialExtension>>>)>::new(world);
                                            let (mut meshes, mut mats) = system_state.get_mut(world);
    
                                            (meshes.add(mesh), mats.add(ExtendedMaterial {
                                                base: grass_material(),
                                                extension: GrassMaterialExtension {}
                                            }))
                                        };

                                        world.entity_mut(task_entity)
                                            .insert((
                                                Mesh3d(grass_mesh_handle),
                                                MeshMaterial3d(grass_mat_handle),
                                                transform,
                                                Grass,
                                                grass_data,
                                                ContainsPlayer(false)
                                            )).remove::<GenGrassTask>();
                                    });
    
                                    command_queue
                                });
    
                                commands.entity(task_entity).insert(GenGrassTask(task)); // spawn a task marked GenGrassTask in the world to be handled by handle_tasks fn when complete

                            //     // old way (sync)
                            //     let (main_mat, main_grass, main_data) = generate_grass(&mut commands, &mut meshes, &mut materials, a, b, NUM_GRASS_1, GRASS_TILE_SIZE_1);
                            //     commands.spawn(main_mat)
                            //         .insert(main_grass)
                            //         .insert(main_data)
                            //         .insert(ContainsPlayer(false))
                            //         // .insert(ShowAabbGizmo {color: Some(Color::PURPLE)})
                            //         ;
                            }
                        }
                    }

                }
            }
            if contains_player.0 {
                grass_w_player = Some(ent);
            }
            // simulate wind only if close enough and if visible
            if (player_trans.translation.x - grass_trans.translation.x).abs() < WIND_SIM_TRIGGER_DISTANCE && (player_trans.translation.z - grass_trans.translation.z).abs() < WIND_SIM_TRIGGER_DISTANCE && visibility.get() {
                // if let Some(mesh) = meshes.get_mut(mh) {
                //     apply_wind(mesh, grass_data, &perlin, elapsed_time, player_trans.translation.xz());
                // }
            } else if is_outside_draw_distance  {
                grass_grid.0.insert((grass_trans.translation.x as i32, grass_trans.translation.z as i32), false);
                commands.get_entity(ent).unwrap().despawn_recursive();
            }
        }

        if let Some(grass_w_player) = grass_w_player {
            // update aabb color
            // commands.get_entity(grass_w_player).unwrap().insert(AabbGizmo {color: Some(Color::RED)});
        }

    }
}

fn handle_tasks(mut commands: Commands, mut grass_tasks: Query<&mut GenGrassTask>) {
    for mut task in &mut grass_tasks {
        if let Some(mut commands_queue) = block_on(poll_once(&mut task.0)) {
            // append the returned command queue to have it execute later
            commands.append(&mut commands_queue);
        }
    }
}

#[derive(Component)]
struct GenGrassTask(Task<CommandQueue>);

#[derive(Resource, Deref)]
struct GrassMeshHandle(Handle<Mesh>);

#[derive(Resource, Deref)]
struct GrassMaterialHandle(Handle<StandardMaterial>);

// 
fn color_gradient_y_based(y: f32, rgba1: [f32; 4], rgba2: [f32; 4]) -> [f32;4] {
    let [r1, g1, b1, a1] = rgba1;
    let [r2, g2, b2, a2] = rgba2;
    let r = r1 + (r2-r1)*(y/GRASS_HEIGHT);
    let g = g1 + (g2-g1)*(y/GRASS_HEIGHT);
    let b = b1 + (b2-b1)*(y/GRASS_HEIGHT);
    let a = a1 + (a2-a1)*(y/GRASS_HEIGHT);
    [r, g, b, a]
}

fn apply_wind(mesh: &mut Mesh, grass: &GrassData, time: f64, player_xz: Vec2) {
    let wind_perlin = Perlin::new(0);
    let pos_attr = mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION).unwrap();
    let VertexAttributeValues::Float32x3(pos_attr) = pos_attr else {
        panic!("Unexpected vertex format, expected Float32x3");
    };
    // for now modify x,z pos. Ideally apply curve instead
    for i in 0..pos_attr.len() {
        let pos = pos_attr.get_mut(i).unwrap(); // current vertex positions
        let initial = grass.initial_vertices.get(i).unwrap(); // initial vertex positions
        let grass_pos = grass.initial_positions.get(i).unwrap(); // initial grass positions

        let [x, y, z] = grass_pos;

        let relative_vertex_height = pos[1] - y;

        let curve_amount = 
            WIND_STRENGTH
            * ((WIND_SIM_DISTANCE - player_xz.distance(Vec2::new(*x,*z))) / WIND_SIM_DISTANCE).powi(2)
            * (sample_noise(&wind_perlin, *x, *z, time) * (relative_vertex_height.powf(CURVE_POWER)/GRASS_HEIGHT.powf(CURVE_POWER)));
        pos[0] = initial.x + curve_amount;
        pos[2] = initial.z + curve_amount;
    }
}

fn sample_noise(perlin: &Perlin, x: f32, z: f32, time: f64) -> f32 {
    WIND_LEAN + perlin.get([WIND_SPEED * time + (x as f64/WIND_CONSISTENCY), WIND_SPEED * time + (z as f64/WIND_CONSISTENCY)]) as f32
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct GrassMaterialExtension {
}

impl MaterialExtension for GrassMaterialExtension {

    fn vertex_shader() -> bevy::render::render_resource::ShaderRef {
        "grass_shader.wgsl".into()
    }
    fn prepass_vertex_shader() -> ShaderRef {
        "grass_shader.wgsl".into()
    }
    

   /*fn fragment_shader() -> ShaderRef {
       "test_shader.wgsl".into()
   }*/



    fn specialize(
            _pipeline: &MaterialExtensionPipeline,
            descriptor: &mut RenderPipelineDescriptor,
            layout: &mesh::MeshVertexBufferLayoutRef,
            _key: MaterialExtensionKey<Self>,
        ) -> Result<(), SpecializedMeshPipelineError> {
        let mut pos_position = 0;
        let mut normal_position = 1;
        let mut color_position = 5;
        if let Some(label) = &mut descriptor.label {
            if label == "pbr_prepass_pipeline" {
                pos_position = 0;
                normal_position = 3;
                color_position = 7;
            }
        }

        let vertex_layout = layout.0.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(pos_position),
            Mesh::ATTRIBUTE_NORMAL.at_shader_location(normal_position),
            Mesh::ATTRIBUTE_COLOR.at_shader_location(color_position),
            // Mesh::ATTRIBUTE_UV_0.at_shader_location(1),
            // Mesh::ATTRIBUTE_TANGENT.at_shader_location(4),
            ATTRIBUTE_BASE_Y.at_shader_location(16),
            ATTRIBUTE_STARTING_POSITION.at_shader_location(17),
            ATTRIBUTE_WORLD_POSITION.at_shader_location(18),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];

        Ok(())
    }
}


pub struct GrassPlugin;

impl Plugin for GrassPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<ExtendedMaterial<StandardMaterial,GrassMaterialExtension>>{
            //prepass_enabled: false, // the current implementation is buggy with prepass enabled and i couldnt figure out how to fix it
            ..Default::default() // its something todo with the globals.time not being passed to the shader in the prepass stage
        });
        app.add_systems(Startup, (init_mask, generate_grass_items.after(super::load_map).after(init_mask)));
        app.add_systems(Update, (update_grass,handle_tasks));
    }
}
#[derive(Resource)]
pub struct BushMask(pub Mask);
pub fn init_mask(
    mut commands: Commands,
    mut materials2: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>
) {
    let mask = Mask::from_png("./assets/011/BushBlurred.png").unwrap();

    commands.insert_resource(BushMask(mask));
}