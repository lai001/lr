use crate::{
    buffer_dimensions::BufferDimensions,
    command::TextureHandle,
    depth_texture::DepthTexture,
    error::Result,
    frame_buffer::FrameBuffer,
    gpu_vertex_buffer::GpuVertexBufferImp,
    render_pipeline::{
        virtual_texture_feed_back::{Constants, VirtualTextureFeedBackPipeline},
        virtual_texture_feed_back_clean::VirtualTextureFeedBackClearPipeline,
    },
    shader_library::ShaderLibrary,
    texture_readback::{create_read_buffer, map_texture_options2},
    virtual_texture_source::VirtualTextureSource,
};
use rs_core_minimal::{misc::get_mip_level_size, settings::VirtualTextureSetting};
use std::collections::{HashMap, HashSet};
use wgpu::{util::DeviceExt, *};

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
struct Value {
    x: u32,
    y: u32,
    lod: u32,
    id: u32,
}

#[derive(Debug)]
pub struct IndirectMapData {
    pub source: glam::UVec3,
    pub to: glam::UVec2,
}

const INVALID_ID: u32 = u32::MAX;

pub struct VirtualTexturePass {
    settings: VirtualTextureSetting,
    feeb_back_pipeline: VirtualTextureFeedBackPipeline,
    virtual_texture_feed_back_clear_pipeline: VirtualTextureFeedBackClearPipeline,
    feeb_back_frame_buffer: FrameBuffer,
    feed_back_size: glam::UVec2,
    feed_back_texture_format: TextureFormat,
    physical_texture: Texture,
    textures_cache: HashMap<u64, HashMap<glam::UVec3, Texture>>,
    indirect_table: Texture,
    indirect_table_datas: Vec<Vec<glam::UVec2>>,
    output_feeb_back: (Buffer, BufferDimensions),
    pub virtual_texture_sources: HashMap<TextureHandle, VirtualTextureSource>,
}

impl VirtualTexturePass {
    pub fn new(
        device: &Device,
        shader_library: &ShaderLibrary,
        is_noninterleaved: bool,
        surface_size: glam::UVec2,
        settings: VirtualTextureSetting,
    ) -> Result<Self> {
        let feed_back_size = surface_size / settings.feed_back_texture_div;
        let feed_back_texture_format = TextureFormat::Rgba32Uint;
        let feeb_back_frame_buffer =
            Self::create_feeb_back_frame_buffer(device, feed_back_texture_format, feed_back_size);
        let feeb_back_pipeline = VirtualTextureFeedBackPipeline::new(
            device,
            shader_library,
            &feed_back_texture_format,
            is_noninterleaved,
        );
        let virtual_texture_feed_back_clear_pipeline = VirtualTextureFeedBackClearPipeline::new(
            device,
            shader_library,
            &feed_back_texture_format,
        );
        let physical_texture = Self::create_physical_texture(device, &settings);
        let indirect_table = Self::create_indirect_table(device);

        let mut indirect_table_datas: Vec<Vec<glam::UVec2>> = Vec::new();
        for level in 0..indirect_table.mip_level_count() {
            let mip_level_size = indirect_table
                .size()
                .mip_level_size(level, TextureDimension::D2);
            let mut indirect_table_data: Vec<glam::UVec2> = Vec::new();
            indirect_table_data.resize(
                (mip_level_size.width * mip_level_size.height) as usize,
                glam::UVec2::splat(0),
            );
            indirect_table_datas.push(indirect_table_data);
        }

        let output_feeb_back = create_read_buffer(
            device,
            feeb_back_frame_buffer.get_color_texture(),
            None,
            None,
            Some("output_feeb_back"),
        )?;

        Ok(Self {
            settings,
            feeb_back_pipeline,
            feeb_back_frame_buffer,
            feed_back_size,
            feed_back_texture_format,
            virtual_texture_feed_back_clear_pipeline,
            physical_texture,
            virtual_texture_sources: HashMap::new(),
            textures_cache: HashMap::new(),
            indirect_table,
            indirect_table_datas,
            output_feeb_back,
        })
    }

    fn create_indirect_table(device: &Device) -> Texture {
        let size = glam::uvec2(8, 4);

        let texture_extent = wgpu::Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_extent,
            mip_level_count: rs_core_minimal::misc::calculate_max_mips(size.x.min(size.y)),
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TextureFormat::Rg32Uint,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
            label: Some("VirtualTexturePass.IndirectTable"),
        });

        texture
    }

    fn create_physical_texture(device: &Device, settings: &VirtualTextureSetting) -> Texture {
        let size = glam::uvec2(
            settings.physical_texture_size,
            settings.physical_texture_size,
        );

        let texture_extent = wgpu::Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: 1,
        };

        device.create_texture(&wgpu::TextureDescriptor {
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
            label: Some("VirtualTexturePass.PhysicalTexture"),
        })
    }

    fn create_feeb_back_frame_buffer(
        device: &Device,
        texture_format: TextureFormat,
        feed_back_size: glam::UVec2,
    ) -> FrameBuffer {
        let depth_texture = DepthTexture::new(
            feed_back_size.x,
            feed_back_size.y,
            device,
            Some("VirtualTexturePass.DepthTexture"),
        );
        FrameBuffer::new(
            device,
            feed_back_size,
            texture_format,
            Some(depth_texture),
            Some("VirtualTexturePass.FeedbackFrameBuffer"),
        )
    }

    pub fn change_surface_size(&mut self, device: &Device, surface_size: glam::UVec2) {
        self.feed_back_size = surface_size / self.settings.feed_back_texture_div;
        self.feeb_back_frame_buffer = Self::create_feeb_back_frame_buffer(
            device,
            self.feed_back_texture_format,
            self.feed_back_size,
        );
        self.output_feeb_back = create_read_buffer(
            device,
            self.feeb_back_frame_buffer.get_color_texture(),
            None,
            None,
            Some("output_feeb_back"),
        )
        .unwrap();
    }

    fn clear_physical_texture(&self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let mut command_encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("clear_physical_texture"),
        });
        let subresource_range = wgpu::ImageSubresourceRange {
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: Some(self.physical_texture.mip_level_count()),
            base_array_layer: 0,
            array_layer_count: Some(self.physical_texture.depth_or_array_layers()),
        };
        command_encoder.clear_texture(&self.physical_texture, &subresource_range);
        let _ = queue.submit([command_encoder.finish()]);
    }

    pub fn begin_new_frame(&self, device: &Device, queue: &Queue) {
        self.clear_physical_texture(device, queue);
        self.virtual_texture_feed_back_clear_pipeline.draw(
            device,
            queue,
            &self.feeb_back_frame_buffer.get_color_texture_view(),
            &self
                .feeb_back_frame_buffer
                .get_depth_texture_view()
                .expect("Not null"),
        );
    }

    pub fn render(
        &self,
        device: &Device,
        queue: &Queue,
        mesh_buffers: &[GpuVertexBufferImp],
        model: glam::Mat4,
        view: glam::Mat4,
        projection: glam::Mat4,
        id: u32,
    ) {
        let constants = Constants {
            model,
            view,
            projection,
            physical_texture_size: self.settings.physical_texture_size,
            scene_factor: self.settings.feed_back_texture_div,
            feedback_bias: self.settings.feedback_bias,
            id,
        };
        self.feeb_back_pipeline.draw(
            device,
            queue,
            &self.feeb_back_frame_buffer.get_color_texture_view(),
            &self
                .feeb_back_frame_buffer
                .get_depth_texture_view()
                .expect("Not null"),
            &constants,
            mesh_buffers,
        );
    }

    fn load_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_handle: u64,
        index: &glam::UVec3,
    ) {
        let Some(virtual_texture_source) = self.virtual_texture_sources.get_mut(&texture_handle)
        else {
            return;
        };
        if !self.textures_cache.contains_key(&texture_handle) {
            self.textures_cache.insert(texture_handle, HashMap::new());
        }
        let Some(textures_cach) = self.textures_cache.get_mut(&texture_handle) else {
            return;
        };
        if textures_cach.contains_key(&index) {
            return;
        }
        let Some(image) = virtual_texture_source.get_tile_image(index) else {
            return;
        };
        let texture_extent = wgpu::Extent3d {
            width: image.width(),
            height: image.height(),
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                size: texture_extent,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
                label: Some(&format!(
                    "VirtualTexturePass.CacheImage_{texture_handle}_{}_{}_{}",
                    index.x, index.y, index.z
                )),
            },
            util::TextureDataOrder::LayerMajor,
            image.as_bytes(),
        );
        textures_cach.insert(*index, texture);
    }

    pub fn parse_feed_back(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<HashMap<u32, Vec<glam::UVec3>>> {
        let texture = self.feeb_back_frame_buffer.get_color_texture();
        let image_data = map_texture_options2(
            device,
            queue,
            texture,
            0,
            1,
            &self.output_feeb_back.1,
            &self.output_feeb_back.0,
        )?;
        let buffer = &image_data[0];
        let values = rs_foundation::cast_to_type_buffer::<Value>(buffer);
        let mut table: HashMap<u32, HashSet<glam::UVec3>> = HashMap::new();

        for value in values {
            let id = value.id;
            if id == INVALID_ID {
                continue;
            }
            let Some(virtual_texture_source) = self.virtual_texture_sources.get(&(id as u64))
            else {
                continue;
            };

            let texture_size = virtual_texture_source.get_size();

            let max_mips = rs_core_minimal::misc::calculate_max_mips(texture_size.min_element());
            let max_lod = max_mips - self.settings.tile_size.ilog2() - 1;
            let lod = max_lod.min(value.lod);

            let texture_size = Extent3d {
                width: texture_size.x,
                height: texture_size.y,
                depth_or_array_layers: 1,
            };
            let texture_size = texture_size.mip_level_size(lod, TextureDimension::D2);

            if !table.contains_key(&id) {
                table.insert(id, HashSet::new());
            }
            let Some(tiles) = table.get_mut(&id) else {
                continue;
            };

            let x = (value.x as f32 / u32::MAX as f32 * texture_size.width as f32) as u32
                / self.settings.tile_size;
            let y = (value.y as f32 / u32::MAX as f32 * texture_size.height as f32) as u32
                / self.settings.tile_size;

            tiles.insert(glam::uvec3(x as u32, y as u32, lod));
        }
        let mut out: HashMap<u32, Vec<glam::UVec3>> = HashMap::new();
        for (k, v) in table {
            let mut v: Vec<glam::UVec3> = v.into_iter().collect();
            v.sort_by(|l, r| {
                let l = l.z * 1000000 + l.y * 100000 + l.x * 1000;
                let r = r.z * 1000000 + r.y * 100000 + r.x * 1000;
                l.cmp(&r)
            });
            out.insert(k, v);
        }
        Ok(out)
    }

    pub fn upload_physical_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        parse_feed_back_result: &HashMap<u32, Vec<glam::UVec3>>,
    ) -> HashMap<u32, Vec<IndirectMapData>> {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("upload_physical_texture"),
        });
        // let max_lod = rs_core_minimal::misc::calculate_max_mips(self.settings.tile_size);
        let mut indirect_map: HashMap<u32, Vec<IndirectMapData>> = HashMap::new();
        let mut next_upload_index: u32 = 0;
        for (texture_handle, tiles) in parse_feed_back_result {
            if !indirect_map.contains_key(texture_handle) {
                indirect_map.insert(*texture_handle, Vec::new());
            }
            let indirect_map_data = indirect_map.get_mut(texture_handle).unwrap();
            for tile_index in tiles {
                let mut lods: HashSet<u32> = HashSet::new();
                // lods.insert(((tile_index.z as i32) - 1).max(0) as u32);
                lods.insert(tile_index.z);
                // lods.insert(((tile_index.z as i32) + 1).min(max_lod as i32) as u32);
                for lod in lods {
                    let index = glam::uvec3(tile_index.x, tile_index.y, lod);
                    self.load_texture(device, queue, *texture_handle as u64, &index);
                    let Some(textures_cach) =
                        self.textures_cache.get_mut(&(*texture_handle as u64))
                    else {
                        continue;
                    };
                    let Some(texture) = textures_cach.get(&index) else {
                        continue;
                    };
                    let physical_texture_size = self.settings.physical_texture_size;
                    let tile_size = self.settings.tile_size;
                    let steps = physical_texture_size / tile_size;
                    let x = next_upload_index % steps;
                    let y = next_upload_index / steps;

                    indirect_map_data.push(IndirectMapData {
                        source: index,
                        to: glam::uvec2(x, y),
                    });
                    encoder.copy_texture_to_texture(
                        ImageCopyTexture {
                            texture: &texture,
                            mip_level: 0,
                            origin: Origin3d::ZERO,
                            aspect: TextureAspect::All,
                        },
                        ImageCopyTexture {
                            texture: &self.physical_texture,
                            mip_level: 0,
                            origin: Origin3d {
                                x: x * tile_size,
                                y: y * tile_size,
                                z: 0,
                            },
                            aspect: TextureAspect::All,
                        },
                        Extent3d {
                            width: tile_size,
                            height: tile_size,
                            depth_or_array_layers: 1,
                        },
                    );
                    next_upload_index += 1;
                }
            }
        }
        let command_buffer = encoder.finish();
        queue.submit(std::iter::once(command_buffer));
        indirect_map
    }

    pub fn update_indirec_table(
        &mut self,
        queue: &wgpu::Queue,
        indirect_map: HashMap<u32, Vec<IndirectMapData>>,
    ) {
        for (texture_handle, map_datas) in indirect_map {
            for map_data in map_datas {
                let source_tile = map_data.source;
                let to_tile = map_data.to;
                let lod = source_tile.z;
                let Some(indirect_table_data) = self.indirect_table_datas.get_mut(lod as usize)
                else {
                    continue;
                };
                let indirect_table_size = get_mip_level_size(self.indirect_table.width(), lod);
                let indirect_origin = glam::uvec2(
                    get_mip_level_size(8, lod) * texture_handle % indirect_table_size,
                    get_mip_level_size(4, lod) * texture_handle / indirect_table_size,
                );

                let index = (indirect_origin.y * indirect_table_size + indirect_origin.x)
                    + (source_tile.y * get_mip_level_size(8, lod) + source_tile.x);
                let Some(data) = indirect_table_data.get_mut(index as usize) else {
                    continue;
                };
                data.x = to_tile.x;
                data.y = to_tile.y;
            }
        }
        self.upload_indirec_table(queue);
    }

    fn upload_indirec_table(&mut self, queue: &wgpu::Queue) {
        for (level, indirect_table_data) in self.indirect_table_datas.iter().enumerate() {
            let level = level as u32;
            let mip_level_size = self
                .indirect_table
                .size()
                .mip_level_size(level, TextureDimension::D2);

            let buffer_dimensions = BufferDimensions::new(
                mip_level_size.width as usize,
                mip_level_size.height as usize,
                std::mem::size_of::<glam::UVec2>(),
            );

            queue.write_texture(
                ImageCopyTexture {
                    texture: &self.indirect_table,
                    mip_level: level,
                    origin: Origin3d { x: 0, y: 0, z: 0 },
                    aspect: TextureAspect::All,
                },
                &rs_foundation::cast_to_raw_buffer(&indirect_table_data),
                ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(buffer_dimensions.unpadded_bytes_per_row as u32),
                    rows_per_image: Some(mip_level_size.height),
                },
                Extent3d {
                    width: mip_level_size.width,
                    height: mip_level_size.height,
                    depth_or_array_layers: 1,
                },
            );
        }
    }

    pub fn get_physical_texture_view(&self) -> TextureView {
        self.physical_texture.create_view(&TextureViewDescriptor {
            label: Some("physical_texture_view"),
            format: Some(self.physical_texture.format()),
            dimension: Some(TextureViewDimension::D2),
            aspect: TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: Some(self.physical_texture.mip_level_count()),
            base_array_layer: 0,
            array_layer_count: Some(self.physical_texture.depth_or_array_layers()),
        })
    }

    pub fn get_indirect_table_view(&self) -> TextureView {
        self.indirect_table.create_view(&TextureViewDescriptor {
            label: Some("indirect_table_view"),
            format: Some(self.indirect_table.format()),
            dimension: Some(TextureViewDimension::D2),
            aspect: TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: Some(self.indirect_table.mip_level_count()),
            base_array_layer: 0,
            array_layer_count: Some(self.indirect_table.depth_or_array_layers()),
        })
    }
}
