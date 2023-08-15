use super::{tile_index::TileIndex, virtual_texture_configuration::VirtualTextureConfiguration};
use crate::{
    actor::Actor,
    buffer_dimensions::BufferDimensions,
    default_textures::DefaultTextures,
    depth_texture::DepthTexture,
    render_pipeline::{
        feed_back_pipeline::FeedBackPipeline,
        virtual_texture_clean_pipeline::VirtualTextureCleanPipeline,
    },
};
use image::{ImageBuffer, Rgba};
use std::{collections::HashMap, sync::Arc};
use wgpu::*;
use winit::dpi::PhysicalSize;

pub struct VirtualTextureSystem {
    physical_texture: Arc<Option<Texture>>,
    page_table_texture: Arc<Option<Texture>>,
    feed_back_texture: Texture,
    feed_back_depth_texture: DepthTexture,
    page_table_size: u32,
    feed_back_pipeline: FeedBackPipeline,
    feed_back_texture_clean_pipeline: VirtualTextureCleanPipeline,
    page_table_data: Vec<u8>,
    page_buffer_dimensions: BufferDimensions,
    virtual_texture_configuration: VirtualTextureConfiguration,
}

impl VirtualTextureSystem {
    pub fn new(
        device: &Device,
        virtual_texture_configuration: VirtualTextureConfiguration,
        feed_back_texture_width: u32,
        feed_back_texture_height: u32,
        physical_texture_color_format: TextureFormat,
    ) -> VirtualTextureSystem {
        let physical_texture_size = virtual_texture_configuration.physical_texture_size;
        let virtual_texture_size = virtual_texture_configuration.virtual_texture_size;
        let tile_size = virtual_texture_configuration.tile_size;

        assert_eq!(physical_texture_size % tile_size, 0);
        assert_eq!(virtual_texture_size % tile_size, 0);

        let available_texture_formats = HashMap::from([
            (TextureFormat::Rgba8Unorm, true),
            (TextureFormat::Rgba8UnormSrgb, true),
            (TextureFormat::Bgra8Unorm, true),
            (TextureFormat::Bgra8UnormSrgb, true),
        ]);
        assert!(available_texture_formats.contains_key(&physical_texture_color_format));
        let page_table_size = virtual_texture_size / tile_size;

        let physical_texture = device.create_texture(&TextureDescriptor {
            label: None,
            size: Extent3d {
                width: physical_texture_size,
                height: physical_texture_size,
                depth_or_array_layers: 1,
            },
            mip_level_count: virtual_texture_configuration.get_max_mipmap_level() as u32,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: physical_texture_color_format,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_SRC
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let page_table_texture = device.create_texture(&TextureDescriptor {
            label: None,
            size: Extent3d {
                width: page_table_size,
                height: page_table_size,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Uint,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_SRC
                | TextureUsages::COPY_DST,
            // | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let feed_back_texture = device.create_texture(&TextureDescriptor {
            label: None,
            size: Extent3d {
                width: feed_back_texture_width,
                height: feed_back_texture_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba16Uint,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_SRC
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let feed_back_pipeline = FeedBackPipeline::new(
            device,
            Some(DepthStencilState {
                depth_compare: CompareFunction::Less,
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            &feed_back_texture.format(),
        );

        let feed_back_depth_texture =
            DepthTexture::new(feed_back_texture_width, feed_back_texture_height, device);
        let feed_back_texture_clean_pipeline =
            VirtualTextureCleanPipeline::new(device, &feed_back_texture.format());

        let page_buffer_dimensions = crate::buffer_dimensions::BufferDimensions::new(
            page_table_size as usize,
            page_table_size as usize,
            4 * std::mem::size_of::<u8>(),
        );
        let page_table_data: Vec<u8> =
            vec![0 as u8; page_buffer_dimensions.get_padded_width() * page_table_size as usize * 4];

        VirtualTextureSystem {
            physical_texture: Arc::new(Some(physical_texture)),
            page_table_texture: Arc::new(Some(page_table_texture)),
            feed_back_texture,
            feed_back_depth_texture,
            page_table_size,
            feed_back_pipeline,
            feed_back_texture_clean_pipeline,
            page_table_data,
            page_buffer_dimensions,
            virtual_texture_configuration,
        }
    }

    pub fn new_frame(&mut self, device: &Device, queue: &Queue) {
        let output_texture_view_descriptor = TextureViewDescriptor {
            label: None,
            format: Some(self.feed_back_texture.format()),
            dimension: Some(TextureViewDimension::D2),
            aspect: TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: Some(1),
        };
        let output_view: TextureView = self
            .feed_back_texture
            .create_view(&output_texture_view_descriptor);
        let depth_view: TextureView = self.feed_back_depth_texture.get_view();
        self.feed_back_texture_clean_pipeline.draw(
            device,
            queue,
            &output_view,
            &depth_view,
            wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: true,
            },
            Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: true,
            }),
            Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(0),
                store: true,
            }),
        );
    }

    pub fn render_actor(
        &self,
        device: &Device,
        queue: &Queue,
        actor: &Actor,
        camera: &crate::camera::Camera,
    ) {
        let output_texture_view_descriptor = TextureViewDescriptor {
            label: None,
            format: Some(self.feed_back_texture.format()),
            dimension: Some(TextureViewDimension::D2),
            aspect: TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: Some(1),
        };
        let output_view: TextureView = self
            .feed_back_texture
            .create_view(&output_texture_view_descriptor);
        let depth_view: TextureView = self.feed_back_depth_texture.get_view();
        self.feed_back_pipeline.render_actor(
            device,
            queue,
            &output_view,
            &depth_view,
            actor,
            camera,
            self.feed_back_texture.size().width,
            self.feed_back_texture.size().height,
            Some(wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: true,
            }),
            Some(wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: true,
            }),
        )
    }

    pub fn get_feed_back_texture_size(&self) -> PhysicalSize<u32> {
        let texture_size = self.feed_back_texture.size();
        PhysicalSize::<u32> {
            width: texture_size.width,
            height: texture_size.height,
        }
    }

    pub fn read(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> Vec<TileIndex> {
        let texture_size = self.feed_back_texture.size();

        let width = texture_size.width;
        let height = texture_size.height;
        let bytes_per_pixel: usize = 4 * std::mem::size_of::<u16>();
        let buffer_dimensions = crate::buffer_dimensions::BufferDimensions::new(
            width as usize,
            height as usize,
            bytes_per_pixel,
        );
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (buffer_dimensions.padded_bytes_per_row * buffer_dimensions.height) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let texture_extent = wgpu::Extent3d {
            width: buffer_dimensions.width as u32,
            height: buffer_dimensions.height as u32,
            depth_or_array_layers: 1,
        };
        encoder.copy_texture_to_buffer(
            self.feed_back_texture.as_image_copy(),
            wgpu::ImageCopyBuffer {
                buffer: &output_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(buffer_dimensions.padded_bytes_per_row as u32),
                    rows_per_image: None,
                },
            },
            texture_extent,
        );
        let command_buffer = encoder.finish();
        let submission_index = queue.submit(std::iter::once(command_buffer));
        let buffer_slice = output_buffer.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());
        device.poll(wgpu::Maintain::WaitForSubmissionIndex(submission_index));
        if let Ok(Ok(_)) = receiver.recv() {
            let padded_buffer = buffer_slice.get_mapped_range();
            let type_buffer: &[u16] = crate::util::cast_to_type_buffer(&padded_buffer);
            let line_buffer_chunks = type_buffer
                .chunks(buffer_dimensions.padded_bytes_per_row / std::mem::size_of::<u16>());
            assert_eq!(line_buffer_chunks.len(), height as usize);
            let mut uniq: HashMap<TileIndex, bool> = HashMap::new();
            for line in line_buffer_chunks {
                for data in line.chunks(4) {
                    debug_assert_eq!(data.len(), 4);
                    let is_valid = *data.get(3).unwrap() == 1;
                    if is_valid {
                        let x = *data.get(0).unwrap() as u16;
                        let y = *data.get(1).unwrap() as u16;
                        let mipmap_level = *data.get(2).unwrap() as u8;
                        uniq.insert(TileIndex { x, y, mipmap_level }, true);
                    }
                }
            }
            let mut pages: Vec<TileIndex> = uniq.keys().map(|x| *x).collect();
            pages.sort_by(|a, b| {
                if a.mipmap_level == b.mipmap_level {
                    let a = width * a.y as u32 + a.x as u32;
                    let b = width * b.y as u32 + b.x as u32;
                    a.cmp(&b)
                } else {
                    a.mipmap_level.cmp(&b.mipmap_level)
                }
            });
            drop(padded_buffer);
            output_buffer.unmap();
            pages
        } else {
            panic!()
        }
    }

    pub fn upload_page_table(&mut self, queue: &wgpu::Queue) {
        let buffer_dimensions = &self.page_buffer_dimensions;

        let texture_extent = wgpu::Extent3d {
            depth_or_array_layers: 1,
            width: self.page_table_size,
            height: self.page_table_size,
        };

        queue.write_texture(
            self.page_table_texture
                .as_ref()
                .as_ref()
                .unwrap()
                .as_image_copy(),
            crate::util::cast_to_raw_buffer(&self.page_table_data),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(buffer_dimensions.padded_bytes_per_row as u32),
                rows_per_image: None,
            },
            texture_extent,
        );
    }

    pub fn upload_page_image(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        page: (u16, u16),
        image: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    ) {
        let tile_size = self.virtual_texture_configuration.tile_size;
        assert_eq!(image.width(), tile_size);
        assert_eq!(image.height(), tile_size);
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let buffer_dimensions = crate::buffer_dimensions::BufferDimensions::new(
            image.width() as usize,
            image.height() as usize,
            4,
        );
        let page_texture = crate::util::texture2d_from_rgba_image(device, queue, image);

        encoder.copy_texture_to_texture(
            ImageCopyTexture {
                texture: &page_texture,
                mip_level: 0,
                origin: Origin3d { x: 0, y: 0, z: 0 },
                aspect: TextureAspect::All,
            },
            ImageCopyTexture {
                texture: self.physical_texture.as_ref().as_ref().unwrap(),
                mip_level: 0,
                origin: Origin3d {
                    x: page.0 as u32 * tile_size,
                    y: page.1 as u32 * tile_size,
                    z: 0,
                },
                aspect: TextureAspect::All,
            },
            Extent3d {
                width: buffer_dimensions.get_padded_width() as u32,
                height: image.height(),
                depth_or_array_layers: 1,
            },
        );

        let command_buffer = encoder.finish();
        let _ = queue.submit(std::iter::once(command_buffer));
    }

    pub fn upload_page_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        page: TileIndex,
        texture: Arc<Texture>,
    ) {
        {
            let size = wgpu::Extent3d {
                width: self.virtual_texture_configuration.tile_size,
                height: self.virtual_texture_configuration.tile_size,
                depth_or_array_layers: 1,
            };
            let size = size.mip_level_size(page.mipmap_level as u32, TextureDimension::D2);
            assert_eq!(size.width, texture.size().width);
            assert_eq!(size.height, texture.size().width);
        }

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        encoder.copy_texture_to_texture(
            ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: Origin3d { x: 0, y: 0, z: 0 },
                aspect: TextureAspect::All,
            },
            ImageCopyTexture {
                texture: self.physical_texture.as_ref().as_ref().unwrap(),
                mip_level: page.mipmap_level as u32,
                origin: Origin3d {
                    x: page.x as u32 * texture.width(),
                    y: page.y as u32 * texture.height(),
                    z: 0,
                },
                aspect: TextureAspect::All,
            },
            Extent3d {
                width: texture.width(),
                height: texture.height(),
                depth_or_array_layers: 1,
            },
        );

        let command_buffer = encoder.finish();
        let _ = queue.submit(std::iter::once(command_buffer));
    }

    pub fn get_physical_texture_view(&self) -> wgpu::TextureView {
        match self.physical_texture.clone().as_ref() {
            Some(texture) => texture.create_view(&wgpu::TextureViewDescriptor::default()),
            None => DefaultTextures::default()
                .lock()
                .unwrap()
                .get_black_texture_view(),
        }
    }

    pub fn get_page_table_texture_view(&self) -> wgpu::TextureView {
        let texture_view_descriptor = wgpu::TextureViewDescriptor {
            label: Some("page table"),
            format: Some(TextureFormat::Rgba16Uint),
            dimension: Some(TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: Some(1),
            base_array_layer: 0,
            array_layer_count: Some(1),
        };
        match self.page_table_texture.clone().as_ref() {
            Some(texture) => texture.create_view(&texture_view_descriptor),
            None => {
                DefaultTextures::default()
                    .lock()
                    .unwrap()
                    .get_black_texture_view()
                // panic!()
            }
        }
    }

    pub fn update_page_table(
        &mut self,
        virtual_tile_index: TileIndex,
        physical_tile_index: TileIndex,
        debug: u8,
    ) {
        assert!(
            physical_tile_index.x
                <= (self.virtual_texture_configuration.physical_texture_size
                    / self.virtual_texture_configuration.tile_size) as u16
        );
        assert!(
            physical_tile_index.y
                <= (self.virtual_texture_configuration.physical_texture_size
                    / self.virtual_texture_configuration.tile_size) as u16
        );

        let buffer_dimensions = &self.page_buffer_dimensions;
        let chunks = self
            .page_table_data
            .chunks_mut(buffer_dimensions.get_padded_width() * 4);
        let line = chunks.skip(virtual_tile_index.y as usize).next().unwrap();
        let page_data = line
            .chunks_mut(4)
            .skip(virtual_tile_index.x as usize)
            .next()
            .unwrap();
        page_data[0] = physical_tile_index.x as u8;
        page_data[1] = physical_tile_index.y as u8;
        page_data[2] = physical_tile_index.mipmap_level;
        page_data[3] = debug;
    }

    pub fn clean_page_table(&mut self, value: u8) {
        self.page_table_data.fill(value);
    }

    pub fn get_physical_texture(&self) -> Arc<Option<Texture>> {
        self.physical_texture.clone()
    }

    pub fn get_page_table_texture(&self) -> Arc<Option<Texture>> {
        self.page_table_texture.clone()
    }

    pub fn get_physical_texture_size(&self) -> u32 {
        self.virtual_texture_configuration.physical_texture_size
    }

    pub fn get_tile_size(&self) -> u32 {
        self.virtual_texture_configuration.tile_size
    }
}
