use rs_engine::content::texture::TextureFile;
use std::{cell::RefCell, rc::Rc};

#[derive(Debug)]
pub enum EClickEventType {
    IsVirtualTexture(bool),
}

pub fn draw(ui: &mut egui::Ui, texture_file: Rc<RefCell<TextureFile>>) -> Option<EClickEventType> {
    let mut click: Option<EClickEventType> = None;
    egui::Grid::new("PropertyGrid")
        .num_columns(2)
        .spacing([40.0, 4.0])
        .striped(true)
        .show(ui, |ui| {
            let mut texture_file = texture_file.borrow_mut();
            ui.label(format!("name: {}", texture_file.name.clone()));
            ui.end_row();
            ui.label(format!("url: {}", texture_file.url.to_string()));
            ui.end_row();
            if ui
                .checkbox(&mut texture_file.is_virtual_texture, "Is Virtual Texture")
                .changed()
            {
                click = Some(EClickEventType::IsVirtualTexture(
                    texture_file.is_virtual_texture,
                ));
            }
        });
    click
}
