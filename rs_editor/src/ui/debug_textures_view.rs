use egui::{load::SizedTexture, ImageSource, TextureId, Ui};
use rs_engine::resource_manager::ResourceManager;

pub enum EClickEventType {
    Selected(url::Url),
}

pub struct DebugTexturesView {
    pub all_texture_urls: Vec<url::Url>,
    pub current_ui_texture: Option<url::Url>,
}

impl DebugTexturesView {
    pub fn new() -> DebugTexturesView {
        DebugTexturesView {
            current_ui_texture: None,
            all_texture_urls: Vec::new(),
        }
    }

    pub fn draw(&mut self, ui: &mut Ui) -> Option<EClickEventType> {
        let mut event: Option<EClickEventType> = None;

        if ui
            .selectable_value(&mut self.current_ui_texture, None, "None")
            .clicked()
        {}
        for selectable_texture_url in self.all_texture_urls.iter() {
            let des = selectable_texture_url.to_string();
            if ui
                .selectable_value(
                    &mut self.current_ui_texture,
                    Some(selectable_texture_url.clone()),
                    des.clone(),
                )
                .clicked()
            {
                event = Some(EClickEventType::Selected(selectable_texture_url.clone()))
            }
        }

        let Some(current_ui_texture) = &self.current_ui_texture else {
            return None;
        };
        let rm = ResourceManager::default();
        match rm.clone().get_ui_texture_by_url(current_ui_texture) {
            Some(ui_texture_handle) => {
                ui.image(ImageSource::Texture(SizedTexture {
                    id: TextureId::User(*ui_texture_handle),
                    size: egui::Vec2 { x: 500.0, y: 500.0 },
                }));
            }
            None => {}
        }

        event
    }
}
