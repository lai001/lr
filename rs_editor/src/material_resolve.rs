use crate::ui::material_view::{EMaterialNodeType, MaterialNode};
use egui_snarl::{InPinId, NodeId, OutPinId, Snarl};
use rs_artifact::material::{
    GroupBinding, MaterialInfo, MaterialParamentersCollectionBinding, TextureBinding,
};
use rs_engine::{
    content::material_paramenters_collection::MaterialParamentersCollection,
    url_extension::UrlExtension,
};
use rs_foundation::new::SingleThreadMutType;
use rs_render::constants::{MAX_POINT_LIGHTS_NUM, MAX_SPOT_LIGHTS_NUM};
use rs_render_types::MaterialOptions;
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    rc::Rc,
};

struct ResolveResultInternal {
    lines: Vec<String>,
}

#[derive(Clone)]
pub struct ResolveResult {
    pub shader_code: String,
    pub material_info: MaterialInfo,
}

#[derive(Debug, Default)]
struct NodeIOInfo {
    inputs: HashMap<usize, OutPinId>,
    outputs: HashMap<usize, HashSet<InPinId>>,
}

// #[derive(Debug)]
struct ResolveContext<'a> {
    snarl: &'a Snarl<MaterialNode>,
    nodes: HashMap<NodeId, NodeIOInfo>,
    current_group: usize,
    current_binding: usize,
    used_material_paramenters_collections: Vec<SingleThreadMutType<MaterialParamentersCollection>>,
}

impl<'a> ResolveContext<'a> {
    fn from_snarl(snarl: &Snarl<MaterialNode>) -> ResolveContext {
        let mut node_io_infos: HashMap<NodeId, NodeIOInfo> = HashMap::new();
        for (out_pin_id, in_pin_id) in snarl.wires() {
            node_io_infos
                .entry(out_pin_id.node)
                .or_insert_with(|| Default::default())
                .outputs
                .entry(out_pin_id.output)
                .or_insert_with(|| Default::default())
                .insert(in_pin_id);

            *node_io_infos
                .entry(in_pin_id.node)
                .or_insert_with(|| Default::default())
                .inputs
                .entry(in_pin_id.input)
                .or_insert_with(|| OutPinId {
                    node: NodeId(0),
                    output: 0,
                }) = out_pin_id;
        }
        for node_id in snarl.node_ids().map(|x| x.0) {
            if !node_io_infos.contains_key(&node_id) {
                node_io_infos.insert(node_id, Default::default());
            }
        }
        ResolveContext {
            nodes: node_io_infos,
            snarl,
            current_group: 0,
            current_binding: 0,
            used_material_paramenters_collections: vec![],
        }
    }

    fn next_binding(&mut self) -> usize {
        let binding = self.current_binding;
        self.current_binding += 1;
        return binding;
    }

    fn compose_definitions(
        &mut self,
        definitions: &mut Vec<String>,
        options: &MaterialOptions,
        material_info: &mut MaterialInfo,
        is_support_cluster_light: bool,
        // group: usize,
        // binding_start: &mut usize,
    ) {
        // https://www.reddit.com/r/vulkan/comments/abjk81/comment/ed0ut27/?utm_source=share&utm_medium=web3x&utm_name=web3xcss&utm_term=1&utm_content=share_button
        // Note: The maximum binding number specified should be as compact as possible to avoid wasted memory.
        macro_rules! group_binding {
            ($member:ident, $name:literal) => {
                material_info.$member =
                    Some(GroupBinding::new(self.current_group, self.current_binding));
                definitions.append(&mut vec![
                    format!("{}_GROUP={}", $name, self.current_group),
                    format!("{}_BINDING={}", $name, self.current_binding),
                ]);
                self.current_binding += 1;
            };
        }
        group_binding!(global_constants_binding, "GLOBAL_CONSTANTS");
        group_binding!(base_color_sampler_binding, "BASE_COLOR_SAMPLER");
        group_binding!(physical_texture_binding, "PHYSICAL_TEXTURE");
        group_binding!(page_table_texture_binding, "PAGE_TABLE_TEXTURE");
        group_binding!(brdflut_texture_binding, "BRDFLUT_TEXTURE");
        group_binding!(
            pre_filter_cube_map_texture_binding,
            "PRE_FILTER_CUBE_MAP_TEXTURE"
        );
        group_binding!(irradiance_texture_binding, "IRRADIANCE_TEXTURE");
        group_binding!(shadow_map_binding, "SHADOW_MAP");
        group_binding!(constants_binding, "CONSTANTS");
        group_binding!(point_lights_binding, "POINT_LIGHTS");
        group_binding!(spot_lights_binding, "SPOT_LIGHTS");
        group_binding!(
            virtual_texture_constants_binding,
            "VIRTUAL_TEXTURE_CONSTANTS"
        );
        if is_support_cluster_light {
            group_binding!(cluster_light_binding, "CLUSTER_LIGHT");
            group_binding!(cluster_light_index_binding, "CLUSTER_LIGHT_INDEX");
        }
        if options.is_skin {
            group_binding!(skin_constants_binding, "SKIN_CONSTANTS");
        }
    }

    fn resolve(&mut self, options: &MaterialOptions) -> anyhow::Result<ResolveResult> {
        let mut material_info = MaterialInfo {
            map_textures: HashSet::new(),
            virtual_textures: HashSet::new(),
            global_constants_binding: None,
            base_color_sampler_binding: None,
            physical_texture_binding: None,
            page_table_texture_binding: None,
            brdflut_texture_binding: None,
            pre_filter_cube_map_texture_binding: None,
            irradiance_texture_binding: None,
            shadow_map_binding: None,
            constants_binding: None,
            point_lights_binding: None,
            spot_lights_binding: None,
            skin_constants_binding: None,
            virtual_texture_constants_binding: None,
            cluster_light_binding: None,
            cluster_light_index_binding: None,
            material_paramenters_collection_bindings: HashSet::new(),
        };
        let is_support_cluster_light = true;
        let mut definitions: Vec<String> = vec![
            "VIRTUAL_TEXTURE=1".to_string(),
            "MATERIAL_SHADER_CODE=@MATERIAL_SHADER_CODE@".to_string(),
            "USER_TEXTURES=@USER_TEXTURES@".to_string(),
            "MATERIAL_PARAMENTERS_COLLECTION_UNIFORMS=@MATERIAL_PARAMENTERS_COLLECTION_UNIFORMS@"
                .to_string(),
            format!("MAX_POINT_LIGHTS_NUM={}", MAX_POINT_LIGHTS_NUM),
            format!("MAX_SPOT_LIGHTS_NUM={}", MAX_SPOT_LIGHTS_NUM),
        ];
        if is_support_cluster_light {
            definitions.push("SUPPORT_CLUSTER_LIGHTS=1".to_string());
        }
        // let current_max_binding =
        self.compose_definitions(
            &mut definitions,
            options,
            &mut material_info,
            is_support_cluster_light,
        );
        // let resolve_context = ResolveContext::from_snarl(snarl);
        let attribute_node_id = egui_snarl::NodeId(0);
        let result = self.resolve_attribute_node(
            attribute_node_id,
            // &self,
            // self.snarl,
            &mut material_info,
            // current_max_binding,
        )?;
        let mut lines: Vec<String> = vec![];
        for resolve_result in result.iter() {
            lines.extend_from_slice(&resolve_result.lines);
        }
        let material_shader_code = lines.join("\n");
        let shader_path = rs_render::get_buildin_shader_dir().join("pbr_shading.wgsl");
        let include_dirs: Vec<PathBuf> = vec![];

        if options.is_skin {
            definitions.push(format!(
                "SKELETON_MAX_BONES={}",
                rs_render::global_shaders::skeleton_shading::NUM_MAX_BONE
            ));
        }
        let shader_code = rs_shader_compiler_core::pre_process::pre_process(
            &shader_path,
            include_dirs.iter(),
            definitions.iter(),
        )?;

        let shader_code = shader_code.replace("@MATERIAL_SHADER_CODE@", &material_shader_code);

        let mut texture_uniform_code = "".to_string();
        for map_texture in material_info.map_textures.clone() {
            let name = map_texture.get_texture_bind_name();
            let line = format!(
                "@group({}) @binding({}) var {}: texture_2d<f32>;\n",
                map_texture.group, map_texture.binding, name
            );
            texture_uniform_code += &line;
        }
        let shader_code = shader_code.replace("@USER_TEXTURES@", &texture_uniform_code);

        let mut material_paramenters_collection_uniform_code = "".to_string();
        for material_paramenters_collection in self.used_material_paramenters_collections.iter() {
            let uniform_struct_string = material_paramenters_collection_to_struct_string(
                &material_paramenters_collection.borrow(),
            );
            material_paramenters_collection_uniform_code.push_str(&uniform_struct_string);
            material_paramenters_collection_uniform_code.push_str("\n");
        }
        for material_paramenters_collection in self.used_material_paramenters_collections.iter() {
            let material_paramenters_collection = material_paramenters_collection.borrow();
            let url = material_paramenters_collection.url.clone();
            let type_text = material_paramenters_collection.url.get_name_in_editor();
            let line = format!(
                "@group({}) @binding({}) var<uniform> material_paramenters_collection_uniform_{}: {};\n",
                self.current_group, self.current_binding, type_text, type_text
            );
            material_info
                .material_paramenters_collection_bindings
                .insert(MaterialParamentersCollectionBinding {
                    group: self.current_group,
                    binding: self.current_binding,
                    material_paramenters_collection_url: url,
                });
            self.current_binding += 1;
            material_paramenters_collection_uniform_code.push_str(&line);
            material_paramenters_collection_uniform_code.push_str("\n");
        }
        let shader_code = shader_code.replace(
            "@MATERIAL_PARAMENTERS_COLLECTION_UNIFORMS@",
            &material_paramenters_collection_uniform_code,
        );

        Ok(ResolveResult {
            shader_code,
            material_info,
        })
    }

    fn resolve_attribute_node(
        &mut self,
        attribute_node_id: NodeId,
        // resolve_context: &ResolveContext,
        // snarl: &Snarl<MaterialNode>,
        material_info: &mut MaterialInfo,
        // user_texture_binding_start: usize,
    ) -> anyhow::Result<Vec<ResolveResultInternal>> {
        let mut result: Vec<ResolveResultInternal> = Vec::new();

        macro_rules! resolve_attribute {
            ($name:ident, $input:literal, $convert_type:ident) => {{
                let attribute_node = self
                    .snarl
                    .get_node(attribute_node_id)
                    .expect("This node should not be null");

                let EMaterialNodeType::Sink(attribute) = &attribute_node.node_type else {
                    panic!("This node should be a sink node");
                };
                let attribute_value_literal = { attribute.$name.$convert_type().literal() };
                let value = self.resolve_attribute(
                    stringify!($name),
                    $input,
                    attribute_node_id,
                    &attribute_value_literal,
                    // self,
                    // self.snarl,
                    material_info,
                    // user_texture_binding_start,
                )?;
                result.push(value);
            }};
        }

        resolve_attribute!(base_color, 0, convert_to_vec3);
        resolve_attribute!(metallic, 1, convert_to_f32);
        resolve_attribute!(roughness, 2, convert_to_f32);
        resolve_attribute!(normal, 3, convert_to_vec3);
        resolve_attribute!(opacity, 4, convert_to_f32);
        resolve_attribute!(clear_coat, 5, convert_to_f32);
        resolve_attribute!(clear_coat_roughness, 6, convert_to_f32);

        Ok(result)
    }

    fn resolve_attribute(
        &mut self,
        name: &str,
        input: usize,
        attribute_node_id: NodeId,
        attribute_value_literal: &str,
        // resolve_context: &ResolveContext,
        // snarl: &Snarl<MaterialNode>,
        material_info: &mut MaterialInfo,
        // user_texture_binding_start: usize,
    ) -> anyhow::Result<ResolveResultInternal> {
        let node_io_info = self
            .nodes
            .get(&attribute_node_id)
            .expect("This node should not be null");

        let right = {
            node_io_info
                .inputs
                .get(&input)
                .and_then(|x| {
                    let var_name = node_var_name(x.node);
                    Some(var_name)
                })
                .unwrap_or_else(|| attribute_value_literal.to_string())
        };

        // let mut lines: Vec<String> = vec![];

        let mut lines: Vec<String> = vec![];

        if let Some(input) = node_io_info.inputs.get(&input) {
            self.walk_resolve_node(
                input.node,
                // self,
                // self.snarl,
                &mut lines,
                material_info,
                // user_texture_binding_start,
            );
        }
        // let Some(input) = node_io_info.inputs.get(&input) else {
        //     panic!()
        // };
        lines.reverse();
        // Some(lines)

        // let mut lines: Vec<String> = node_io_info
        //     .inputs
        //     .get(&input)
        //     .and_then(|x| {

        //     })
        //     .unwrap_or_else(|| vec![]);

        // let Some(right) = node_io_info.inputs.get(&input) else {
        //     panic!()
        // };

        lines.push(format!("    user_attributes.{} = {};", name, right));
        Ok(ResolveResultInternal { lines })
    }

    fn walk_resolve_node(
        &mut self,
        node_id: NodeId,
        // resolve_context: &ResolveContext,
        // snarl: &Snarl<MaterialNode>,
        lines: &mut Vec<String>,
        material_info: &mut MaterialInfo,
        // user_texture_binding_start: usize,
    ) {
        let node = self.snarl.get_node(node_id).expect("Not null");
        let line = self.resolve_node(
            node_id,
            node,
            // resolve_context,
            material_info,
            // snarl,
            // user_texture_binding_start,
        );
        lines.push(line);
        let out_pin_ids: Vec<_> = {
            let node_io_info = self.nodes.get(&node_id).unwrap();
            node_io_info.inputs.iter().map(|x| x.1.clone()).collect()
        };
        for out_pin_id in out_pin_ids {
            self.walk_resolve_node(
                out_pin_id.node,
                // resolve_context,
                // snarl,
                lines,
                material_info,
                // user_texture_binding_start,
            );
        }

        // for (_, out_pin_id) in &node_io_info.inputs {
        //     self.walk_resolve_node(
        //         out_pin_id.node,
        //         // resolve_context,
        //         // snarl,
        //         lines,
        //         material_info,
        //         // user_texture_binding_start,
        //     );
        // }
    }

    fn resolve_node(
        &mut self,
        node_id: NodeId,
        node: &MaterialNode,
        // resolve_context: &ResolveContext,
        material_info: &mut MaterialInfo,
        // snarl: &Snarl<MaterialNode>,
        // user_texture_binding_start: usize,
    ) -> String {
        // let _ = snarl;
        let var_name = node_var_name(node_id);
        match &node.node_type {
            EMaterialNodeType::Add(v1, v2) => {
                let inputs = &self
                    .nodes
                    .get(&node_id)
                    .expect("This node should not be null")
                    .inputs;
                let part_1 = inputs
                    .get(&0)
                    .and_then(|out_pin_id| Some(node_var_name(out_pin_id.node)))
                    .unwrap_or_else(|| v1.literal());
                let part_2 = inputs
                    .get(&1)
                    .and_then(|out_pin_id| Some(node_var_name(out_pin_id.node)))
                    .unwrap_or_else(|| v2.literal());
                format!("var {} = {} + {};", var_name, part_1, part_2)
            }
            EMaterialNodeType::Texture(texture_url) => {
                if let Some(texture_url) = texture_url {
                    let binding = self.next_binding();
                    let inputs = &self
                        .nodes
                        .get(&node_id)
                        .expect("This node should not be null")
                        .inputs;
                    let binding = TextureBinding {
                        group: self.current_group,
                        binding, //user_texture_binding_start + material_info.map_textures.len(),
                        texture_url: texture_url.clone(),
                    };
                    let texture_var_name: String;
                    if let Some(exist) = material_info
                        .map_textures
                        .iter()
                        .find(|x| &x.texture_url == texture_url)
                    {
                        texture_var_name = exist.get_texture_bind_name();
                    } else {
                        texture_var_name = binding.get_texture_bind_name();
                        material_info.map_textures.insert(binding);
                    }
                    inputs.get(&0).and_then(|out_pin_id| {
                    Some(format!(
                        "var {} = textureSample({}, base_color_sampler, {}).xyz;",
                        var_name,
                        texture_var_name,
                        node_var_name(out_pin_id.node)
                    ))
                }).unwrap_or_else(|| {
                    format!("var {} = textureSample({}, base_color_sampler, vertex_output.tex_coord0).xyz;", var_name, texture_var_name)
                })
                } else {
                    format!("var {} = vec4<f32>(0.0);", var_name)
                }
            }
            EMaterialNodeType::TexCoord(tex_coord_index) => {
                format!(
                    "var {} = vertex_output.tex_coord{};",
                    var_name, tex_coord_index
                )
            }
            EMaterialNodeType::Sink(_) => unreachable!(),
            EMaterialNodeType::VirtualTexture(texture_url) => {
                if let Some(texture_url) = texture_url {
                    material_info.virtual_textures.insert(texture_url.clone());
                    format!(
                    "var {} = virtual_texture_sample(vertex_output.tex_coord0, virtual_texture_constants.virtual_texture_max_lod, virtual_texture_constants.virtual_texture_size).xyz;",
                    var_name,
                )
                } else {
                    format!("var {} = vec3<f32>(0.0);", var_name)
                }
            }
            EMaterialNodeType::Time => todo!(),
            EMaterialNodeType::Sin(v1) => {
                let inputs = &self
                    .nodes
                    .get(&node_id)
                    .expect("This node should not be null")
                    .inputs;
                let part_1 = inputs
                    .get(&0)
                    .and_then(|out_pin_id| Some(node_var_name(out_pin_id.node)))
                    .unwrap_or_else(|| v1.literal());
                format!("var {} = {}", var_name, part_1)
            }
            EMaterialNodeType::MaterialParamentersCollection((
                material_paramenters_collection,
                name,
            )) => {
                if let (Some(material_paramenters_collection), Some(name)) =
                    (material_paramenters_collection.clone(), name)
                {
                    let is_contain = self
                        .used_material_paramenters_collections
                        .iter()
                        .find(|x| Rc::ptr_eq(&x, &material_paramenters_collection))
                        .is_some();
                    if !is_contain {
                        self.used_material_paramenters_collections
                            .push(material_paramenters_collection.clone());
                    }
                    let type_text = material_paramenters_collection
                        .borrow()
                        .url
                        .get_name_in_editor();
                    format!(
                        "var {} = material_paramenters_collection_uniform_{}.{};",
                        var_name, type_text, name
                    )
                } else {
                    "".to_string()
                }
            }
        }
    }
}

pub fn resolve(
    snarl: &Snarl<MaterialNode>,
    options: Vec<MaterialOptions>,
) -> anyhow::Result<HashMap<MaterialOptions, ResolveResult>> {
    let mut results: HashMap<MaterialOptions, ResolveResult> = HashMap::new();
    for option in options {
        let mut resolve_context = ResolveContext::from_snarl(snarl);
        let result = resolve_context.resolve(&option)?;
        // let result = resolve_internal(snarl, &option)?;
        results.insert(option, result);
    }
    Ok(results)
}

fn node_var_name(node_id: NodeId) -> String {
    format!("v{}", node_id.0)
}

fn material_paramenters_collection_to_struct_string(
    material_paramenters_collection: &MaterialParamentersCollection,
) -> String {
    let mut fields = "".to_string();
    let len = material_paramenters_collection.fields.len();
    for (index, field) in material_paramenters_collection.fields.iter().enumerate() {
        match field.data_type {
            rs_engine::uniform_map::BaseDataValueType::F32(_) => {
                let text = format!("    {}: f32,", &field.name);
                fields.push_str(&text);
                if index != len - 1 {
                    fields.push_str("\n");
                }
            }
            rs_engine::uniform_map::BaseDataValueType::Vec2(_) => {
                let text = format!("    {}: vec{}<f32>,", &field.name, 2);
                fields.push_str(&text);
                if index != len - 1 {
                    fields.push_str("\n");
                }
            }
            rs_engine::uniform_map::BaseDataValueType::Vec3(_) => {
                let text = format!("    {}: vec{}<f32>,", &field.name, 3);
                fields.push_str(&text);
                if index != len - 1 {
                    fields.push_str("\n");
                }
            }
            rs_engine::uniform_map::BaseDataValueType::Vec4(_) => {
                let text = format!("    {}: vec{}<f32>,", &field.name, 4);
                fields.push_str(&text);
                if index != len - 1 {
                    fields.push_str("\n");
                }
            }
        }
    }

    return format!(
        r"
struct {} {{
{}
}}",
        material_paramenters_collection.url.get_name_in_editor(),
        fields
    );
}
