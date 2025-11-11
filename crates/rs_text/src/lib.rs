pub mod cache;
pub mod error;

#[cfg(test)]
pub mod test {
    use crate::cache::{FontCache, GlyphKey};
    use image::{GenericImage, GenericImageView};
    use linebender_resource_handle::Blob;
    use parley::{
        Alignment, AlignmentOptions, FontContext, Layout, LayoutContext, PositionedLayoutItem,
        StyleProperty,
    };
    use parley::{FontFamily, FontStack};
    use rand::Rng;
    use std::collections::HashMap;
    use std::num::NonZeroUsize;
    use std::sync::Arc;
    use std::{borrow::Cow, collections::HashSet};
    use vello::kurbo::Vec2;
    use vello::{kurbo::Affine, *};
    use vello::{Renderer, RendererOptions};

    pub fn test_font_file_path() -> std::path::PathBuf {
        rs_core_minimal::file_manager::get_engine_resource(
            "Remote/Font/SourceHanSansHWSC/OTF/SimplifiedChineseHW/SourceHanSansHWSC-Bold.otf",
        )
    }

    pub fn test_font_file_path_fall() -> std::path::PathBuf {
        rs_core_minimal::file_manager::get_engine_resource(
            "Remote/Font/SourceHanSansHWSC/OTF/SimplifiedChineseHW/SourceHanSansHWSC-Regular.otf",
        )
    }

    #[test]
    fn fontdue_test() {
        let font_file = test_font_file_path();
        let font = std::fs::read(&font_file).unwrap();
        let font = fontdue::Font::from_bytes(font, fontdue::FontSettings::default()).unwrap();
        let px = 170.0f32;
        let character = 'A';
        let (metrics, bitmap) = if false {
            let index = font.lookup_glyph_index(character);
            let config = fontdue::layout::GlyphRasterConfig {
                glyph_index: index,
                px: px,
                font_hash: font.file_hash(),
            };
            let (metrics, bitmap) = font.rasterize_config(config);
            (metrics, bitmap)
        } else {
            let (metrics, bitmap) = font.rasterize(character, px);
            (metrics, bitmap)
        };
        println!("{:?}, {}", metrics, bitmap.len());
        let gray_image =
            image::GrayImage::from_vec(metrics.width as u32, metrics.height as u32, bitmap)
                .unwrap();
        let output = std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .join("fontdue_test.png");
        println!("Save to {:?}", &output);
        gray_image.save(output).unwrap();
    }

    #[test]
    fn ab_glyph_test() {
        let font_file = test_font_file_path();

        let font = std::fs::read(&font_file).unwrap();
        let font = ab_glyph::FontRef::try_from_slice(&font).unwrap();
        let character = 'A';
        let pixel_size = 170.0;
        let id = ab_glyph::Font::glyph_id(&font, character);
        let a_glyph = id.with_scale_and_position(pixel_size, ab_glyph::point(0.0, 0.0));
        let outline_glyph = ab_glyph::Font::outline_glyph(&font, a_glyph).unwrap();
        let px_bounds = outline_glyph.px_bounds();
        let mut gray_image = image::GrayImage::new(
            px_bounds.width().ceil() as u32,
            px_bounds.height().ceil() as u32,
        );
        println!("{:?}", px_bounds);
        outline_glyph.draw(|x, y, c| {
            let pixel_mut = gray_image.get_pixel_mut(x, y);
            pixel_mut.0[0] = (c * 255.0f32) as u8;
        });
        let output = std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .join("ab_glyph_test.png");
        println!("Save to {:?}", &output);
        gray_image.save(output).unwrap();
    }

    #[test]
    fn atlas_test() {
        let length = 512;
        let mut packer = rs_pack::skyline::SkylineBinPack::new(length, length);

        let font_file = test_font_file_path();

        let font = std::fs::read(&font_file).unwrap();
        let font = fontdue::Font::from_bytes(font, fontdue::FontSettings::default()).unwrap();

        let mut gray_image = image::GrayImage::new(length, length);
        let characters = "Whereas recognition of the inherent dignity and of the equal and inalienable rights of all
members of the human family is the foundation of freedom, justice and peace in the world";
        let mut rasterized: HashSet<char> = HashSet::new();
        for character in characters.chars() {
            if character.is_whitespace() || rasterized.contains(&character) {
                continue;
            }
            rasterized.insert(character);
            let px = rand::rng().random_range(15.0..55.0f32);
            let (metrics, bitmap) = if font.has_glyph(character) {
                let index = font.lookup_glyph_index(character);
                let config = fontdue::layout::GlyphRasterConfig {
                    glyph_index: index,
                    px: px,
                    font_hash: font.file_hash(),
                };
                let (metrics, bitmap) = font.rasterize_config(config);
                (metrics, bitmap)
            } else {
                let (metrics, bitmap) = font.rasterize(character, px);
                (metrics, bitmap)
            };
            let rect = packer.insert(metrics.width as u32, metrics.height as u32);
            if let Some(rect) = rect {
                let mut sub_image = gray_image.sub_image(
                    rect.x as u32,
                    rect.y as u32,
                    rect.width as u32,
                    rect.height as u32,
                );
                for y in 0..rect.height {
                    for x in 0..rect.width {
                        let value = bitmap[(rect.width * y + x) as usize];
                        sub_image.put_pixel(x, y, image::Luma([value]));
                    }
                }
            }
        }

        let output = std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .join("atlas_test.png");
        println!("Save to {:?}", &output);
        gray_image.save(output).unwrap();
    }

    struct FontTest {
        font_cache: FontCache,
        positioned_glyphs: Vec<(f32, f32, GlyphKey)>,
    }

    impl FontTest {
        fn new(max_width: Option<f32>) -> Self {
            let mut positioned_glyphs: Vec<(f32, f32, GlyphKey)> = vec![];
            let font_file = test_font_file_path();

            let mut font_cache = FontCache::default();
            let font_name = font_cache.push_font_file(None, &font_file).unwrap();
            let font_data = font_cache.font_data(&font_name).unwrap();

            let mut font_cx = FontContext::new();

            let _ = font_cx.collection.register_fonts(
                font_data,
                Some(parley::fontique::FontInfoOverride {
                    family_name: Some(font_name.as_str()),
                    ..Default::default()
                }),
            );

            let mut layout_cx = LayoutContext::new();

            let display_scale: f32 = 1.0;
            let text: &str = "Whereas recognition of the inherent dignity and of the equal and inalienable rights of all members of the human family is the foundation of freedom, justice and peace in the world
鉴于对人类家庭所有成员的固有尊严及其平等的和不移的权利的承认，乃是世界自由、正义与和平的基础";
            let mut builder = layout_cx.ranged_builder(&mut font_cx, text, display_scale, true);

            let font_size: u32 = 30;
            builder.push_default(StyleProperty::FontSize(font_size as f32));

            let binding = [FontFamily::Named(Cow::Borrowed(&font_name))];
            let font_stack = FontStack::List(Cow::Borrowed(&binding));
            builder.push_default(StyleProperty::FontStack(font_stack));

            let mut layout: Layout<()> = builder.build(text);

            layout.break_all_lines(max_width);
            layout.align(max_width, Alignment::Start, AlignmentOptions::default());

            // let width = layout.width();
            // let height = layout.height();
            let mut line_top_y = 0.0f32;
            // println!("Width: {}, Height: {}", width, height);
            for line in layout.lines() {
                let current_line_top_y = line_top_y;
                let line_metrics = line.metrics();
                line_top_y += line_metrics.line_height - line_metrics.leading;
                // println!("line_metrics: {:#?}", line_metrics);
                for item in line.items() {
                    match item {
                        PositionedLayoutItem::GlyphRun(glyph_run) => {
                            if font_cache
                                .font_name(glyph_run.run().font().data.id())
                                .is_none()
                            {
                                let _ = font_cache.push_font(
                                    Some(font_name.clone()),
                                    glyph_run.run().font().data.clone(),
                                );
                            }

                            let run_metrics = glyph_run.run().metrics();
                            // println!("run_metrics: {:#?}", run_metrics);
                            for glyph in glyph_run.positioned_glyphs() {
                                let character =
                                    *font_cache.character(&font_name, glyph.id).unwrap();
                                let glyph_key = GlyphKey {
                                    font_name: font_name.clone(),
                                    character: character,
                                    size: font_size,
                                };
                                let font_metrics = font_cache.metrics(glyph_key.clone()).unwrap();
                                let rasterized = font_cache.rasterized(glyph_key.clone());
                                if let Some(rasterized) = rasterized {
                                    let glyph_height = rasterized.height();
                                    let x = glyph.x + font_metrics.xmin as f32;
                                    let y = current_line_top_y + run_metrics.ascent
                                        - glyph_height as f32
                                        - font_metrics.ymin as f32
                                        + run_metrics.leading;
                                    positioned_glyphs.push((x, y, glyph_key));
                                }
                            }
                        }
                        _ => {}
                    };
                }
            }
            Self {
                font_cache,
                positioned_glyphs,
            }
        }
    }

    #[test]
    fn text_layout_test() {
        let length: u32 = 512;
        let mut gray_image = image::GrayImage::new(length, length);
        let mut font_test = FontTest::new(Some(length as f32));
        for glyph in &font_test.positioned_glyphs {
            if let Some(rasterized) = font_test.font_cache.rasterized(glyph.2.clone()) {
                let x = glyph.0;
                let y = glyph.1;
                let glyph_width = rasterized.width();
                let glyph_height = rasterized.height();
                let mut canvas =
                    gray_image.sub_image(x as u32, y as u32, glyph_width, glyph_height);
                let other = rasterized.to_image();
                let _ = canvas.copy_from(&other, 0, 0);
            }
        }
        let output = std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .join("text_layout_test.png");
        println!("Save to {:?}", &output);
        gray_image.save(output).unwrap();
    }

    pub fn random_pos<R: Rng + ?Sized>(
        rng: &mut R,
        outer_x_min: f64,
        outer_x_max: f64,
        outer_y_min: f64,
        outer_y_max: f64,
        inner_x_min: f64,
        inner_x_max: f64,
        inner_y_min: f64,
        inner_y_max: f64,
    ) -> Vec2 {
        let left_area = (inner_x_min - outer_x_min) * (outer_y_max - outer_y_min);
        let right_area = (outer_x_max - inner_x_max) * (outer_y_max - outer_y_min);
        let top_area = (inner_y_max - inner_y_min) * (inner_x_max - inner_x_min);
        let bottom_area = (inner_y_max - inner_y_min) * (inner_x_max - inner_x_min);

        let total_area = left_area + right_area + top_area + bottom_area;
        let r = rng.random::<f64>() * total_area;

        let mut uniform = |a: f64, b: f64| a + (b - a) * rng.random::<f64>();

        if r < left_area {
            Vec2::new(
                uniform(outer_x_min, inner_x_min),
                uniform(outer_y_min, outer_y_max),
            )
        } else if r < left_area + right_area {
            Vec2::new(
                uniform(inner_x_max, outer_x_max),
                uniform(outer_y_min, outer_y_max),
            )
        } else if r < left_area + right_area + top_area {
            Vec2::new(
                uniform(inner_x_min, inner_x_max),
                uniform(inner_y_max, outer_y_max),
            )
        } else {
            Vec2::new(
                uniform(inner_x_min, inner_x_max),
                uniform(outer_y_min, inner_y_min),
            )
        }
    }

    #[test]
    fn render_test() {
        let context =
            rs_render_core::wgpu_context::WGPUContext::windowless(None, None, None).unwrap();

        let device = context.get_device();
        let queue = context.get_queue();
        let mut renderer = Renderer::new(
            &device,
            RendererOptions {
                use_cpu: false,
                antialiasing_support: vello::AaSupport::all(),
                num_init_threads: NonZeroUsize::new(1),
                pipeline_cache: None,
            },
        )
        .expect("Failed to create renderer");

        let mut scene = vello::Scene::new();

        let width = 512;
        let height = 512;

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: width,
                height: height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::STORAGE_BINDING,
            view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
        });
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let fps = 24.0f32;

        let mut font_test = FontTest::new(Some(width as f32));

        let mut start: HashMap<usize, Vec2> = HashMap::new();
        let mut rng = <rand::rngs::SmallRng as rand::SeedableRng>::seed_from_u64(0);
        for i in 0..font_test.positioned_glyphs.len() {
            start.insert(
                i,
                random_pos(
                    &mut rng,
                    -200.0,
                    width as f64 + 200.0,
                    -200.0,
                    height as f64 + 200.0,
                    -100.0,
                    width as f64 + 100.0,
                    height as f64 + 100.0,
                    height as f64,
                ),
            );
        }

        for frame_number in 0..120 {
            let time = frame_number as f32 / fps;
            scene.reset();

            fn luma_to_rgba_with_alpha(
                luma_img: image::ImageBuffer<image::Luma<u8>, Vec<u8>>,
            ) -> image::ImageBuffer<image::Rgba<u8>, Vec<u8>> {
                let (width, height) = luma_img.dimensions();
                let mut rgba_img: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> =
                    image::ImageBuffer::new(width, height);
                for (x, y, pixel) in luma_img.enumerate_pixels() {
                    let luma_val = pixel[0];
                    let alpha = if luma_val == 0 { 0 } else { 255 };
                    rgba_img.put_pixel(x, y, image::Rgba([luma_val, luma_val, luma_val, alpha]));
                }
                rgba_img
            }

            let mut glyph_bitmaps: HashMap<GlyphKey, Blob<u8>> = HashMap::new();

            for (i, glyph) in font_test.positioned_glyphs.iter().enumerate() {
                if let Some(rasterized) = font_test.font_cache.rasterized(glyph.2.clone()) {
                    let x = glyph.0;
                    let y = glyph.1;
                    let glyph_width = rasterized.width();
                    let glyph_height = rasterized.height();

                    let blob = glyph_bitmaps.entry(glyph.2.clone()).or_insert_with(|| {
                        let rgba8_image = luma_to_rgba_with_alpha(rasterized.to_image());
                        let data = rgba8_image.as_raw().to_vec();
                        let blob = Blob::<u8>::new(Arc::new(data));
                        blob
                    });

                    let image = vello::peniko::ImageBrush {
                        image: vello::peniko::ImageData {
                            data: blob.clone(),
                            format: vello::peniko::ImageFormat::Rgba8,
                            width: glyph_width,
                            height: glyph_height,
                            alpha_type: vello::peniko::ImageAlphaType::Alpha,
                        },
                        sampler: vello::peniko::ImageSampler {
                            quality: vello::peniko::ImageQuality::High,
                            ..Default::default()
                        },
                    };
                    let source = start[&i];
                    let target = Vec2::new(x as f64, y as f64);
                    let factor = (time as f64 * 10.0 - (i as f64) * 0.2 as f64).clamp(0.0, 1.0);
                    let pos = source.lerp(target, factor);

                    scene.draw_image(&image, Affine::IDENTITY.then_translate(pos));
                }
            }

            renderer
                .render_to_texture(
                    &device,
                    &queue,
                    &scene,
                    &texture_view,
                    &vello::RenderParams {
                        base_color: peniko::Color::TRANSPARENT,
                        antialiasing_method: AaConfig::Msaa16,
                        width: width,
                        height: height,
                    },
                )
                .expect("Failed to render to a texture");

            let folder = std::env::current_exe()
                .unwrap()
                .parent()
                .unwrap()
                .join("render_test");
            if !folder.exists() {
                let _ = std::fs::create_dir_all(&folder);
            }
            let output = folder.join(format!("render_test_{:04}.png", frame_number));
            println!("Save to {:?}", &output);
            let data = rs_render_core::texture_readback::map_texture_full(device, queue, &texture)
                .unwrap()
                .remove(0)
                .remove(0);
            let image = image::RgbaImage::from_vec(width, height, data).unwrap();
            image.save(output).unwrap();
        }
    }
}
