use crate::ui::material_view::{EMaterialNodeType, MaterialNode};
use egui_snarl::{InPinId, NodeId, OutPinId, Snarl};
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

struct ResolveResult {
    lines: Vec<String>,
    map_texture_names: HashMap<url::Url, String>,
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

pub fn resolve(snarl: &Snarl<MaterialNode>) -> anyhow::Result<String> {
    let resolve_context = ResolveContext::from_snarl(snarl);
    let attribute_node_id = egui_snarl::NodeId(0);
    let result = resolve_attribute_node(attribute_node_id, &resolve_context, snarl)?;
    let mut lines: Vec<String> = vec![];
    for resolve_result in result.iter() {
        lines.extend_from_slice(&resolve_result.lines);
    }
    let material_shader_code = lines.join("\n");
    let shader_path = rs_render::get_buildin_shader_dir().join("pbr_shading.wgsl");
    let include_dirs: Vec<PathBuf> = vec![];
    let definitions: Vec<String> = vec![
        "MATERIAL_SHADER_CODE=@MATERIAL_SHADER_CODE@".to_string(),
        format!(
            "SKELETON_MAX_BONES={}",
            rs_render::global_shaders::skeleton_shading::NUM_MAX_BONE
        ),
    ];
    let shader_code = rs_shader_compiler::pre_process::pre_process2(
        &shader_path,
        include_dirs.iter(),
        definitions.iter(),
    )?;
    let shader_code = shader_code.replace("@MATERIAL_SHADER_CODE@", &material_shader_code);
    Ok(shader_code)
}

fn node_var_name(node_id: NodeId) -> String {
    format!("v{}", node_id.0)
}

fn resolve_attribute_node(
    attribute_node_id: NodeId,
    resolve_context: &ResolveContext,
    snarl: &Snarl<MaterialNode>,
) -> anyhow::Result<Vec<ResolveResult>> {
    let mut result: Vec<ResolveResult> = Vec::new();

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
) -> anyhow::Result<ResolveResult> {
    let node_io_info = resolve_context
        .nodes
        .get(&attribute_node_id)
        .expect("This node should not be null");

    let mut lines = node_io_info
        .inputs
        .get(&input)
        .and_then(|x| {
            let mut lines: Vec<String> = vec![];
            walk_resolve_node(x.node, resolve_context, snarl, &mut lines);
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
    Ok(ResolveResult {
        lines,
        map_texture_names: HashMap::new(),
    })
}

fn walk_resolve_node(
    node_id: NodeId,
    resolve_context: &ResolveContext,
    snarl: &Snarl<MaterialNode>,
    lines: &mut Vec<String>,
) {
    let node = snarl.get_node(node_id).expect("Not null");
    let line = resolve_node(node_id, node, resolve_context);
    lines.push(line);
    let node_io_info = resolve_context.nodes.get(&node_id).unwrap();
    for (_, out_pin_id) in &node_io_info.inputs {
        walk_resolve_node(out_pin_id.node, resolve_context, snarl, lines);
    }
}

fn resolve_node(node_id: NodeId, node: &MaterialNode, resolve_context: &ResolveContext) -> String {
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
            format!("var {} = {} + {}", var_name, part_1, part_2)
        }
        EMaterialNodeType::Texture(texture_url) => {
            if let Some(_) = texture_url {
                let inputs = &resolve_context
                    .nodes
                    .get(&node_id)
                    .expect("This node should not be null")
                    .inputs;

                inputs.get(&0).and_then(|out_pin_id| {
                    Some(format!(
                        "var {} = textureSample(texture, base_color_sampler, {});",
                        var_name,
                        node_var_name(out_pin_id.node)
                    ))
                }).unwrap_or_else(|| {
                    format!("var {} = textureSample(texture, base_color_sampler, vertex_output.tex_coord0);", var_name)
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
    }
}
