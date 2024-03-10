use crate::{
    convert::ConvertToString, error::Result, get_assimp_error, material::Material, mesh::Mesh,
    node::Node, post_process_steps::PostProcessSteps,
};
use russimp_sys::*;
use std::{
    cell::RefCell, collections::HashMap, ffi::CString, marker::PhantomData, path::Path, rc::Rc,
};

fn walk_ai_node<'a, F>(node: &'a mut aiNode, f: &mut F)
where
    F: FnMut(&'a mut aiNode) -> (),
{
    let children = rs_foundation::get_vec_from_raw_mut(node.mChildren, node.mNumChildren);
    for item in children {
        walk_ai_node(item, f);
    }
    f(node);
}

fn collect_all_nodes<'a>(scene: &'a aiScene) -> HashMap<String, Rc<RefCell<Node<'a>>>> {
    let mut nodes: HashMap<String, Rc<RefCell<Node<'a>>>> = HashMap::new();
    if scene.mRootNode == std::ptr::null_mut() {
    } else {
        walk_ai_node(unsafe { scene.mRootNode.as_mut().unwrap() }, &mut |child| {
            nodes.insert(
                child.mName.to_string(),
                Rc::new(RefCell::new(Node::new(child))),
            );
        });
    }
    nodes
}

fn build_node_tree(map: &mut HashMap<String, Rc<RefCell<Node>>>) {
    let mut binding = map.clone();
    let nodes = binding.values_mut();
    for node in nodes {
        let mut node = node.borrow_mut();
        node.parent_and_leaf(map);
    }
}

pub struct Scene<'a> {
    c: *const russimp_sys::aiScene,
    pub name: String,
    pub meshes: Vec<Mesh<'a>>,
    pub root_node: Option<Rc<RefCell<Node<'a>>>>,
    all_nodes: HashMap<String, Rc<RefCell<Node<'a>>>>,
    pub materials: Vec<Material<'a>>,
    marker: PhantomData<&'a ()>,
}

impl<'a> Scene<'a> {
    pub fn from_file<P: AsRef<Path>>(path: P, flags: PostProcessSteps) -> Result<Scene<'a>> {
        let path = path.as_ref();
        let path = path.as_os_str().as_encoded_bytes();
        let path = CString::new(path).map_err(|err| crate::error::Error::Nul(err))?;
        unsafe {
            let ai_scene = russimp_sys::aiImportFile(path.as_ptr(), flags.bits());
            if ai_scene == std::ptr::null() {
                return Err(crate::error::Error::Assimp(get_assimp_error()));
            }
            let ai_scene = ai_scene.as_ref().unwrap();

            let mut all_nodes = collect_all_nodes(ai_scene);
            build_node_tree(&mut all_nodes);

            let root_node = match ai_scene.mRootNode.as_mut() {
                Some(m_root_node) => {
                    let name = m_root_node.mName.to_string();
                    all_nodes.get(&name).cloned()
                }
                None => None,
            };

            let slice =
                std::ptr::slice_from_raw_parts(ai_scene.mMeshes, ai_scene.mNumMeshes as usize)
                    .as_ref()
                    .unwrap();
            let mut meshes = Vec::new();
            for mesh in slice {
                let mut mesh = Mesh::borrow_from(mesh.as_mut().unwrap());
                for bone in &mut mesh.bones {
                    bone.execute(&mut all_nodes);
                }
                meshes.push(mesh);
            }

            let slice = std::ptr::slice_from_raw_parts(
                ai_scene.mMaterials,
                ai_scene.mNumMaterials as usize,
            )
            .as_ref()
            .unwrap();
            let mut materials: Vec<Material<'a>> = vec![];
            for material in slice {
                let material = Material::borrow_from(material.as_mut().unwrap());
                materials.push(material);
            }

            let scene_name = ai_scene.mName.into();
            let scene = Scene {
                c: ai_scene,
                marker: PhantomData,
                meshes,
                root_node,
                name: scene_name,
                all_nodes,
                materials,
            };

            Ok(scene)
        }
    }
}

impl<'a> Drop for Scene<'a> {
    fn drop(&mut self) {
        unsafe {
            aiReleaseImport(self.c);
        }
    }
}

#[cfg(test)]
mod test {
    use super::{PostProcessSteps, Scene};
    use std::iter::zip;

    #[test]
    fn test_case() {
        let resource_path =
            rs_core_minimal::file_manager::get_engine_resource("Remote/MonkeyAnim.fbx");
        let scene = Scene::from_file(
            resource_path,
            PostProcessSteps::PopulateArmatureData | PostProcessSteps::Triangulate,
        )
        .expect("Exists");
        println!("Nodes:");
        for (k, v) in scene.all_nodes.iter() {
            let v = v.borrow();
            let parent_name = match &v.parent {
                Some(parent) => match parent.upgrade() {
                    Some(parent) => Some(parent.borrow().name.clone()),
                    None => None,
                },
                None => None,
            };
            println!("    {k}\n        parent: {parent_name:?}");
            println!("        metadata:");
            if let Some(metadata) = &v.metadata {
                for (key, value) in zip(&metadata.keys, &metadata.values) {
                    println!("            {} - {:?}", key, value);
                }
            } else {
                println!("            None");
            }
        }
        println!("Meshes:");
        for mesh in &scene.meshes {
            println!(
                "    {}\n        vertices num: {}, normals num: {}, bitangents num: {}, tangents num: {},
                uv maps num: {}, vertex color maps num: {}, bones num: {}, faces num: {}",
                mesh.name,
                mesh.vertices.len(),
                mesh.normals.len(),
                mesh.bitangents.len(),
                mesh.tangents.len(),
                mesh.texture_coords.len(),
                mesh.colors.len(),
                mesh.bones.len(),
                mesh.faces.len(),
            );
            for bone in &mesh.bones {
                if let Some(armature) = bone.armature.as_ref() {
                    println!("        armature: {}", armature.borrow().name.clone());
                }
            }
        }
        println!("Materials:");
        for material in &scene.materials {
            println!(
                "    num_allocated: {}, material_properties num: {}",
                material.num_allocated,
                material.material_properties.len()
            );
            for property in &material.material_properties {
                println!("        {}, {:?}", property.key, property.value);
            }
        }
    }
}
