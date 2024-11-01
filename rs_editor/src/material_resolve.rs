use crate::ui::material_view::{EMaterialNodeType, MaterialNode};
use egui_snarl::{InPinId, NodeId, OutPinId, Snarl};
use rs_artifact::material::{GroupBinding, MaterialInfo, TextureBinding};
use rs_render::constants::{MAX_POINT_LIGHTS_NUM, MAX_SPOT_LIGHTS_NUM};
use rs_render_types::MaterialOptions;
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
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

#[derive(Debug)]
struct ResolveContext {
    nodes: HashMap<NodeId, NodeIOInfo>,
}

impl ResolveContext {
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
        }
    }
}

pub fn resolve(
    snarl: &Snarl<MaterialNode>,
    options: Vec<MaterialOptions>,
) -> anyhow::Result<HashMap<MaterialOptions, ResolveResult>> {
    let mut results: HashMap<MaterialOptions, ResolveResult> = HashMap::new();
    for option in options {
        let result = resolve_internal(snarl, &option)?;
        results.insert(option, result);
    }
    Ok(results)
}

fn compose_definitions(
    definitions: &mut Vec<String>,
    options: &MaterialOptions,
    material_info: &mut MaterialInfo,
) -> usize {
    // https://www.reddit.com/r/vulkan/comments/abjk81/comment/ed0ut27/?utm_source=share&utm_medium=web3x&utm_name=web3xcss&utm_term=1&utm_content=share_button
    // Note: The maximum binding number specified should be as compact as possible to avoid wasted memory.
    let group: usize = 0;
    let mut binding: usize = 0;
    macro_rules! group_binding {
        ($name:literal, $g:ty, $b:ty) => {
            definitions.append(&mut vec![
                format!("{}_GROUP={}", $name, group),
                format!("{}_BINDING={}", $name, binding),
            ]);
            binding += 1;
        };
    }
    material_info.global_constants_binding = Some(GroupBinding::new(group, binding));
    group_binding!("GLOBAL_CONSTANTS", group, binding);
    material_info.base_color_sampler_binding = Some(GroupBinding::new(group, binding));
    group_binding!("BASE_COLOR_SAMPLER", group, binding);
    material_info.physical_texture_binding = Some(GroupBinding::new(group, binding));
    group_binding!("PHYSICAL_TEXTURE", group, binding);
    material_info.page_table_texture_binding = Some(GroupBinding::new(group, binding));
    group_binding!("PAGE_TABLE_TEXTURE", group, binding);
    material_info.brdflut_texture_binding = Some(GroupBinding::new(group, binding));
    group_binding!("BRDFLUT_TEXTURE", group, binding);
    material_info.pre_filter_cube_map_texture_binding = Some(GroupBinding::new(group, binding));
    group_binding!("PRE_FILTER_CUBE_MAP_TEXTURE", group, binding);
    material_info.irradiance_texture_binding = Some(GroupBinding::new(group, binding));
    group_binding!("IRRADIANCE_TEXTURE", group, binding);
    material_info.shadow_map_binding = Some(GroupBinding::new(group, binding));
    group_binding!("SHADOW_MAP", group, binding);
    material_info.constants_binding = Some(GroupBinding::new(group, binding));
    group_binding!("CONSTANTS", group, binding);
    material_info.point_lights_binding = Some(GroupBinding::new(group, binding));
    group_binding!("POINT_LIGHTS", group, binding);
    material_info.spot_lights_binding = Some(GroupBinding::new(group, binding));
    group_binding!("SPOT_LIGHTS", group, binding);
    material_info.virtual_texture_constants_binding = Some(GroupBinding::new(group, binding));
    group_binding!("VIRTUAL_TEXTURE_CONSTANTS", group, binding);
    if options.is_skin {
        material_info.skin_constants_binding = Some(GroupBinding::new(group, binding));
        group_binding!("SKIN_CONSTANTS", group, binding);
    }
    return binding;
}

fn resolve_internal(
    snarl: &Snarl<MaterialNode>,
    options: &MaterialOptions,
) -> anyhow::Result<ResolveResult> {
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
        skin_constants_binding: None,
        virtual_texture_constants_binding: None,
        spot_lights_binding: None,
    };
    let mut definitions: Vec<String> = vec![
        "VIRTUAL_TEXTURE=1".to_string(),
        "MATERIAL_SHADER_CODE=@MATERIAL_SHADER_CODE@".to_string(),
        "USER_TEXTURES=@USER_TEXTURES@".to_string(),
        format!("MAX_POINT_LIGHTS_NUM={}", MAX_POINT_LIGHTS_NUM),
        format!("MAX_SPOT_LIGHTS_NUM={}", MAX_SPOT_LIGHTS_NUM),
    ];
    let current_max_binding = compose_definitions(&mut definitions, options, &mut material_info);
    let resolve_context = ResolveContext::from_snarl(snarl);
    let attribute_node_id = egui_snarl::NodeId(0);
    let result = resolve_attribute_node(
        attribute_node_id,
        &resolve_context,
        snarl,
        &mut material_info,
        current_max_binding,
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
    let shader_code = rs_shader_compiler::pre_process::pre_process(
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

    Ok(ResolveResult {
        shader_code,
        material_info,
    })
}

fn node_var_name(node_id: NodeId) -> String {
    format!("v{}", node_id.0)
}

fn resolve_attribute_node(
    attribute_node_id: NodeId,
    resolve_context: &ResolveContext,
    snarl: &Snarl<MaterialNode>,
    material_info: &mut MaterialInfo,
    user_texture_binding_start: usize,
) -> anyhow::Result<Vec<ResolveResultInternal>> {
    let mut result: Vec<ResolveResultInternal> = Vec::new();

    macro_rules! resolve_attribute {
        ($name:ident, $input:literal, $convert_type:ident) => {{
            let attribute_node = snarl
                .get_node(attribute_node_id)
                .expect("This node should not be null");

            let EMaterialNodeType::Sink(attribute) = &attribute_node.node_type else {
                panic!("This node should be a sink node");
            };
            let attribute_value_literal = { attribute.$name.$convert_type().literal() };
            let value = resolve_attribute(
                stringify!($name),
                $input,
                attribute_node_id,
                &attribute_value_literal,
                resolve_context,
                snarl,
                material_info,
                user_texture_binding_start,
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
    name: &str,
    input: usize,
    attribute_node_id: NodeId,
    attribute_value_literal: &str,
    resolve_context: &ResolveContext,
    snarl: &Snarl<MaterialNode>,
    material_info: &mut MaterialInfo,
    user_texture_binding_start: usize,
) -> anyhow::Result<ResolveResultInternal> {
    let node_io_info = resolve_context
        .nodes
        .get(&attribute_node_id)
        .expect("This node should not be null");
    let mut lines = node_io_info
        .inputs
        .get(&input)
        .and_then(|x| {
            let mut lines: Vec<String> = vec![];
            walk_resolve_node(
                x.node,
                resolve_context,
                snarl,
                &mut lines,
                material_info,
                user_texture_binding_start,
            );
            lines.reverse();
            Some(lines)
        })
        .unwrap_or_else(|| vec![]);

    let right = node_io_info
        .inputs
        .get(&input)
        .and_then(|x| {
            let var_name = node_var_name(x.node);
            Some(var_name)
        })
        .unwrap_or_else(|| attribute_value_literal.to_string());

    lines.push(format!("user_attributes.{} = {};", name, right));
    Ok(ResolveResultInternal { lines })
}

fn walk_resolve_node(
    node_id: NodeId,
    resolve_context: &ResolveContext,
    snarl: &Snarl<MaterialNode>,
    lines: &mut Vec<String>,
    material_info: &mut MaterialInfo,
    user_texture_binding_start: usize,
) {
    let node = snarl.get_node(node_id).expect("Not null");
    let line = resolve_node(
        node_id,
        node,
        resolve_context,
        material_info,
        snarl,
        user_texture_binding_start,
    );
    lines.push(line);
    let node_io_info = resolve_context.nodes.get(&node_id).unwrap();
    for (_, out_pin_id) in &node_io_info.inputs {
        walk_resolve_node(
            out_pin_id.node,
            resolve_context,
            snarl,
            lines,
            material_info,
            user_texture_binding_start,
        );
    }
}

fn resolve_node(
    node_id: NodeId,
    node: &MaterialNode,
    resolve_context: &ResolveContext,
    material_info: &mut MaterialInfo,
    snarl: &Snarl<MaterialNode>,
    user_texture_binding_start: usize,
) -> String {
    let _ = snarl;
    let var_name = node_var_name(node_id);
    match &node.node_type {
        EMaterialNodeType::Add(v1, v2) => {
            let inputs = &resolve_context
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
                let inputs = &resolve_context
                    .nodes
                    .get(&node_id)
                    .expect("This node should not be null")
                    .inputs;
                let binding = TextureBinding {
                    group: 0,
                    binding: user_texture_binding_start + material_info.map_textures.len(),
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
            let inputs = &resolve_context
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
    }
}
