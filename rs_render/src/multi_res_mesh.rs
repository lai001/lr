use rs_artifact::mesh_vertex::MeshVertex;
use rs_metis::cluster::ClusterCollection;

pub struct MultipleResolutionMesh {
    pub vertexes: Vec<MeshVertex>,
    pub indices: Vec<u32>,
    pub cluster_collection: ClusterCollection,
}
