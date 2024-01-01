use rs_editor::model_loader::ModelLoader;
use rs_engine::logger::LoggerConfiguration;
use russimp::scene::PostProcessSteps;

fn main() {
    let path = rs_foundation::change_working_directory();
    let logger = rs_engine::logger::Logger::new(LoggerConfiguration {
        is_write_to_file: false,
    });
    let remote_dir = std::path::Path::new("../../../Resource/Remote");
    let fbx_filename = "Monkey.fbx";
    let fbx_file_path = rs_foundation::absolute_path(remote_dir.join(fbx_filename));
    if let Ok(fbx_file_path) = fbx_file_path {
        let mesh_clusters = ModelLoader::load_from_file(fbx_file_path.to_str().unwrap(), &[]);
        for mesh_cluster in mesh_clusters {
            log::trace!("{}", mesh_cluster.vertex_buffer.len());
            log::trace!("{}", mesh_cluster.index_buffer.len());
            log::trace!("{}", mesh_cluster.textures_dic.len());
            let static_mesh = rs_artifact::static_mesh::StaticMesh {
                name: "".to_string(),
                id: uuid::Uuid::new_v4(),
                vertexes: mesh_cluster.vertex_buffer,
                indexes: mesh_cluster.index_buffer,
            };
            bincode::serialize(&static_mesh);
        }
    }
    logger.flush();
}
