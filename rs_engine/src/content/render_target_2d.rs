use crate::engine::Engine;
use rs_artifact::{asset::Asset, resource_type::EResourceType};
use rs_render::command::TextureDescriptorCreateInfo;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct Runtime {
    texture: Option<crate::handle::TextureHandle>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RenderTarget2D {
    pub url: url::Url,
    pub width: u32,
    pub height: u32,
    pub format: wgpu::TextureFormat,
    #[serde(skip)]
    pub run_time: Option<Runtime>,
}

impl RenderTarget2D {
    pub fn new(
        url: url::Url,
        width: u32,
        height: u32,
        format: Option<wgpu::TextureFormat>,
    ) -> RenderTarget2D {
        RenderTarget2D {
            url,
            width,
            height,
            format: format.unwrap_or(wgpu::TextureFormat::Rgba8Unorm),
            run_time: Some(Runtime { texture: None }),
        }
    }

    pub fn default_length() -> u32 {
        256
    }

    pub fn get_name(&self) -> String {
        crate::url_extension::UrlExtension::get_name_in_editor(&self.url)
    }

    pub fn init_resouce(&mut self, engine: &mut Engine) {
        assert_ne!(0, self.width);
        assert_ne!(0, self.height);
        let label = self.get_name();
        let run_time = self
            .run_time
            .get_or_insert_with(|| Runtime { texture: None });
        let info = TextureDescriptorCreateInfo {
            label: Some(label.clone()),
            size: wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: None,
        };
        log::trace!(
            "Init RenderTarget2D, name: {}, width: {}, height: {}, format: {:?}",
            label,
            self.width,
            self.height,
            self.format
        );
        run_time.texture = Some(engine.create_texture(&self.url, info));
    }

    pub fn texture_handle(&self) -> Option<crate::handle::TextureHandle> {
        self.run_time.as_ref().map(|x| x.texture.clone()).flatten()
    }
}

impl Asset for RenderTarget2D {
    fn get_url(&self) -> url::Url {
        self.url.clone()
    }

    fn get_resource_type(&self) -> EResourceType {
        EResourceType::Content(rs_artifact::content_type::EContentType::RenderTarget2D)
    }
}
