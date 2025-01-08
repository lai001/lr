use std::num::NonZeroUsize;

#[derive(Default, Clone)]
pub struct MeshoptMesh {
    pub vertices: Vec<meshopt::Vertex>,
    pub indices: Vec<u32>,
}

impl MeshoptMesh {
    pub fn vertex_adapter(&self) -> meshopt::VertexDataAdapter {
        let position_offset = std::mem::offset_of!(meshopt::Vertex, p);
        let vertex_stride = std::mem::size_of::<meshopt::Vertex>();
        let vertex_data = meshopt::typed_to_bytes(&self.vertices);

        meshopt::VertexDataAdapter::new(vertex_data, vertex_stride, position_offset)
            .expect("failed to create vertex data reader")
    }
}

pub fn simplify(mesh: &MeshoptMesh, lod_count: NonZeroUsize) -> Vec<Vec<u32>> {
    let lod_count = lod_count.get();

    let vertex_adapter = mesh.vertex_adapter();

    let mut lods: Vec<Vec<u32>> = Vec::with_capacity(lod_count);
    lods.push(mesh.indices.clone());

    for i in 1..lod_count {
        let threshold = 0.7f32.powf(i as f32);
        let target_index_count = (mesh.indices.len() as f32 * threshold) as usize / 3 * 3;
        let target_error = 1e-3f32;
        let lod: Vec<u32>;
        {
            let src = &lods[lods.len() - 1];
            lod = meshopt::simplify(
                src,
                &vertex_adapter,
                std::cmp::min(src.len(), target_index_count),
                target_error,
                meshopt::SimplifyOptions::None,
                None,
            );
        }
        lods.push(lod);
    }

    for lod in &mut lods {
        meshopt::optimize_vertex_cache_in_place(lod, vertex_adapter.vertex_count);
        meshopt::optimize_overdraw_in_place(lod, &vertex_adapter, 1f32);
    }
    lods
}
