use bevy::{
    prelude::*,
    render::mesh::{Indices, VertexAttributeValues},
};

pub struct MeshBuilder {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    indices: Vec<u32>,
    offset: u32,
}
impl MeshBuilder {
    pub fn new() -> Self {
        Self {
            positions: Vec::new(),
            normals: Vec::new(),
            uvs: Vec::new(),
            indices: Vec::new(),
            offset: 0,
        }
    }

    pub fn add_mesh(&mut self, mesh: &Mesh, transform: Transform) {
        if let Some(VertexAttributeValues::Float32x3(positions)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        {
            self.positions.extend(
                positions
                    .iter()
                    .map(|p| transform * Vec3::from_array(*p))
                    .map(|p| p.to_array()),
            );
        }

        if let Some(VertexAttributeValues::Float32x3(normals)) =
            mesh.attribute(Mesh::ATTRIBUTE_NORMAL)
        {
            self.normals.extend(normals);
        }
        if let Some(VertexAttributeValues::Float32x2(uvs)) = mesh.attribute(Mesh::ATTRIBUTE_UV_0) {
            self.uvs.extend(uvs);
        }
        if let Some(Indices::U32(indices)) = mesh.indices() {
            self.indices.extend(indices.iter().map(|i| i + self.offset));
        }
        self.offset += mesh.count_vertices() as u32;
    }
    pub fn build(self) -> Mesh {
        let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs);
        mesh.set_indices(Some(Indices::U32(self.indices)));
        mesh
    }
}

impl Default for MeshBuilder {
    fn default() -> Self {
        Self::new()
    }
}
