use crate::{
    build_config::{BuildConfig, EArchType, EBuildPlatformType, EBuildType},
    data_source::DataSource,
};
use egui::{menu, Button, Context, TopBottomPanel};

#[derive(Debug)]
pub enum EWindowType {
    Asset,
    Texture,
    Property,
    Level,
}

#[derive(Debug)]
pub enum EToolType {
    IBL,
    DebugShader,
}

#[derive(Debug)]
pub enum EClickEventType {
    NewProject(String),
    OpenProject,
    OpenProjectSettings,
    SaveProject,
    Export,
    OpenVisualStudioCode,
    Build(BuildConfig),
    OpenWindow(EWindowType),
    Tool(EToolType),
}

pub struct TopMenu {
    pub new_project_name: String,
}

impl TopMenu {
    pub fn draw(
        &mut self,
        context: &Context,
        datasource: &mut DataSource,
    ) -> Option<EClickEventType> {
        let mut click: Option<EClickEventType> = None;
        TopBottomPanel::top("menu_bar").show(context, |ui| {
            menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    // ui.set_min_width(220.0);
                    // ui.style_mut().wrap = Some(false);

                    ui.menu_button("New Project", |ui| {
                        ui.text_edit_singleline(&mut self.new_project_name);
                        if ui.add(Button::new("OK")).clicked() {
                            click =
                                Some(EClickEventType::NewProject(self.new_project_name.clone()));
                        }
                    });
                    if ui.add(Button::new("Open Project")).clicked() {
                        click = Some(EClickEventType::OpenProject);
                        ui.close_menu();
                    }
                    if ui.add(Button::new("Open Project Settings")).clicked() {
                        click = Some(EClickEventType::OpenProjectSettings);
                        ui.close_menu();
                    }
                    if ui.add(Button::new("Save Project")).clicked() {
                        click = Some(EClickEventType::SaveProject);
                        ui.close_menu();
                    }
                    if ui.add(Button::new("Export")).clicked() {
                        click = Some(EClickEventType::Export);
                        ui.close_menu();
                    }
                    if ui.add(Button::new("Open Visual Studio Code")).clicked() {
                        click = Some(EClickEventType::OpenVisualStudioCode);
                        ui.close_menu();
                    }
                    ui.menu_button("Build", |ui| {
                        ui.menu_button("Windows", |ui| {
                            ui.menu_button("Debug", |ui| {
                                if ui.add(Button::new("x64")).clicked() {
                                    click = Some(EClickEventType::Build(BuildConfig {
                                        build_platform: EBuildPlatformType::Windows,
                                        build_type: EBuildType::Debug,
                                        arch_type: EArchType::X64,
                                    }));
                                    ui.close_menu();
                                }
                            });
                            ui.menu_button("Release", |ui| {
                                if ui.add(Button::new("x64")).clicked() {
                                    click = Some(EClickEventType::Build(BuildConfig {
                                        build_platform: EBuildPlatformType::Windows,
                                        build_type: EBuildType::Release,
                                        arch_type: EArchType::X64,
                                    }));
                                    ui.close_menu();
                                }
                            });
                        });
                    });
                });
                ui.menu_button("Window", |ui| {
                    if ui.add(Button::new("Asset")).clicked() {
                        click = Some(EClickEventType::OpenWindow(EWindowType::Asset));
                        ui.close_menu();
                    }
                    if ui.add(Button::new("Texture")).clicked() {
                        click = Some(EClickEventType::OpenWindow(EWindowType::Texture));
                        ui.close_menu();
                    }
                    if ui.add(Button::new("Property")).clicked() {
                        click = Some(EClickEventType::OpenWindow(EWindowType::Property));
                        ui.close_menu();
                    }
                    if ui.add(Button::new("Level")).clicked() {
                        click = Some(EClickEventType::OpenWindow(EWindowType::Level));
                        ui.close_menu();
                    }
                });
                ui.menu_button("Tool", |ui| {
                    ui.menu_button("IBL", |ui| {
                        let ibl_bake_info = &mut datasource.ibl_bake_info;
                        ui.add(
                            egui::DragValue::new(&mut ibl_bake_info.brdf_sample_count)
                                .speed(1)
                                .prefix("BRDF Sample Count: ")
                                .clamp_range(1..=8192),
                        );
                        ui.add(
                            egui::DragValue::new(&mut ibl_bake_info.irradiance_sample_count)
                                .speed(1)
                                .prefix("Irradiance Sample Count: ")
                                .clamp_range(1..=8192),
                        );
                        ui.add(
                            egui::DragValue::new(&mut ibl_bake_info.pre_filter_sample_count)
                                .speed(1)
                                .prefix("Prefilter Sample Count: ")
                                .clamp_range(1..=8192),
                        );
                        ui.add(
                            egui::DragValue::new(&mut ibl_bake_info.brdflutmap_length)
                                .speed(1)
                                .prefix("BRDF Length: ")
                                .clamp_range(64..=2048),
                        );
                        ui.add(
                            egui::DragValue::new(
                                &mut ibl_bake_info.pre_filter_cube_map_max_mipmap_level,
                            )
                            .speed(1)
                            .prefix("Prefilter Max Mipmap: ")
                            .clamp_range(1..=64),
                        );
                        ui.add(
                            egui::DragValue::new(&mut ibl_bake_info.irradiance_cube_map_length)
                                .speed(1)
                                .prefix("Irradiance Length: ")
                                .clamp_range(4..=8192),
                        );
                        ui.add(
                            egui::DragValue::new(&mut ibl_bake_info.pre_filter_cube_map_length)
                                .speed(1)
                                .prefix("Prefilter Cube Map Length: ")
                                .clamp_range(4..=8192),
                        );
                        if ui.add(Button::new("Bake")).clicked() {
                            click = Some(EClickEventType::Tool(EToolType::IBL));
                            ui.close_menu();
                        }
                    });
                    if ui.add(Button::new("Debug Shader")).clicked() {
                        click = Some(EClickEventType::Tool(EToolType::DebugShader));
                        ui.close_menu();
                    }
                });
            });
        });

        click
    }
}
