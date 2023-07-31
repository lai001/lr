use crate::{
    brigde_data::mesh_vertex::MeshVertex, material_type::EMaterialType,
    primitive_data::PrimitiveData, util,
};

pub struct Mesh {
    vertex_buffer: Vec<MeshVertex>,
    index_buffer: Vec<u32>,
}

impl Mesh {
    pub fn new(vertex_buffer: Vec<MeshVertex>, index_buffer: Vec<u32>) -> Mesh {
        Mesh {
            vertex_buffer,
            index_buffer,
        }
    }

    pub fn quad() -> Mesh {
        let data = PrimitiveData::quad();
        Mesh {
            vertex_buffer: data.vertices,
            index_buffer: data.indices,
        }
    }

    pub fn triangles_view(&self) -> Vec<(&glam::Vec3, &glam::Vec3, &glam::Vec3)> {
        assert_eq!(self.index_buffer.len() % 3, 0);
        assert_eq!(self.vertex_buffer.len(), self.index_buffer.len());
        let mut triangles: Vec<(&glam::Vec3, &glam::Vec3, &glam::Vec3)> = vec![];
        for i in (0..self.index_buffer.len()).step_by(3) {
            if let (Some(a), Some(b), Some(c)) = (
                self.index_buffer.get(i + 0),
                self.index_buffer.get(i + 1),
                self.index_buffer.get(i + 2),
            ) {
                triangles.push((
                    &self.vertex_buffer.get(*a as usize).unwrap().position,
                    &self.vertex_buffer.get(*b as usize).unwrap().position,
                    &self.vertex_buffer.get(*c as usize).unwrap().position,
                ));
            }
        }
        triangles
    }
}

pub struct MeshBuffer {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
}

impl MeshBuffer {
    pub fn get_vertex_buffer(&self) -> &wgpu::Buffer {
        &self.vertex_buffer
    }

    pub fn get_index_buffer(&self) -> &wgpu::Buffer {
        &self.index_buffer
    }

    pub fn get_index_count(&self) -> u32 {
        self.index_count
    }

    pub fn from(device: &wgpu::Device, mesh: &Mesh) -> MeshBuffer {
        let vertex_buffer = util::create_gpu_vertex_buffer_from(device, &mesh.vertex_buffer, None);
        let index_buffer = util::create_gpu_index_buffer_from(device, &mesh.index_buffer, None);
        let buffer = MeshBuffer {
            vertex_buffer,
            index_buffer,
            index_count: mesh.index_buffer.len() as u32,
        };
        buffer
    }
}

pub struct StaticMesh {
    name: String,
    mesh: Mesh,
    mesh_buffer: MeshBuffer,
    material_type: EMaterialType,
}

impl StaticMesh {
    pub fn new(
        name: &str,
        mesh: Mesh,
        device: &wgpu::Device,
        material_type: EMaterialType,
    ) -> StaticMesh {
        let mesh_buffer = MeshBuffer::from(device, &mesh);
        StaticMesh {
            name: name.to_string(),
            mesh,
            mesh_buffer,
            material_type,
        }
    }

    pub fn quad(name: &str, device: &wgpu::Device, material_type: EMaterialType) -> StaticMesh {
        let mesh = Mesh::quad();
        let mesh_buffer = MeshBuffer::from(device, &mesh);
        StaticMesh {
            name: name.to_string(),
            mesh,
            mesh_buffer,
            material_type,
        }
    }

    pub fn get_material_type(&self) -> &EMaterialType {
        &self.material_type
    }

    pub fn get_mesh_buffer(&self) -> &MeshBuffer {
        &self.mesh_buffer
    }

    pub fn get_name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn get_triangles_view(&self) -> Vec<(&glam::Vec3, &glam::Vec3, &glam::Vec3)> {
        self.mesh.triangles_view()
    }

    pub fn set_material_type(&mut self, material_type: EMaterialType) {
        self.material_type = material_type;
    }
}
