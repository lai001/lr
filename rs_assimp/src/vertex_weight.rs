use russimp_sys::aiVertexWeight;

pub struct VertexWeight {
    pub vertex_id: u32,
    pub weight: f32,
}

impl VertexWeight {
    pub fn new(c: &aiVertexWeight) -> VertexWeight {
        VertexWeight {
            vertex_id: c.mVertexId,
            weight: c.mWeight,
        }
    }
}
