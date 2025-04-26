use bevy::{
    asset::RenderAssetUsages, gltf::{GltfMesh, GltfNode}, pbr::{NotShadowCaster, NotShadowReceiver}, prelude::*, render::view::VisibilityRange, tasks::{AsyncComputeTaskPool, ComputeTaskPool, TaskPool}
};
use pathfinding::matrix::directions::N;
use rand::{rng, Rng, SeedableRng};

use crate::mask::{BlueNoise, Mask};

use super::map::HeightMap;

#[derive(Component)]
pub enum ModelType{
    Main,
    Lod,
    Billboard
}
pub struct Tree{
    main: Handle<Scene>,
    lod: Handle<Scene>,
    billboard: Handle<Scene>,
    scaler: f32
}
impl Tree {
    pub fn new(asset_server: &AssetServer, path: &str, scaler: f32) -> Self {
        let p1 = format!("{path}/main.glb#Scene0");
        let p2 = format!("{path}/lod.glb#Scene0");
        let p3 = format!("{path}/billboard.glb#Scene0");
        let main = asset_server.load(p1);
        let lod = asset_server.load(p2);
        let billboard = asset_server.load(p3);
        Self {
            main,
            lod,
            billboard,
            scaler
        }
    }
}


/*#[derive(Resource)]
pub struct TerrainResources {
    // material: Handle<StandardMaterial>,
    // tree: Handle<Scene>,
    trees_gltf: Handle<Gltf>,
    trees: Vec<Handle<Scene>>,
}
pub fn setup_trees(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(TerrainResources {
        // material: asset_server.load("forest_ground/forest_ground_04_4k.gltf#Material0"),
        // tree: asset_server.load("japanese_spruce_trees.glb#Scene3"),
        trees_gltf: asset_server.load("fir_tree_stylized.glb"),
        trees: vec![],
    });
}

pub fn on_terrain_resource_loaded(
    mut terrain_resources: ResMut<TerrainResources>,
    gltf_assets: Res<Assets<Gltf>>,
    gltf_nodes: Res<Assets<GltfNode>>,
    gltf_meshes: Res<Assets<GltfMesh>>,
    mut scenes: ResMut<Assets<Scene>>,
    mut terrain_config: ResMut<TerrainConfig>,
    mut loaded: Local<bool>,
) {
    if *loaded {
        return;
    }
    let Some(trees_gltf) = gltf_assets.get(&terrain_resources.trees_gltf) else {
        return;
    };

    // tree 0
    let mut scene_world = World::new();
    let gltf_node = gltf_nodes.get(&trees_gltf.named_nodes["Branches"]).unwrap();
    spawn_gltf_node(&mut scene_world, gltf_node, &gltf_meshes);
    let gltf_node = gltf_nodes
        .get(&trees_gltf.named_nodes["Tree_bark"])
        .unwrap();
    spawn_gltf_node(&mut scene_world, gltf_node, &gltf_meshes);
    let scene_handle = scenes.add(Scene::new(scene_world));
    terrain_resources.trees.push(scene_handle);

    // tree 1
    let mut scene_world = World::new();
    let gltf_node = gltf_nodes
        .get(&trees_gltf.named_nodes["Branches001"])
        .unwrap();
    spawn_gltf_node(&mut scene_world, gltf_node, &gltf_meshes);
    let gltf_node = gltf_nodes
        .get(&trees_gltf.named_nodes["Tree_bark001"])
        .unwrap();
    spawn_gltf_node(&mut scene_world, gltf_node, &gltf_meshes);
    let scene_handle = scenes.add(Scene::new(scene_world));
    terrain_resources.trees.push(scene_handle);

    // tree 2
    let mut scene_world = World::new();
    let gltf_node = gltf_nodes
        .get(&trees_gltf.named_nodes["Branches002"])
        .unwrap();
    spawn_gltf_node(&mut scene_world, gltf_node, &gltf_meshes);
    let gltf_node = gltf_nodes
        .get(&trees_gltf.named_nodes["Tree_bark002"])
        .unwrap();
    spawn_gltf_node(&mut scene_world, gltf_node, &gltf_meshes);
    let scene_handle = scenes.add(Scene::new(scene_world));
    terrain_resources.trees.push(scene_handle);

    terrain_config.set_changed();

    println!("tree scene loaded");
    *loaded = true;
}

fn spawn_gltf_node(scene: &mut World, gltf_node: &GltfNode, gltf_meshes: &Assets<GltfMesh>) {
    if let Some(gltf_mesh) = &gltf_node.mesh {
        spawn_gltf_mesh(scene, gltf_mesh, gltf_meshes);
    }
    // recursion stops once there are no children
    for gltf_node in &gltf_node.children {
        spawn_gltf_node(scene, gltf_node, gltf_meshes);
    }
}

fn spawn_gltf_mesh(
    scene: &mut World,
    gltf_mesh: &Handle<GltfMesh>,
    gltf_meshes: &Assets<GltfMesh>,
) {
    let gltf_mesh = gltf_meshes.get(gltf_mesh).unwrap();
    for primitive in &gltf_mesh.primitives {
        scene.spawn(PbrBundle {
            mesh: primitive.mesh.clone(),
            material: if let Some(mat) = primitive.material.as_ref() {
                mat.clone()
            } else {
                Default::default()
            },
            ..default()
        });
    }
}*/

pub fn spawn_trees(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    height_map: Res<HeightMap>,
) {
    // this dumb small piece of code took so much work :(
    // the assets i got were gltfs and adding so many of them bloated the ecs so it started lagging
    // i had to learn blender baking and texture merging to get this to work :(

    /*let obj = asset_server.load("reallistic_pine_tree_6/tree1.obj");
    let texture = asset_server.load("tree/OutputTexture.png");
    let material = materials.add(StandardMaterial {
        base_color_texture: Some(texture),
        alpha_mode: AlphaMode::Mask(0.5),
        ..Default::default()
    });*/
    let birch = Tree::new(&asset_server, "birch1", 2.0);
    let oak = Tree::new(&asset_server, "oak1", 1.0);
    let trees = [birch, oak];

    let mask = Mask::from_file("assets/002/Oakmap.r32", 1024).expect("failed to find mask");
    let noise = BlueNoise::new(0.006, mask);
    let mut rng = rng();
    
    for [x, y] in noise.take(1000) { // generate 5000 trees at a minimum distance of 0.002 from eachother
        let mut local_rng = rand::prelude::SmallRng::seed_from_u64((x * y + y) as u64);
        let height = height_map.height_at(x, y);
        let tree_used = rng.random_range(0..trees.len());
        let tree_used = &trees[tree_used];
        let rand_transform = Transform::from_xyz(x * 100.0, height * 100.0, y * 100.0)
            .with_rotation(Quat::from_rotation_y(local_rng.random_range(0.0..6.24)))
            .with_scale(
                Vec3::splat(0.1)
                    * Vec3::new(
                        local_rng.random_range(0.9..1.1) * tree_used.scaler,
                        local_rng.random_range(0.9..1.1) * tree_used.scaler,
                        local_rng.random_range(0.9..1.1) * tree_used.scaler,
                    ),
            );
        commands.spawn((rand_transform, Visibility::Visible))
            .with_children(|children| {
                children.spawn((
                    SceneRoot(tree_used.main.clone()),
                    ModelType::Main,
                ));
                children.spawn((
                    SceneRoot(tree_used.lod.clone()),
                    ModelType::Lod,
                ));
                children.spawn((
                    SceneRoot(tree_used.billboard.clone()),
                    ModelType::Billboard,
                ));
            });
        /*commands.spawn((
            SceneRoot(tree.base.clone()),
            tree.clone(),
            //Mesh3d(obj.clone()),
            //MeshMaterial3d(material.clone()),
            rand_transform,
        ));*/
    }


    let birch = Tree::new(&asset_server, "fir_big", 3.0);
    let oak = Tree::new(&asset_server, "fir_small", 1.0);
    let trees = [birch, oak];

    let mask = Mask::from_png("assets/011/PinemapBlurred.png").expect("failed to find mask");
    let noise = BlueNoise::new(0.006, mask);
    
    for [x, y] in noise.take(1000) { // generate 5000 trees at a minimum distance of 0.002 from eachother
        let mut local_rng = rand::prelude::SmallRng::seed_from_u64((x * y + y) as u64);
        let height = height_map.height_at(x, y);
        let tree_used = rng.random_range(0..trees.len());
        let tree_used = &trees[tree_used];
        let rand_transform = Transform::from_xyz(x * 100.0, height * 100.0, y * 100.0)
            .with_rotation(Quat::from_rotation_y(local_rng.random_range(0.0..6.24)))
            .with_scale(
                Vec3::splat(0.1)
                    * Vec3::new(
                        local_rng.random_range(0.9..1.1),
                        local_rng.random_range(0.9..1.1) * tree_used.scaler,
                        local_rng.random_range(0.9..1.1),
                    ),
            );
        commands.spawn((rand_transform, Visibility::Visible))
            .with_children(|children| {
                children.spawn((
                    SceneRoot(tree_used.main.clone()),
                    ModelType::Main,
                ));
                children.spawn((
                    SceneRoot(tree_used.lod.clone()),
                    ModelType::Lod,
                ));
                children.spawn((
                    SceneRoot(tree_used.billboard.clone()),
                    ModelType::Billboard,
                ));
            });
        /*commands.spawn((
            SceneRoot(tree.base.clone()),
            tree.clone(),
            //Mesh3d(obj.clone()),
            //MeshMaterial3d(material.clone()),
            rand_transform,
        ));*/
    }
}

// https://bevyengine.org/examples/3d-rendering/visibility-range/
pub fn set_visibility_and_merge_meshes(
    mut commands: Commands,
    mut new_meshes: Query<(Entity, &MeshMaterial3d<StandardMaterial>), Added<Mesh3d>>,
    parents: Query<(Option<&Parent>, Option<&ModelType>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (new_mesh, material) in new_meshes.iter_mut() {
        let (mut current, mut main_model) = (new_mesh, None);
        while let Ok((parent, maybe_main_model)) = parents.get(current) {
            if let Some(model) = maybe_main_model {
                main_model = Some(model);
                break;
            }
            match parent {
                Some(parent) => current = **parent,
                None => break
            }
        }
        let Some(main_model) = main_model else {
            continue;
        };
        let mat = materials.get_mut(material).unwrap();
        mat.alpha_mode = AlphaMode::Mask(0.5);

        match main_model {
            ModelType::Main => {
                commands
                    .entity(new_mesh)
                    .insert(VisibilityRange {
                        use_aabb: false,
                        start_margin: 0.0..0.0,
                        end_margin: 10.0..15.0
                    })
                    .insert(ModelType::Main);
            }
            ModelType::Lod => {
                commands
                    .entity(new_mesh)
                    .insert(VisibilityRange {
                        use_aabb: false,
                        start_margin: 10.0..15.0,
                        end_margin: 30.0..40.0
                    })
                    .insert(ModelType::Lod);
            },
            ModelType::Billboard => {
                commands
                .entity(new_mesh)
                .insert(VisibilityRange {
                    use_aabb: false,
                    start_margin: 30.0..40.0,
                    end_margin: 90.0..100.0
                })
                .insert(ModelType::Billboard);
            }
        }

    }
}