use crate::{
    animation::Animation,
    bone::Bone,
    error::Result,
    get_assimp_error,
    material::Material,
    mesh::Mesh,
    metadata::Metadata,
    node::{self, get_node_path, Node},
    post_process_steps::PostProcessSteps,
    property_store::PropertyStore,
    skeleton::Skeleton,
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
            let path = get_node_path(child);
            let node = Rc::new(RefCell::new(Node::new(child, path.clone())));
            nodes.insert(path, node);
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

fn collect_armatures<'a>(
    bones: HashMap<String, Rc<RefCell<Bone<'a>>>>,
) -> HashMap<String, Rc<RefCell<Node<'a>>>> {
    let mut nodes = HashMap::new();
    for (_, bone) in bones {
        if let Some(armature) = &bone.borrow().armature {
            nodes.insert(armature.borrow().name.clone(), armature.clone());
        }
    }
    nodes
}

pub struct Scene<'a> {
    c: *const russimp_sys::aiScene,
    pub name: String,
    pub meshes: Vec<Mesh<'a>>,
    pub root_node: Option<Rc<RefCell<Node<'a>>>>,
    pub all_nodes: HashMap<String, Rc<RefCell<Node<'a>>>>,
    pub armatures: HashMap<String, Rc<RefCell<Node<'a>>>>,
    pub materials: Vec<Material<'a>>,
    pub skeletons: Vec<Skeleton<'a>>,
    pub animations: Vec<Animation<'a>>,
    pub metadata: Option<Metadata<'a>>,
    marker: PhantomData<&'a ()>,
}

impl<'a> Scene<'a> {
    unsafe fn new(ai_scene: *const aiScene) -> Result<Scene<'a>> {
        if ai_scene == std::ptr::null() {
            return Err(crate::error::Error::Assimp(get_assimp_error()));
        }
        let ai_scene = ai_scene.as_ref().unwrap();

        let mut all_nodes = collect_all_nodes(ai_scene);
        build_node_tree(&mut all_nodes);

        let mut skeletons: Vec<Skeleton<'a>> = vec![];
        if ai_scene.mSkeletons.is_null() == false {
            let slice = std::ptr::slice_from_raw_parts(
                ai_scene.mSkeletons,
                ai_scene.mNumSkeletons as usize,
            )
            .as_ref()
            .unwrap();

            for item in slice {
                let skeleton = Skeleton::borrow_from(item.as_mut().unwrap(), &all_nodes);
                skeletons.push(skeleton);
            }
        }

        let root_node = match ai_scene.mRootNode.as_mut() {
            Some(m_root_node) => {
                let path = node::get_node_path(m_root_node);
                all_nodes.get(&path).cloned()
            }
            None => None,
        };

        let mut meshes = Vec::new();
        if !ai_scene.mMeshes.is_null() {
            let slice =
                std::ptr::slice_from_raw_parts(ai_scene.mMeshes, ai_scene.mNumMeshes as usize)
                    .as_ref()
                    .unwrap();
            for mesh in slice {
                let mesh = Mesh::borrow_from(mesh.as_mut().unwrap(), &mut all_nodes);
                meshes.push(mesh);
            }
        }

        let mut materials: Vec<Material<'a>> = vec![];
        if !ai_scene.mMaterials.is_null() {
            let slice = std::ptr::slice_from_raw_parts(
                ai_scene.mMaterials,
                ai_scene.mNumMaterials as usize,
            )
            .as_ref()
            .unwrap();
            for material in slice {
                let material = Material::borrow_from(material.as_mut().unwrap());
                materials.push(material);
            }
        }

        let mut bones = HashMap::new();
        for mesh in &meshes {
            for bone in &mesh.bones {
                let node = bone.borrow().node.clone().unwrap();
                let node = node.borrow();
                bones.insert(node.path.clone(), bone.clone());
            }
        }
        let metadata = match unsafe { ai_scene.mMetaData.as_mut() } {
            Some(m_meta_data) => Some(Metadata::borrow_from(m_meta_data)),
            None => None,
        };

        let mut animations = vec![];

        if !ai_scene.mAnimations.is_null() {
            let slice = std::ptr::slice_from_raw_parts_mut(
                ai_scene.mAnimations,
                ai_scene.mNumAnimations as _,
            )
            .as_mut()
            .unwrap();
            for item in slice {
                let animation = Animation::borrow_from(item.as_mut().unwrap(), &mut all_nodes);
                animations.push(animation);
            }
        }

        let scene_name = ai_scene.mName.into();
        let scene = Scene {
            c: ai_scene,
            name: scene_name,
            meshes,
            root_node,
            all_nodes,
            armatures: collect_armatures(bones),
            materials,
            skeletons,
            animations,
            metadata,
            marker: PhantomData,
        };

        Ok(scene)
    }

    pub fn from_file<P: AsRef<Path>>(path: P, flags: PostProcessSteps) -> Result<Scene<'a>> {
        let path = path.as_ref();
        let path = path.as_os_str().as_encoded_bytes();
        let path = CString::new(path).map_err(|err| crate::error::Error::Nul(err))?;
        unsafe {
            let ai_scene = russimp_sys::aiImportFile(path.as_ptr(), flags.bits());
            Self::new(ai_scene)
        }
    }

    pub fn from_file_with_properties<P: AsRef<Path>>(
        path: P,
        flags: PostProcessSteps,
        props: PropertyStore,
    ) -> Result<Scene<'a>> {
        let path = path.as_ref();
        let path = path.as_os_str().as_encoded_bytes();
        let path = CString::new(path).map_err(|err| crate::error::Error::Nul(err))?;
        unsafe {
            let ai_scene = russimp_sys::aiImportFileExWithProperties(
                path.as_ptr(),
                flags.bits(),
                std::ptr::null_mut(),
                props.get(),
            );
            Self::new(ai_scene)
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
    use crate::{config, property_store::PropertyStore};
    use std::{cell::RefCell, iter::zip, rc::Rc};

    #[test]
    fn test_case() {
        let resource_path =
            rs_core_minimal::file_manager::get_engine_resource("Remote/MonkeyAnim.fbx");
        let scene = Scene::from_file(
            resource_path,
            PostProcessSteps::PopulateArmatureData | PostProcessSteps::Triangulate,
        )
        .expect("Exists");
        dump(scene);
    }

    #[test]
    fn test_case_1() {
        let resource_path =
            rs_core_minimal::file_manager::get_engine_resource("Remote/MonkeyAnim.fbx");
        let mut props = PropertyStore::new();
        props.set_property_bool(&config::AI_CONFIG_FBX_USE_SKELETON_BONE_CONTAINER, true);
        let scene = Scene::from_file_with_properties(
            resource_path,
            PostProcessSteps::PopulateArmatureData | PostProcessSteps::Triangulate,
            props,
        )
        .expect("Exists");
        dump(scene);
    }

    fn walk_armature<'a>(node: Rc<RefCell<crate::node::Node<'a>>>, offset: i32, level: i32) {
        let spaces = offset + level * 2;
        let mut s = String::from("");
        for _ in 0..spaces {
            s += " ";
        }
        println!("{}{}", s, node.borrow().name.clone());
        for child in node.borrow().children.clone() {
            walk_armature(child, offset, level + 1);
        }
    }

    fn dump(scene: Scene<'_>) {
        println!("Scene Metadata:");
        if let Some(metadata) = &scene.metadata {
            for (key, value) in zip(&metadata.keys, &metadata.values) {
                println!("    {} - {:?}", key, value);
            }
        } else {
            println!("    None");
        }

        println!("Root Node:");
        println!("    {}", scene.root_node.as_ref().unwrap().borrow().name);

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
            let node_name = v.name.clone();
            println!("    {node_name}({k})\n        parent: {parent_name:?}");
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
                uv maps num: {}, vertex color maps num: {}, bones num: {}, faces num: {}, bones num : {}",
                mesh.name,
                mesh.vertices.len(),
                mesh.normals.len(),
                mesh.bitangents.len(),
                mesh.tangents.len(),
                mesh.texture_coords.len(),
                mesh.colors.len(),
                mesh.bones.len(),
                mesh.faces.len(),
                mesh.bones.len(),
            );
            for bone in &mesh.bones {
                println!("        bone: {}", bone.borrow().name);
                println!("            weights num: {}", bone.borrow().weights.len());
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

        println!("Skeletons:");
        for skeleton in &scene.skeletons {
            println!("    {}", skeleton.name);
            for bone in &skeleton.bones {
                if let Some(node) = &bone.node {
                    println!("        {}", node.borrow().name);
                }
                if let Some(armature) = &bone.armature {
                    println!("        {}", armature.borrow().name);
                }
            }
        }

        println!("Armatures:");
        for armature in scene.armatures.values() {
            println!("    name: {}", armature.borrow().name);
            walk_armature(armature.clone(), 6, 0);
        }

        println!("Animations:");
        for animation in &scene.animations {
            println!("    name: {}", animation.name);
            println!("    channels: {}", animation.channels.len());
            for channel in animation.channels.iter() {
                println!(
                    "        name: {}",
                    channel.node.as_ref().unwrap().borrow().name
                );
                println!("        pre_state: {:?}", channel.pre_state);
                println!("        post_state: {:?}", channel.post_state);
                println!("        position_keys: {}", channel.position_keys.len());
                println!("        scaling_keys: {}", channel.scaling_keys.len());
                println!("        rotation_keys: {}", channel.rotation_keys.len());
                println!("");
            }
            println!("    mesh_channels: {}", animation.mesh_channels.len());
            println!(
                "    morph_mesh_channels: {}",
                animation.morph_mesh_channels.len()
            );
        }
    }
}
