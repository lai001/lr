use crate::{
    content_folder::ContentFolder,
    editor_ui,
    material_resolve::{self, ResolveResult},
};
use egui::*;
use egui_snarl::{
    ui::{Grid, PinInfo, SnarlStyle, SnarlViewer},
    InPin, NodeId, OutPin, Snarl,
};
use rs_engine::content::material_paramenters_collection::MaterialParamentersCollection;
use rs_foundation::new::SingleThreadMutType;
use rs_render_types::MaterialOptions;
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

const NODE_IO_COLOR: Color32 = Color32::WHITE;

pub struct GraphViewer {
    pub texture_urls: Vec<url::Url>,
    pub virtual_texture_urls: Vec<url::Url>,
    pub material_parameters_collection_urls: Vec<url::Url>,
    pub is_updated: bool,
    pub folder: SingleThreadMutType<ContentFolder>,
}

impl GraphViewer {
    fn value_type_combo_box(
        &mut self,
        id_source: impl std::hash::Hash,
        value_type: &mut EValueType,
        ui: &mut egui::Ui,
    ) {
        let mut responses: Vec<egui::Response> = vec![];
        egui::ComboBox::from_id_salt(id_source)
            .width(1.0)
            .selected_text(value_type.get_type_name())
            .show_ui(ui, |ui| {
                let response = ui.selectable_value(
                    value_type,
                    EValueType::F32(0.0),
                    EValueType::F32(0.0).get_type_name(),
                );
                responses.push(response);
                let response = ui.selectable_value(
                    value_type,
                    EValueType::Vec2(glam::Vec2::ZERO),
                    EValueType::Vec2(glam::Vec2::ZERO).get_type_name(),
                );
                responses.push(response);
                let response = ui.selectable_value(
                    value_type,
                    EValueType::Vec3(glam::Vec3::ZERO),
                    EValueType::Vec3(glam::Vec3::ZERO).get_type_name(),
                );
                responses.push(response);
                let response = ui.selectable_value(
                    value_type,
                    EValueType::Vec4(glam::Vec4::ZERO),
                    EValueType::Vec4(glam::Vec4::ZERO).get_type_name(),
                );
                responses.push(response);
            });
        match value_type {
            EValueType::F32(value) => {
                let response = ui.add(egui::DragValue::new(value).speed(0.1));
                responses.push(response);
            }
            EValueType::Vec2(value) => {
                let response = ui.add(egui::DragValue::new(&mut value.x).speed(0.1).prefix("x:"));
                responses.push(response);
                let response = ui.add(egui::DragValue::new(&mut value.y).speed(0.1).prefix("y:"));
                responses.push(response);
            }
            EValueType::Vec3(value) => {
                let response = ui.add(egui::DragValue::new(&mut value.x).speed(0.1).prefix("x:"));
                responses.push(response);
                let response = ui.add(egui::DragValue::new(&mut value.y).speed(0.1).prefix("y:"));
                responses.push(response);
                let response = ui.add(egui::DragValue::new(&mut value.z).speed(0.1).prefix("z:"));
                responses.push(response);
            }
            EValueType::Vec4(value) => {
                let response = ui.add(egui::DragValue::new(&mut value.x).speed(0.1).prefix("x:"));
                responses.push(response);
                let response = ui.add(egui::DragValue::new(&mut value.y).speed(0.1).prefix("y:"));
                responses.push(response);
                let response = ui.add(egui::DragValue::new(&mut value.z).speed(0.1).prefix("z:"));
                responses.push(response);
                let response = ui.add(egui::DragValue::new(&mut value.w).speed(0.1).prefix("w:"));
                responses.push(response);
            }
        }
        for response in responses {
            if response.clicked() || response.changed() {
                self.is_updated = true;
                break;
            }
        }
    }
}

impl SnarlViewer<MaterialNode> for GraphViewer {
    fn title(&mut self, node: &MaterialNode) -> String {
        node.node_type.get_name()
    }

    fn outputs(&mut self, node: &MaterialNode) -> usize {
        match node.node_type {
            EMaterialNodeType::Add(..) => 1,
            EMaterialNodeType::Sink(..) => 0,
            EMaterialNodeType::Texture(_) => 1,
            EMaterialNodeType::TexCoord(_) => 1,
            EMaterialNodeType::VirtualTexture(_) => 1,
            EMaterialNodeType::Time => 1,
            EMaterialNodeType::Sin(_) => 1,
            EMaterialNodeType::MaterialParamentersCollection(_) => 1,
        }
    }

    fn inputs(&mut self, node: &MaterialNode) -> usize {
        match node.node_type {
            EMaterialNodeType::Add(..) => 2,
            EMaterialNodeType::Sink(..) => 7,
            EMaterialNodeType::Texture(_) => 2,
            EMaterialNodeType::TexCoord(_) => 0,
            EMaterialNodeType::VirtualTexture(_) => 1,
            EMaterialNodeType::Time => 0,
            EMaterialNodeType::Sin(_) => 1,
            EMaterialNodeType::MaterialParamentersCollection(_) => 2,
        }
    }

    fn show_input(
        &mut self,
        pin: &InPin,
        ui: &mut Ui,
        snarl: &mut Snarl<MaterialNode>,
    ) -> impl egui_snarl::ui::SnarlPin + 'static {
        let node = &mut snarl[pin.id.node];
        match &mut node.node_type {
            EMaterialNodeType::Add(v1, v2) => {
                if !pin.remotes.is_empty() {
                    return PinInfo::square().with_fill(NODE_IO_COLOR);
                }
                match (&v1, &v2) {
                    (EValueType::F32(_), EValueType::F32(_)) => {}
                    (EValueType::F32(_), EValueType::Vec2(_)) => {}
                    (EValueType::F32(_), EValueType::Vec3(_)) => {}
                    (EValueType::F32(_), EValueType::Vec4(_)) => {}
                    (EValueType::Vec2(_), EValueType::F32(_)) => {}
                    (EValueType::Vec2(_), EValueType::Vec2(_)) => {}
                    (EValueType::Vec3(_), EValueType::F32(_)) => {}
                    (EValueType::Vec3(_), EValueType::Vec3(_)) => {}
                    (EValueType::Vec4(_), EValueType::F32(_)) => {}
                    (EValueType::Vec4(_), EValueType::Vec4(_)) => {}
                    _ => panic!(),
                }
                match pin.id.input {
                    0 => {
                        // ui.add(egui::DragValue::new(v1));
                        self.value_type_combo_box("v1", v1, ui);
                    }
                    1 => {
                        self.value_type_combo_box("v2", v2, ui);
                        // ui.add(egui::DragValue::new(v2));
                    }
                    _ => unreachable!(),
                }
                PinInfo::square().with_fill(NODE_IO_COLOR)
            }
            EMaterialNodeType::Sink(attribute) => {
                let names = vec![
                    "Base Color",
                    "Metallic",
                    "Roughness",
                    "Normal",
                    "Opacity",
                    "Clear Coat",
                    "Clear Coat Roughness",
                ];
                ui.label(names[pin.id.input]);
                if !pin.remotes.is_empty() {
                    return PinInfo::square().with_fill(NODE_IO_COLOR);
                }
                match pin.id.input {
                    0 => {
                        self.value_type_combo_box("Base Color", &mut attribute.base_color, ui);
                    }
                    1 => {
                        self.value_type_combo_box("Metallic", &mut attribute.metallic, ui);
                    }
                    2 => {
                        self.value_type_combo_box("Roughness", &mut attribute.roughness, ui);
                    }
                    3 => {
                        self.value_type_combo_box("Normal", &mut attribute.normal, ui);
                    }
                    4 => {
                        self.value_type_combo_box("Opacity", &mut attribute.opacity, ui);
                    }
                    5 => {
                        self.value_type_combo_box("Clear Coat", &mut attribute.clear_coat, ui);
                    }
                    6 => {
                        self.value_type_combo_box(
                            "Clear Coat Roughness",
                            &mut attribute.clear_coat_roughness,
                            ui,
                        );
                    }
                    _ => unreachable!(),
                }
                PinInfo::square().with_fill(NODE_IO_COLOR)
            }
            EMaterialNodeType::Texture(current_value) => match pin.id.input {
                0 => {
                    ui.label("UV");
                    PinInfo::square().with_fill(NODE_IO_COLOR)
                }
                1 => {
                    let text = if let Some(current_value) = current_value.as_ref() {
                        current_value.to_string()
                    } else {
                        "None".to_string()
                    };

                    egui::ComboBox::from_label("")
                        .selected_text(format!("{}", text))
                        .show_ui(ui, |ui| {
                            if ui.selectable_value(current_value, None, "None").clicked() {
                                self.is_updated = true;
                            }
                            for selectable_texture_url in self.texture_urls.iter_mut() {
                                let des = selectable_texture_url.to_string();
                                if ui
                                    .selectable_value(
                                        current_value,
                                        Some(selectable_texture_url.clone()),
                                        des.clone(),
                                    )
                                    .clicked()
                                {
                                    self.is_updated = true;
                                }
                            }
                        });

                    PinInfo::default()
                }
                _ => unreachable!(),
            },
            EMaterialNodeType::TexCoord(_) => PinInfo::default(),
            EMaterialNodeType::VirtualTexture(current_value) => {
                let text = if let Some(current_value) = current_value.as_ref() {
                    current_value.to_string()
                } else {
                    "None".to_string()
                };

                egui::ComboBox::from_label("")
                    .selected_text(format!("{}", text))
                    .show_ui(ui, |ui| {
                        if ui.selectable_value(current_value, None, "None").clicked() {
                            self.is_updated = true;
                        }
                        for selectable_texture_url in self.virtual_texture_urls.iter_mut() {
                            let des = selectable_texture_url.to_string();
                            if ui
                                .selectable_value(
                                    current_value,
                                    Some(selectable_texture_url.clone()),
                                    des.clone(),
                                )
                                .clicked()
                            {
                                self.is_updated = true;
                            }
                        }
                    });

                PinInfo::default()
            }
            EMaterialNodeType::Time => todo!(),
            EMaterialNodeType::Sin(v1) => {
                if !pin.remotes.is_empty() {
                    return PinInfo::square().with_fill(NODE_IO_COLOR);
                }
                match pin.id.input {
                    0 => {
                        self.value_type_combo_box("v1", v1, ui);
                    }
                    _ => unreachable!(),
                }
                PinInfo::square().with_fill(NODE_IO_COLOR)
            }
            EMaterialNodeType::MaterialParamentersCollection((current_value, current_name)) => {
                //
                match pin.id.input {
                    0 => {
                        let text = if let Some(current_value) = current_value.as_ref() {
                            current_value.borrow().url.to_string()
                        } else {
                            "None".to_string()
                        };
                        ui.push_id("value", |ui| {
                            egui::ComboBox::from_label("")
                                .selected_text(format!("{}", text))
                                .show_ui(ui, |ui| {
                                    if ui
                                        .add(egui::Button::selectable(
                                            current_value.is_none(),
                                            "None",
                                        ))
                                        .clicked()
                                    {
                                        *current_value = None;
                                        self.is_updated = true;
                                    }

                                    let material_parameters_collections = {
                                        self.folder
                                            .borrow()
                                            .collect_material_parameters_collections(true)
                                    };

                                    for material_parameters_collection in
                                        material_parameters_collections
                                    {
                                        let (is_selected, text) = {
                                            let material_parameters_collection =
                                                material_parameters_collection.borrow();
                                            let current_url = current_value
                                                .as_ref()
                                                .map(|x| x.borrow().url.clone());
                                            (
                                                current_url
                                                    == Some(
                                                        material_parameters_collection.url.clone(),
                                                    ),
                                                material_parameters_collection.url.to_string(),
                                            )
                                        };
                                        if ui
                                            .add(egui::Button::selectable(is_selected, &text))
                                            .clicked()
                                        {
                                            *current_value = Some(material_parameters_collection);
                                            self.is_updated = true;
                                        }
                                    }
                                });
                        });
                    }
                    1 => {
                        let text = if let Some(current_name) = current_name.as_ref() {
                            current_name.to_string()
                        } else {
                            "None".to_string()
                        };
                        ui.push_id("name", |ui| {
                            egui::ComboBox::from_label("")
                                .selected_text(format!("{}", text))
                                .show_ui(ui, |ui| {
                                    let mut names: Vec<Option<String>> =
                                        vec![Some("None".to_string())];
                                    let mut field_names = current_value
                                        .as_ref()
                                        .map(|x| {
                                            x.borrow()
                                                .fields
                                                .iter()
                                                .map(|x| Some(x.name.clone()))
                                                .collect::<Vec<_>>()
                                        })
                                        .unwrap_or(vec![]);
                                    names.append(&mut field_names);

                                    for selected_value in names {
                                        let text = selected_value
                                            .as_ref()
                                            .cloned()
                                            .unwrap_or("None".to_string());
                                        self.is_updated = self.is_updated
                                            || ui
                                                .selectable_value(
                                                    current_name,
                                                    selected_value,
                                                    text,
                                                )
                                                .changed();
                                    }
                                });
                        });
                    }
                    _ => unimplemented!(),
                }
                PinInfo::default()
            }
        }
    }

    fn show_output(
        &mut self,
        pin: &OutPin,
        ui: &mut Ui,
        snarl: &mut Snarl<MaterialNode>,
    ) -> impl egui_snarl::ui::SnarlPin + 'static {
        let node = &mut snarl[pin.id.node];
        match &mut node.node_type {
            EMaterialNodeType::Add(..) => PinInfo::square().with_fill(NODE_IO_COLOR),
            EMaterialNodeType::Sink(..) => PinInfo::default(),
            EMaterialNodeType::Texture(_) => PinInfo::square().with_fill(NODE_IO_COLOR),
            EMaterialNodeType::TexCoord(index) => {
                let is_changed = egui::ComboBox::from_label("TexCoord")
                    .selected_text(format!("{}", index))
                    .show_ui(ui, |ui| {
                        if ui.selectable_value(index, 0, "0").clicked() {
                            self.is_updated = true;
                        }
                        if ui.selectable_value(index, 1, "1").clicked() {
                            self.is_updated = true;
                        }
                        if ui.selectable_value(index, 2, "2").clicked() {
                            self.is_updated = true;
                        }
                    })
                    .response
                    .changed();
                if is_changed {
                    self.is_updated = true;
                }
                PinInfo::square().with_fill(NODE_IO_COLOR)
            }
            EMaterialNodeType::VirtualTexture(_) => PinInfo::square().with_fill(NODE_IO_COLOR),
            EMaterialNodeType::Time => todo!(),
            EMaterialNodeType::Sin(_) => PinInfo::square().with_fill(NODE_IO_COLOR),
            EMaterialNodeType::MaterialParamentersCollection(_) => {
                PinInfo::square().with_fill(NODE_IO_COLOR)
            }
        }
    }

    fn show_header(
        &mut self,
        node: NodeId,
        inputs: &[InPin],
        outputs: &[OutPin],
        ui: &mut Ui,
        snarl: &mut Snarl<MaterialNode>,
    ) {
        ui.horizontal(|ui| {
            ui.label(format!("[{}] {}", node.0, self.title(&snarl[node])));
            let is_remove = if let EMaterialNodeType::Sink(..) = snarl[node].node_type {
                false
            } else {
                true
            };
            if is_remove {
                if ui.button("X").clicked() {
                    for input in inputs {
                        for remote in &input.remotes {
                            self.disconnect(
                                &OutPin {
                                    id: *remote,
                                    remotes: vec![input.id],
                                },
                                input,
                                snarl,
                            );
                        }
                    }
                    for output in outputs {
                        for remote in &output.remotes {
                            self.disconnect(
                                output,
                                &InPin {
                                    id: *remote,
                                    remotes: vec![output.id],
                                },
                                snarl,
                            );
                        }
                    }
                    snarl.remove_node(node);
                }
            }
        });
    }

    fn has_graph_menu(&mut self, pos: egui::Pos2, snarl: &mut Snarl<MaterialNode>) -> bool {
        let _ = (pos, snarl);
        true
    }

    fn show_graph_menu(&mut self, pos: egui::Pos2, ui: &mut Ui, snarl: &mut Snarl<MaterialNode>) {
        let node_types = vec![
            EMaterialNodeType::Add(EValueType::F32(0.0), EValueType::F32(0.0)),
            EMaterialNodeType::Texture(None),
            EMaterialNodeType::TexCoord(0),
            EMaterialNodeType::VirtualTexture(None),
            EMaterialNodeType::Time,
            EMaterialNodeType::Sin(EValueType::F32(0.0)),
            EMaterialNodeType::MaterialParamentersCollection((None, None)),
        ];

        for node_type in node_types {
            if ui.button(node_type.get_name()).clicked() {
                let node = MaterialNode { node_type };
                snarl.insert_node(pos, node);
                ui.close_kind(egui::UiKind::Menu);
            }
        }
    }

    fn connect(&mut self, from: &OutPin, to: &InPin, snarl: &mut Snarl<MaterialNode>) {
        for remote in &to.remotes {
            snarl.disconnect(*remote, to.id);
        }
        snarl.connect(from.id, to.id);
        self.is_updated = true;
    }

    fn disconnect(&mut self, from: &OutPin, to: &InPin, snarl: &mut Snarl<MaterialNode>) {
        snarl.disconnect(from.id, to.id);
        self.is_updated = true;
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Attribute {
    pub base_color: EValueType,
    pub metallic: EValueType,
    pub roughness: EValueType,
    pub normal: EValueType,
    pub opacity: EValueType,
    pub clear_coat: EValueType,
    pub clear_coat_roughness: EValueType,
}

impl Default for Attribute {
    fn default() -> Self {
        Self {
            base_color: EValueType::Vec3(glam::Vec3::ZERO),
            metallic: EValueType::F32(0.0),
            roughness: EValueType::F32(0.0),
            normal: EValueType::Vec3(glam::vec3(0.5, 0.5, 1.0)),
            opacity: EValueType::F32(1.0),
            clear_coat: EValueType::F32(0.0),
            clear_coat_roughness: EValueType::F32(0.0),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
pub enum EValueType {
    F32(f32),
    Vec2(glam::Vec2),
    Vec3(glam::Vec3),
    Vec4(glam::Vec4),
}

impl EValueType {
    pub fn get_type_name(&self) -> &str {
        match self {
            EValueType::F32(_) => "F32",
            EValueType::Vec2(_) => "Vec2",
            EValueType::Vec3(_) => "Vec3",
            EValueType::Vec4(_) => "Vec4",
        }
    }

    pub fn literal(&self) -> String {
        match self {
            EValueType::F32(value) => {
                format!("{:?}", value)
            }
            EValueType::Vec2(value) => {
                format!("vec2<f32>({:?}, {:?})", value.x, value.y)
            }
            EValueType::Vec3(value) => {
                format!("vec3<f32>({:?}, {:?}, {:?})", value.x, value.y, value.z)
            }
            EValueType::Vec4(value) => {
                format!(
                    "vec4<f32>({:?}, {:?}, {:?}, {:?})",
                    value.x, value.y, value.z, value.w
                )
            }
        }
    }

    pub fn convert_to_vec3(&self) -> EValueType {
        match *self {
            EValueType::F32(value) => EValueType::Vec3(glam::vec3(value, value, value)),
            EValueType::Vec2(value) => EValueType::Vec3(glam::vec3(value.x, value.y, 0.0)),
            EValueType::Vec3(_) => *self,
            EValueType::Vec4(value) => EValueType::Vec3(glam::vec3(value.x, value.y, value.z)),
        }
    }

    pub fn convert_to_f32(&self) -> EValueType {
        match *self {
            EValueType::F32(_) => *self,
            EValueType::Vec2(value) => EValueType::F32(value.x),
            EValueType::Vec3(value) => EValueType::F32(value.x),
            EValueType::Vec4(value) => EValueType::F32(value.x),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum EMaterialNodeType {
    Add(EValueType, EValueType),
    Texture(Option<url::Url>),
    VirtualTexture(Option<url::Url>),
    TexCoord(i32),
    Sink(Attribute),
    Time,
    Sin(EValueType),
    MaterialParamentersCollection(
        (
            Option<SingleThreadMutType<MaterialParamentersCollection>>,
            Option<String>,
        ),
    ),
}

impl EMaterialNodeType {
    pub fn get_name(&self) -> String {
        match self {
            EMaterialNodeType::Add(_, _) => format!("Add"),
            EMaterialNodeType::Texture(_) => format!("Texture"),
            EMaterialNodeType::VirtualTexture(_) => format!("VirtualTexture"),
            EMaterialNodeType::TexCoord(_) => format!("TexCoord"),
            EMaterialNodeType::Sink(_) => format!("Sink"),
            EMaterialNodeType::Time => format!("Time"),
            EMaterialNodeType::Sin(_) => format!("Sin"),
            EMaterialNodeType::MaterialParamentersCollection(_) => {
                format!("MaterialParamentersCollection")
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct MaterialNode {
    pub node_type: EMaterialNodeType,
}

pub enum EEventType {
    Update(
        Rc<RefCell<crate::material::Material>>,
        HashMap<MaterialOptions, ResolveResult>,
    ),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum EOutputValueType {
    Value(EValueType),
}

pub struct MaterialView {
    pub snarl: Snarl<MaterialNode>,
    pub style: SnarlStyle,
    pub viewer: GraphViewer,
    pub attribute_node_id: NodeId,
    pub event: Option<EEventType>,
    pub current_resolve_result: Option<HashMap<MaterialOptions, ResolveResult>>,
    pub validate: Option<HashMap<MaterialOptions, rs_render::error::Result<()>>>,
    pub folder: SingleThreadMutType<ContentFolder>,
}

impl MaterialView {
    pub fn new(folder: SingleThreadMutType<ContentFolder>) -> MaterialView {
        let mut snarl = Snarl::new();
        let mut style = SnarlStyle::new();
        style.bg_pattern = Some(egui_snarl::ui::BackgroundPattern::Grid(Grid {
            spacing: egui::emath::vec2(64.0, 64.0),
            angle: 0.0,
        }));
        style.wire_style = Some(egui_snarl::ui::WireStyle::AxisAligned { corner_radius: 5.0 });

        let material_parameters_collection_urls = {
            folder
                .borrow()
                .collect_material_parameters_collections(true)
                .iter()
                .map(|x| x.borrow().url.clone())
                .collect()
        };
        let viewer = GraphViewer {
            texture_urls: vec![],
            virtual_texture_urls: vec![],
            is_updated: false,
            material_parameters_collection_urls,
            folder: folder.clone(),
        };

        let node = MaterialNode {
            node_type: EMaterialNodeType::Sink(Default::default()),
        };
        let attribute_node_id = snarl.insert_node(egui::pos2(0.0, 0.0), node);

        MaterialView {
            snarl,
            style,
            viewer,
            attribute_node_id,
            event: None,
            current_resolve_result: None,
            validate: None,
            folder,
        }
    }

    pub fn default_resolve() -> anyhow::Result<HashMap<MaterialOptions, ResolveResult>> {
        let mut snarl = Snarl::new();
        let node = MaterialNode {
            node_type: EMaterialNodeType::Sink(Default::default()),
        };
        let _ = snarl.insert_node(egui::pos2(0.0, 0.0), node);
        material_resolve::resolve(&snarl, MaterialOptions::all())
    }

    pub fn draw(
        &mut self,
        current_open_material: Option<SingleThreadMutType<crate::material::Material>>,
        context: &egui::Context,
        data_source: &mut crate::ui::material_ui_window::DataSource,
    ) {
        let Some(material) = current_open_material else {
            return;
        };

        self.event = None;

        TopBottomPanel::top("material_menu_bar").show(context, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("Tool", |ui| {
                    if ui.add(Button::new("Debug Shader Code")).clicked() {
                        data_source.is_shader_code_window_open = true;
                    }
                });
                if ui.button("Apply").clicked() {
                    if let Some(current_resolve_result) = self.current_resolve_result.as_ref() {
                        self.event = Some(EEventType::Update(
                            material.clone(),
                            current_resolve_result.clone(),
                        ));
                    }
                }
            });
        });

        editor_ui::EditorUI::new_window("Shader Code", rs_engine::input_mode::EInputMode::UI)
            .open(&mut data_source.is_shader_code_window_open)
            .vscroll(true)
            .hscroll(true)
            .resizable(true)
            .show(context, |ui| {
                let current_resolve_result = &mut self.current_resolve_result;
                if let Some(current_resolve_result) = current_resolve_result {
                    if ui.button(format!("Validate")).clicked() {
                        let mut validates = HashMap::new();
                        for (k,v) in current_resolve_result.iter() {
                            let validate=rs_render::shader_library::ShaderLibrary::validate_shader_code(
                                &v.shader_code,
                            );
                            validates.insert(k.clone(), validate);
                        }
                        self.validate = Some(
                                validates
                        );
                    }
                    match &self.validate {
                        Some(validates) => {

                            for (_,validate) in validates {
                                match validate {
                                    Ok(_) => {
                                        ui.label(format!("Ok"));
                                    }
                                    Err(err) => {
                                        let theme =
                                            egui_extras::syntax_highlighting::CodeTheme::from_memory(
                                                ui.ctx(),
                                                ui.style()
                                            );
                                        egui_extras::syntax_highlighting::code_view_ui(
                                            ui,
                                            &theme,
                                            &err.to_string(),
                                            "wgsl",
                                        );
                                    }
                                }
                            }

                        },
                        None => {
                            ui.label(format!("None"));
                        }
                    }
                    ui.separator();
                    let theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx(), ui.style());
                    let clicked = egui_extras::syntax_highlighting::code_view_ui(
                        ui,
                        &theme,
                        &current_resolve_result.iter().find(|_|true).unwrap().1.shader_code,
                        "wgsl",
                    )
                    .clicked_by(PointerButton::Secondary);
                    if clicked {
                        ui.ctx()
                            .copy_text(current_resolve_result.iter().find(|_|true).unwrap().1.shader_code.clone());
                    }
                }
            });

        let snarl = &mut material.borrow_mut().snarl;
        let result = Self::do_draw(&mut self.viewer, &self.style, snarl, context);
        if let Some(result) = result {
            if let Ok(result) = result {
                self.current_resolve_result = Some(result.clone());
            }
        }
        self.viewer.is_updated = false;
    }

    fn do_draw(
        viewer: &mut GraphViewer,
        style: &SnarlStyle,
        snarl: &mut Snarl<MaterialNode>,
        context: &egui::Context,
    ) -> Option<anyhow::Result<HashMap<MaterialOptions, ResolveResult>>> {
        egui::SidePanel::left("Detail").show(context, |ui| {
            egui::ScrollArea::vertical().show(ui, |_| {});
        });

        egui::CentralPanel::default().show(context, |ui| {
            snarl.show(viewer, style, egui::Id::new("MaterialView"), ui);
        });

        if !viewer.is_updated {
            return None;
        }
        Some(material_resolve::resolve(snarl, MaterialOptions::all()))
    }
}
