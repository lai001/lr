use rs_artifact::{
    artifact::{encode_artifact_assets_disk, ArtifactReader},
    build_asset_url, default_url,
    resource_type::EResourceType,
    shader_source_code::ShaderSourceCode,
    EEndianType,
};
use rs_editor::model_loader::ModelLoader;
use rs_engine::logger::LoggerConfiguration;

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
                url: default_url().clone(),
            };
            bincode::serialize(&static_mesh);
        }
    }

    let url = build_asset_url("attachment", EResourceType::ShaderSourceCode).unwrap();
    let shader_source_code = ShaderSourceCode {
        name: "attachment".to_string(),
        id: uuid::Uuid::new_v4(),
        code: std::fs::read_to_string("../../../rs_computer_graphics/src/shader/attachment.wgsl")
            .unwrap(),
        url: url.clone(),
    };
    let save_path = std::path::Path::new("./test.rs");
    assert_eq!(
        encode_artifact_assets_disk(&[shader_source_code], Some(EEndianType::Little), save_path),
        true
    );

    match ArtifactReader::new(save_path, Some(EEndianType::Little)) {
        Ok(mut artifact_reader) => {
            assert_eq!(artifact_reader.check_assets(), true);
            let shader_source_code: ShaderSourceCode = artifact_reader
                .get_resource(&url, Some(EResourceType::ShaderSourceCode))
                .unwrap();
            log::trace!("{}", shader_source_code.code);
        }
        Err(err) => {
            log::warn!("{:?}", err);
        }
    }
    logger.flush();
}
