use bevy::{asset::RenderAssetUsages, math::FloatExt, pbr::NotShadowCaster, render::mesh::{Indices, PrimitiveTopology}, utils::HashMap};
use noise::{utils::{NoiseFnWrapper, NoiseMap, NoiseMapBuilder, PlaneMapBuilder}, Fbm, MultiFractal, NoiseFn, Perlin};
use rand::{rng, Rng};
use voronoice::{BoundingBox, Point, Voronoi, VoronoiBuilder};


pub fn generate_polygons() -> Voronoi{
    let mut rng = rng();
    let map_size = 100.0;
    let (lower, upper) = (map_size /2.0, map_size / 2.0);

    let positions = (0..1000)
        .map(|_| Point {
            x: rng.random_range(-1.0 .. 1.0),
            y: rng.random_range(-1.0 .. 1.0),
        }).collect();
    let voronoi = VoronoiBuilder::default()
        .set_sites(positions)
        .set_bounding_box(BoundingBox::new_centered_square(2.0))
        .set_lloyd_relaxation_iterations(5)
        .build().unwrap();

    voronoi
}

struct CustomMap<F, N>(N, F);
impl<F, N> noise::NoiseFn<f64, 2> for CustomMap<F, N>
where F: Fn([f64;2], f64) -> f64,
N: noise::NoiseFn<f64, 2>
{
    fn get(&self, point: [f64; 2]) -> f64 {
        let value = self.0.get(point);
        (self.1)(point, value)
    }
}

pub fn height_map() -> impl NoiseFn<f64, 2>{
    let initial_noise: Fbm<Perlin> = Fbm::new(3)
        .set_frequency(2.0)
        .set_octaves(3)
        .set_persistence(0.45)
        .set_lacunarity(2.28);
    let mapping = CustomMap(initial_noise, |pos: [f64; 2], value: f64| {
        let [x,y] = pos;
        let scaler = 3.5;
        let d = (1.0 - x * x) * (1.0 - y * y) * scaler - scaler + 1.0;
        //return if value > -0.8 { 1.0 } else { -1.0 };
        let value = (value - 0.).lerp(d, 0.5);
        if value > -0.0 { value } else { -0.0 }
    });
    mapping
}

fn test_noise_func<T: NoiseFn<f64, 2>>(func: &T){
    PlaneMapBuilder::new(func)
    .set_size(100, 100)
    .set_x_bounds(-1.0, 1.0)
    .set_y_bounds(-1.0, 1.0)
    .build()
    .write_to_file("test.png");
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Biome{
    Ocean,
    Lake,
    Beach,
    Grassland,
    Forest,
    Snow
}
impl Biome {
    pub fn new(height: f64) -> Self {
        // dont assign any oceans here, this is handled in a next step
        // all params go from 0 to 1

        // TODO: add more inputs like moisture and others to get even more biomes
        if height < 0.001 {
            Self::Lake
        } else if height < 0.2 {
            Self::Beach
        } else if height < 0.6 {
            Self::Grassland
        } else {
            Self::Snow
        }
    }
    pub fn get_color(&self) -> Color {
        match self {
            Self::Beach => Color::linear_rgb(1.0, 1.0, 1.0),
            Self::Lake => Color::linear_rgb(0.0, 0.0, 1.0),
            Self::Ocean => Color::linear_rgb(0.0, 0.0, 1.0),
            Self::Forest => Color::linear_rgb(0.0, 1.0, 0.0),
            Self::Grassland => Color::linear_rgb(0.0, 1.0, 0.0),
            Self::Snow => Color::WHITE
        }
    }
}


use bevy::prelude::*;
pub fn spawn_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    let polygons = generate_polygons();
    let height_map = height_map();

    let mut biomes = vec![];
    // The order of sites should be the same as their index so later iterations should work
    for point in polygons.sites(){
        let height = height_map.get([point.x, point.y]);
        let biome = Biome::new(height);
        biomes.push(biome);
    }
    // now we get a random point at the edge and run a floodfill to mark all ocean tiles
    //polygons.sites().into_iter().enumerate()
    //    .filter(|(idx, point)| biomes[*idx] == Biome::Lake);
    let mut last_site = 0;
    let resolution = 500;
    let subpart_size = 1.0 / (resolution as f32) * 2.0;
    let mesh = meshes.add(Cuboid::new(subpart_size, subpart_size * 20.0, subpart_size));
    let mut colors = HashMap::new();
    for row in 0..resolution {
        println!("spawned {row}");
        for col in 0..resolution {
            let x = row as f64 / (resolution as f64) * 2.0 - 1.0;
            let y = col as f64 / (resolution as f64) * 2.0 - 1.0;
            let point = Point {x, y};
            let point = polygons.cell(last_site).iter_path(point).last().unwrap();
            last_site = point;

            let height = height_map.get([x,y]);
            let biome = biomes[last_site];
            //let biome = Biome::new(height);
            let color = colors.entry(biome).or_insert_with(||{
                materials.add(biome.get_color())
            });
            commands.spawn((
                Mesh3d(mesh.clone()),
                Transform::from_xyz(x as f32, height as f32, y as f32),
                    //.looking_to(-Vec3::Y, Vec3::Y),
                MeshMaterial3d(color.clone()),
                NotShadowCaster
            ));
        }
    }
}