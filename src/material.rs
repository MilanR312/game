use bevy::{pbr::MaterialExtension, prelude::*, render::render_resource::{AsBindGroup, ShaderRef}};

/// A custom material that allows texture repeating and blending between multiple textures using a single mask/splatmap
/// 
/// the splatmap uses the r and g channels to define which texture/normal/depth to choose and the b channel to blend
#[derive(Asset, AsBindGroup, Clone, TypePath)]
pub struct TerrainMaterialExtension {
    #[texture(100, dimension = "2d", sample_type = "u_int")]
    #[sampler(101, sampler_type = "non_filtering")]
    pub splatmap: Handle<Image>,

    #[texture(102, dimension = "2d")]
    #[sampler(103)]
    pub filtered_splatmap: Handle<Image>,

    #[texture(104, dimension = "2d_array")]
    #[sampler(105)]
    pub textures: Handle<Image>,

    #[texture(106, dimension = "2d_array")]
    #[sampler(107)]
    pub normals: Handle<Image>,

    #[texture(108, dimension = "2d_array")]
    #[sampler(109)]
    pub parallax: Handle<Image>
}
impl MaterialExtension for TerrainMaterialExtension {
    fn fragment_shader() -> ShaderRef {
        "blend_shader.wgsl".into()
    }
}