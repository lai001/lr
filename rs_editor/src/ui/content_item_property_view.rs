use crate::content_edit::{ContentEdit, UIContentPropertyEvent};
use rs_foundation::new::SingleThreadMutType;
use std::path::PathBuf;

pub struct ContentItemPropertyView {
    pub content: Option<SingleThreadMutType<Box<dyn rs_content::Content>>>,
    pub image_asset_files: Vec<PathBuf>,
}

impl ContentItemPropertyView {
    pub fn new() -> ContentItemPropertyView {
        ContentItemPropertyView {
            content: None,
            image_asset_files: Vec::new(),
        }
    }

    pub fn draw(
        &mut self,
        ui: &mut egui::Ui,
        content_edit: &mut ContentEdit,
    ) -> Option<Box<dyn UIContentPropertyEvent>> {
        let Some(content) = &self.content.clone() else {
            return None;
        };
        self.render_window_content(content, ui, content_edit)
    }

    fn render_window_content(
        &mut self,
        content: &SingleThreadMutType<Box<dyn rs_content::Content>>,
        ui: &mut egui::Ui,
        content_edit: &mut ContentEdit,
    ) -> Option<Box<dyn UIContentPropertyEvent>> {
        {
            let content = content.borrow();
            ui.label(format!(
                "{} ({})",
                content.get_name(),
                content.get_type_text()
            ));
            ui.label(format!("Url:  {}", content.get_url().to_string()));
        }

        let editable = content_edit.editable(content.borrow().as_ref())?;
        editable.render_detail(content.clone(), self, ui)
    }
}
