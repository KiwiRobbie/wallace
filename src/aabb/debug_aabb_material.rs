use bevy::{
    asset::Asset,
    pbr::MaterialExtension,
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderRef},
};

#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct DebugAabbMaterial {
    // Start at a high binding number to ensure bindings don't conflict
    // with the base material
    #[uniform(100)]
    pub quantize_steps: u32,
}

impl MaterialExtension for DebugAabbMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/debug_aabb.wgsl".into()
    }
    fn vertex_shader() -> ShaderRef {
        "shaders/debug_aabb.wgsl".into()
    }
}
