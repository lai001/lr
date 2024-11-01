use egui::Context;
use rs_engine::console_cmd::ConsoleCmd;
use rs_foundation::new::SingleThreadMutType;
use std::collections::HashMap;

pub fn draw(
    window: egui::Window,
    context: &Context,
    open: &mut bool,
    console_cmds: &mut HashMap<String, SingleThreadMutType<ConsoleCmd>>,
) {
    window
        .open(open)
        .vscroll(true)
        .hscroll(true)
        .resizable(true)
        .default_size([250.0, 500.0])
        .show(context, |ui| {
            egui::Grid::new("Console Cmds")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    let mut keys = console_cmds
                        .keys()
                        .map(|x| x.to_string())
                        .collect::<Vec<String>>();
                    keys.sort();

                    for key in keys.iter() {
                        ui.label(key.clone());

                        let value = console_cmds.get(key).unwrap().clone();
                        let mut value = value.borrow_mut();
                        match &mut value.value {
                            rs_engine::console_cmd::EValue::I32(value) => {
                                ui.add(egui::DragValue::new(value).speed(1));
                            }
                            rs_engine::console_cmd::EValue::String(value) => {
                                ui.text_edit_singleline(value);
                            }
                            rs_engine::console_cmd::EValue::F32(value) => {
                                ui.add(egui::DragValue::new(value).speed(0.1));
                            }
                            rs_engine::console_cmd::EValue::Vec2(vec2) => {
                                ui.add(egui::DragValue::new(&mut vec2.x).speed(0.1).prefix("x: "));
                                ui.add(egui::DragValue::new(&mut vec2.y).speed(0.1).prefix("y: "));
                            }
                            rs_engine::console_cmd::EValue::Vec3(vec3) => {
                                ui.add(egui::DragValue::new(&mut vec3.x).speed(0.1).prefix("x: "));
                                ui.add(egui::DragValue::new(&mut vec3.y).speed(0.1).prefix("y: "));
                                ui.add(egui::DragValue::new(&mut vec3.z).speed(0.1).prefix("z: "));
                            }
                            rs_engine::console_cmd::EValue::Vec4(vec4) => {
                                ui.add(egui::DragValue::new(&mut vec4.x).speed(0.1).prefix("x: "));
                                ui.add(egui::DragValue::new(&mut vec4.y).speed(0.1).prefix("y: "));
                                ui.add(egui::DragValue::new(&mut vec4.z).speed(0.1).prefix("z: "));
                                ui.add(egui::DragValue::new(&mut vec4.w).speed(0.1).prefix("w: "));
                            }
                        }
                        ui.end_row();
                    }
                });
        });
}
