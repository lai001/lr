use super::{tile_index::TileIndex, virtual_texture_configuration::VirtualTextureConfiguration};
use crate::{
    actor::Actor,
    compute_pipeline::update_page_table::UpdatePageTableCSPipeline,
    default_textures::DefaultTextures,
    depth_texture::DepthTexture,
    render_pipeline::{
        feed_back_pipeline::FeedBackPipeline,
        virtual_texture_clean_pipeline::VirtualTextureCleanPipeline,
    },
    virtual_texture::tile_index::TileOffset,
};
use image::{ImageBuffer, Rgba};
use rs_foundation::cast_to_type_buffer;
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
    virtual_texture_configuration: VirtualTextureConfiguration,
    update_page_table_cs_pipeline: UpdatePageTableCSPipeline,
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
            label: Some("VirtualTextureSystem.physical_texture"),
            size: Extent3d {
                width: physical_texture_size,
                height: physical_texture_size,
                depth_or_array_layers: virtual_texture_configuration.physical_texture_array_size,
            },
            mip_level_count: 1,
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
            label: Some("VirtualTextureSystem.page_table_texture"),
            size: Extent3d {
                width: page_table_size,
                height: page_table_size,
                depth_or_array_layers: virtual_texture_configuration.get_max_mipmap_level() as u32,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba16Uint,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_SRC
                | TextureUsages::COPY_DST
                | TextureUsages::STORAGE_BINDING,
            // | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let feed_back_texture = device.create_texture(&TextureDescriptor {
            label: Some("VirtualTextureSystem.feed_back_texture"),
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
                | TextureUsages::RENDER_ATTACHMENT
                | TextureUsages::STORAGE_BINDING,
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
            virtual_texture_configuration,
        );

        let feed_back_depth_texture =
            DepthTexture::new(feed_back_texture_width, feed_back_texture_height, device);
        let feed_back_texture_clean_pipeline =
            VirtualTextureCleanPipeline::new(device, &feed_back_texture.format());

        let update_page_table_cs_pipeline = UpdatePageTableCSPipeline::new(device);

        VirtualTextureSystem {
            physical_texture: Arc::new(Some(physical_texture)),
            page_table_texture: Arc::new(Some(page_table_texture)),
            feed_back_texture,
            feed_back_depth_texture,
            page_table_size,
            feed_back_pipeline,
            feed_back_texture_clean_pipeline,
            virtual_texture_configuration,
            update_page_table_cs_pipeline,
        }
    }

    pub fn new_frame(&mut self, device: &Device, queue: &Queue) {
        self.clear_physical_texture(device, queue);
        self.clear_page_texture(device, queue);
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
                load: wgpu::LoadOp::Clear(Color::TRANSPARENT),
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
            let type_buffer: &[u16] = cast_to_type_buffer(&padded_buffer);
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
                        uniq.insert(
                            TileIndex {
                                tile_offset: TileOffset { x, y },
                                mipmap_level,
                            },
                            true,
                        );
                    }
                }
            }
            let pages: Vec<TileIndex> = uniq.keys().map(|x| *x).collect();
            drop(padded_buffer);
            output_buffer.unmap();
            pages
        } else {
            panic!()
        }
    }

    pub fn upload_physical_page_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture: &Texture,
        array_tile: &super::packing::ArrayTile,
    ) {
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
                mip_level: 0,
                origin: Origin3d {
                    x: array_tile.offset_x,
                    y: array_tile.offset_y,
                    z: array_tile.index,
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
            Some(texture) => {
                let mut texture_view_descriptor = wgpu::TextureViewDescriptor::default();
                texture_view_descriptor.dimension = Some(TextureViewDimension::D2);
                texture.create_view(&texture_view_descriptor)
            }
            None => DefaultTextures::default()
                .lock()
                .unwrap()
                .get_black_texture_view(),
        }
    }

    pub fn get_page_table_texture_view(&self) -> wgpu::TextureView {
        match self.page_table_texture.clone().as_ref() {
            Some(texture) => {
                let texture_view_descriptor = wgpu::TextureViewDescriptor {
                    label: Some("VirtualTextureSystem.page_table_texture_view"),
                    format: Some(TextureFormat::Rgba8Uint),
                    dimension: Some(TextureViewDimension::D2),
                    aspect: wgpu::TextureAspect::All,
                    base_mip_level: 0,
                    mip_level_count: None,
                    base_array_layer: 0,
                    array_layer_count: None,
                };
                texture.create_view(&texture_view_descriptor)
            }
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
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        pack_result: &HashMap<TileIndex, super::packing::ArrayTile>,
    ) {
        self.update_page_table_cs_pipeline.execute(
            device,
            queue,
            self.page_table_texture.as_ref().as_ref().unwrap(),
            pack_result,
        );
    }

    pub fn clear_physical_texture(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("clear_physical_texture"),
        });

        encoder.clear_texture(
            self.physical_texture.as_ref().as_ref().unwrap(),
            &ImageSubresourceRange {
                aspect: TextureAspect::All,
                base_mip_level: 0,
                mip_level_count: Some(1),
                base_array_layer: 0,
                array_layer_count: None,
            },
        );

        let command_buffer = encoder.finish();
        let _ = queue.submit(std::iter::once(command_buffer));
    }

    pub fn clear_page_texture(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        encoder.clear_texture(
            self.page_table_texture.as_ref().as_ref().unwrap(),
            &ImageSubresourceRange {
                aspect: TextureAspect::All,
                base_mip_level: 0,
                mip_level_count: Some(1),
                base_array_layer: 0,
                array_layer_count: None,
            },
        );

        let command_buffer = encoder.finish();
        let _ = queue.submit(std::iter::once(command_buffer));
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
