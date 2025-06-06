use bevy::{
    prelude::*,
    render::{mesh::Indices, render_asset::RenderAssetUsages, render_resource::PrimitiveTopology},
    tasks::{AsyncComputeTaskPool, Task, block_on},
};
use bevy::ecs::world::CommandQueue;

use itertools::Itertools;
use noise::{NoiseFn, Perlin};
use rand::{rng, Rng};
use std::collections::HashSet;

use crate::environment::map::{HeightMap, MapData};
use crate::mask::Mask;

const WHEAT_WIDTH: f32 = 0.05;
const WHEAT_HEIGHT: f32 = 0.7;
const WHEAT_TILE_SIZE: f32 = 3.2;
const WHEAT_DENSITY: u32 = 100;
const WHEAT_RADIUS: i32 = 3;

#[derive(Component)]
pub struct WheatChunk;

#[derive(Component)]
pub struct WheatTask(Task<CommandQueue>);

#[derive(Resource)]
pub struct WheatMask(pub Mask);

#[derive(Resource, Default)]
pub struct WheatGrid(pub HashSet<(i32, i32)>);

pub fn init_wheat_mask(mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>, asset_server: Res<AssetServer>,) {
    let mask = Mask::from_png("assets/011/WheatBlurred.png").unwrap();
    commands.insert_resource(WheatMask(mask));
    commands.insert_resource(WheatGrid::default());

    commands.insert_resource(WheatMaterial{
        0: materials.add(StandardMaterial{
            base_color_texture: Some(asset_server.load("wheat.png")),
            alpha_mode: AlphaMode::Mask(0.5),
            ..Default::default()
        })
    });
}
const DESPAWN_DISTANCE: f32 = 15.0;
pub fn despawn_wheat_chunks(
    mut commands: Commands,
    player_query: Single<&Transform, With<Camera3d>>,
    mut wheat_grid: ResMut<WheatGrid>,
    mapdata: Res<MapData>,
    wheat: Query<(Entity, &Transform, &WheatChunk)>
) {
    let half_map = mapdata.size / 2.0;
    let player_trans = player_query.into_inner();
    for (id, grass_trans, _) in wheat.iter(){
        let is_outside_draw_distance = (player_trans.translation.x - grass_trans.translation.x).abs() > DESPAWN_DISTANCE || (player_trans.translation.z - grass_trans.translation.z).abs() > DESPAWN_DISTANCE;
        if is_outside_draw_distance {
            let tile_x = ((grass_trans.translation.x + half_map) / WHEAT_TILE_SIZE) as i32;
            let tile_y = ((grass_trans.translation.z + half_map) / WHEAT_TILE_SIZE) as i32;
            wheat_grid.0.remove(&(tile_x, tile_y));
            commands.get_entity(id).unwrap().despawn_recursive();
        }
    }
}

pub fn update_wheat_chunks(
    mut commands: Commands,
    height_map: Res<HeightMap>,
    wheat_mask: Res<WheatMask>,
    mapdata: Res<MapData>,
    wheatmat: Res<WheatMaterial>,
    mut wheat_grid: ResMut<WheatGrid>,
    player_query: Query<&Transform, With<Camera3d>>,
) {
    let player_pos = player_query.single().translation;

    let half_map = mapdata.size / 2.0;
    let player_tile_x = ((player_pos.x + half_map) / WHEAT_TILE_SIZE).floor() as i32;
    let player_tile_z = ((player_pos.z + half_map) / WHEAT_TILE_SIZE).floor() as i32;

    let task_pool = AsyncComputeTaskPool::get();

    for dx in -WHEAT_RADIUS..=WHEAT_RADIUS {
        for dz in -WHEAT_RADIUS..=WHEAT_RADIUS {
            let tile_x = player_tile_x + dx;
            let tile_z = player_tile_z + dz;

            if !wheat_grid.0.contains(&(tile_x, tile_z)) {
                let tile_world_x = tile_x as f32 * WHEAT_TILE_SIZE - half_map;
                let tile_world_z = tile_z as f32 * WHEAT_TILE_SIZE - half_map;
                let height_map = height_map.clone();
                let wheat_mask = wheat_mask.0.clone();
                let mapsize = mapdata.size;

                wheat_grid.0.insert((tile_x, tile_z));
                let mat = wheatmat.0.clone();

                let task = task_pool.spawn(async move {
                    let mesh = generate_wheat_chunk(tile_world_x, tile_world_z, &height_map, &wheat_mask, mapsize);
                    let transform = Transform::from_xyz(tile_world_x, 0.0, tile_world_z);

                    let mut command_queue = CommandQueue::default();
                    command_queue.push(move |world: &mut World| {
                        let mesh_handle;
                        {
                            let mut meshes = world.resource_mut::<Assets<Mesh>>();
                            //mesh_handle = meshes.add(Cuboid::from_length(1.0));
                            mesh_handle = meshes.add(mesh);
                        }
                        let material_handle;
                        {
                            let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
                            material_handle = materials.add(StandardMaterial {
                                base_color: Color::srgb(0.96, 0.87, 0.70),
                                double_sided: true,
                                unlit: false,
                                alpha_mode: AlphaMode::Opaque,
                                ..default()
                            });
                        }
                        world.spawn((
                            Mesh3d(mesh_handle),
                            MeshMaterial3d(mat.clone()),
                            transform,
                            WheatChunk,
                        ));
                    });

                    command_queue
                });

                commands.spawn(WheatTask(task));
            }
        }
    }
}

pub fn handle_finished_wheat_tasks(
    mut commands: Commands,
    mut query: Query<(Entity, &mut WheatTask)>,
) {
    for (entity, mut task) in query.iter_mut() {
        if task.0.is_finished() {
            let mut command_queue = block_on(&mut task.0);
            commands.append(&mut command_queue);
            commands.entity(entity).despawn();
        }
    }
}

const WIDTH: f32 = 0.05;
const HEIGHT: f32 = 0.36;
const STRAIGHTNESS: f32 = 10.0;
const GRASS_HEIGHT_VARIATION_FACTOR: f32 = 0.02;
pub fn generate_single_blade_verts(x: f32, y: f32, z: f32, blade_number: u32, blade_height: f32) -> (Vec<Vec3>, Vec<u32>, Vec<Vec3>, Vec<Vec3>) {
    // For grass with 7 vertices, uncomment t3-6, and uncomment indices
    // vertex transforms
    let t1 = Vec3::new(x, y, z);
    let t2 = Vec3::new(x+WIDTH, y, z);
    // let t3 = Transform::from_xyz(x, y+blade_height/3.0, z);
    // let t4 = Transform::from_xyz(x+GRASS_WIDTH, y+blade_height/3.0, z);
    // let t5 = Transform::from_xyz(x, y+2.0*blade_height/3.0, z);
    // let t6 = Transform::from_xyz(x + GRASS_WIDTH, y+2.0*blade_height/3.0, z);
    let t7 = Vec3::new(x, y+blade_height, z);
    let t8 = Vec3::new(x+WIDTH, y+blade_height, z);
    //let t7 = Vec3::new(x+(WIDTH/2.0), y+blade_height, z);
    
    // let mut transforms = vec![t1,t2,t3,t4,t5,t6,t7];
    // let mut transforms = vec![t1,t2,t5,t6,t7];
    let mut vertices = vec![t1,t2,t7,t8];

    let normals = vertices.iter().map(|_| Vec3::new(0.0, 0.0, 1.0)).collect::<Vec<_>>();
    let uvs = vertices.iter().map(|vert| {
        let mut out = *vert;
        out.x = (out.x - x) / WIDTH;
        out.y = 1.0 - (out.y - y) / blade_height;
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
        transform.rotate_around(curve_rotation_point, Quat::from_rotation_z(rand_curve * ((transform.translation.y - y) / (HEIGHT*STRAIGHTNESS))));
        *pt = transform.translation;
    }
    

    // rotate grass again
    let rot1 = rng.random_range(0..628) as f32 / 100.0;
    for pt in &mut vertices {
        *pt = rotate_point(*pt, rotation_point, rot1);
    }
    

    let indices: Vec<u32> = vec![
        blade_number_shift+0, blade_number_shift+1, blade_number_shift+2,
        blade_number_shift+1, blade_number_shift+3, blade_number_shift+2,
        // blade_number_shift+2, blade_number_shift+1, blade_number_shift+3,
        // blade_number_shift+2, blade_number_shift+3, blade_number_shift+4,
        // blade_number_shift+4, blade_number_shift+3, blade_number_shift+5,
        // blade_number_shift+4, blade_number_shift+5, blade_number_shift+6,
    ];
    (vertices, indices, normals, uvs)
}

#[derive(Resource)]
pub struct WheatMaterial(Handle<StandardMaterial>);

fn generate_wheat_chunk(spawn_x: f32, spawn_z: f32, heightmap: &HeightMap, mask: &Mask, mapsize: f32) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD);
    let mut vertices = vec![];
    let mut indices = vec![];
    let mut wheat_offsets = vec![];
    let mut uvs = vec![];
    let mut all_colors: Vec<[f32; 4]> = vec![];

    let mut rng = rng();

    let mut attempted = 0;
    let height_perlin = Perlin::new(0);
    let mut successful = 0;
    let mut blade_number = 0;

    let start_x = - WHEAT_TILE_SIZE/2.;
    let start_z = - WHEAT_TILE_SIZE/2.;

    for i in 0..WHEAT_DENSITY {
        for j in 0..WHEAT_DENSITY {
            let frac_x = start_x + i as f32 * WHEAT_TILE_SIZE / WHEAT_DENSITY as f32;
            let frac_z = start_z + j as f32 * WHEAT_TILE_SIZE / WHEAT_DENSITY as f32;

            let x_offset = frac_x; // + rand1;
            let z_offset = frac_z; // + rand2;

            let t = 0.0;
            let x_sample = (spawn_x + x_offset) / (mapsize - t);
            let z_sample = (spawn_z + z_offset) / (mapsize - t);

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
            let blade_height = HEIGHT + (height_perlin.get([(spawn_x + x_offset) as f64, (spawn_z + z_offset) as f64]) as f32 * GRASS_HEIGHT_VARIATION_FACTOR);
            let (mut verts, mut indices2, mut normals, mut uvs2) = generate_single_blade_verts(x_offset, y, z_offset, blade_number, blade_height);
            if verts.len() == 0 {
            }
            for v in &verts {
                wheat_offsets.push([spawn_x + x_offset,y,spawn_z + z_offset]);
                let r_color_shift = 0.0;
                //let r_color_shift = (terrain_perlin.get([(spawn_x + x_offset) as f64 / 100., (spawn_z + z_offset) as f64 / 100.]) * 0.01) as f32;
                //let mut color = color_gradient_y_based(v.y-y, GRASS_BASE_COLOR_2, GRASS_SECOND_COLOR);
                let mut color = [1.0; 4];
                color[0] += r_color_shift;
                color[3] = 0.3;
                all_colors.push(color);
            }
            vertices.append(&mut verts);
            indices.append(&mut indices2);
            uvs.append(&mut uvs2);
            blade_number += 1;
        }
    }

    // DEBUG: na de loop
    if(successful!=0){
        println!("Attempted placements: {}, Successful wheat placements: {}", attempted, successful);
    }
    let vertices: Vec<([f32;3],[f32;3],[f32;2])> = vertices.iter().map(|v| { (v.to_array(), [0., 0., 1.], [0.0,0.0])} ).collect();

    let mut positions = Vec::with_capacity(vertices.capacity());
    let mut normals = Vec::with_capacity(vertices.capacity());
    let bases: Vec<f32> = wheat_offsets.iter().map(|x| x[1]).collect();
    // let mut uvs: Vec<[f32; 2]> = Vec::new();
    for (position, normal, uv) in vertices.iter() {
        positions.push(*position);
        normals.push(*normal);
    }
    let mut uvs = uvs.into_iter().map(|x| x.xy().to_array()).collect_vec();


    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));

    mesh


}


pub struct WheatPlugin;

impl Plugin for WheatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init_wheat_mask)
           .add_systems(Update, (
               update_wheat_chunks.after(crate::environment::map::load_map),
               handle_finished_wheat_tasks,
               despawn_wheat_chunks
           ));
    }
}
