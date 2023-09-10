use std::path::Path;
use std::process::Command;

pub struct SDF2DGenerator {}

impl SDF2DGenerator {
    pub fn create(source_image: &image::RgbaImage) -> image::Rgba32FImage {
        let directions: Vec<glam::Vec2> = vec![
            glam::vec2(-1.0, 0.0),
            glam::vec2(0.0, -1.0),
            glam::vec2(1.0, 0.0),
            glam::vec2(0.0, 1.0),
            glam::vec2(-1.0, -1.0),
            glam::vec2(1.0, -1.0),
            glam::vec2(1.0, 1.0),
            glam::vec2(-1.0, 1.0),
        ];

        let mut inner: Vec<glam::Vec2> = Vec::new();
        let mut outer: Vec<glam::Vec2> = Vec::new();

        let width = source_image.width();
        let height = source_image.height();

        for x in 0..width {
            for y in 0..height {
                let uv = glam::Vec2 {
                    x: x as f32,
                    y: y as f32,
                };

                let source_pixel = source_image.get_pixel(x, y);
                if source_pixel.0[0] <= 127 {
                    for direction in &directions {
                        let mut offset = uv + *direction;
                        offset = offset.clamp(
                            glam::vec2(0.0, 0.0),
                            glam::vec2(width as f32, height as f32) - 1.0,
                        );
                        let offset_pixel = source_image.get_pixel(offset.x as u32, offset.y as u32);
                        if offset_pixel[0] > 127 {
                            outer.push(offset);
                            break;
                        }
                    }
                } else {
                    for direction in &directions {
                        let mut offset = uv + *direction;
                        offset = offset.clamp(
                            glam::vec2(0.0, 0.0),
                            glam::vec2(width as f32, height as f32) - 1.0,
                        );
                        let offset_pixel = source_image.get_pixel(offset.x as u32, offset.y as u32);
                        if offset_pixel[0] <= 127 {
                            inner.push(offset);
                            break;
                        }
                    }
                }
            }
        }

        let mut output_image: image::Rgba32FImage = image::Rgba32FImage::new(width, height);

        for x in 0..width {
            for y in 0..height {
                let uv = glam::Vec2 {
                    x: x as f32,
                    y: y as f32,
                };

                let source_pixel = source_image.get_pixel(x, y);
                let output_pixel = output_image.get_pixel_mut(x, y);
                output_pixel.0[2] = 0.0;
                output_pixel.0[3] = 1.0;

                let mut min_distance: Option<f32> = None;

                if source_pixel.0[0] <= 127 {
                    for element in &outer {
                        let distance = (*element - uv).length();
                        match min_distance {
                            Some(_min_distance) => {
                                if distance < _min_distance {
                                    min_distance = Some(distance);
                                    output_pixel.0[0] = element.x / width as f32;
                                    output_pixel.0[1] = element.y / height as f32;
                                }
                            }
                            None => {
                                min_distance = Some(distance);
                                output_pixel.0[0] = element.x / width as f32;
                                output_pixel.0[1] = element.y / height as f32;
                            }
                        }
                    }
                } else {
                    for element in &inner {
                        let distance = (*element - uv).length();
                        match min_distance {
                            Some(_min_distance) => {
                                if distance < _min_distance {
                                    min_distance = Some(distance);
                                    output_pixel.0[0] = -element.x / width as f32;
                                    output_pixel.0[1] = -element.y / height as f32;
                                }
                            }
                            None => {
                                min_distance = Some(distance);
                                output_pixel.0[0] = -element.x / width as f32;
                                output_pixel.0[1] = -element.y / height as f32;
                            }
                        }
                    }
                }
            }
        }

        output_image
    }

    pub fn convert_to_recognizable_format(path: &Path) {
        let save_dir = path.parent().unwrap();
        let save_dir = save_dir.join("fix");
        let _ = std::fs::create_dir(save_dir.clone());
        let output_path = save_dir.join(path.file_name().unwrap());

        let output = Command::new("ffmpeg")
            .args([
                "-i",
                path.to_str().unwrap(),
                "-y",
                output_path.to_str().unwrap(),
            ])
            .output();

        match output {
            Ok(output) => {
                if output.status.success() == false {
                    log::warn!("{:?}", String::from_utf8(output.stderr).unwrap());
                }
            }
            Err(error) => {
                log::warn!("{:?}", error);
            }
        }
    }

    pub fn sdf_vis(image: &image::Rgba32FImage) -> image::RgbaImage {
        let mut output_image = image::RgbaImage::new(image.width(), image.height());
        let width = image.width();
        let height = image.height();

        for x in 0..width {
            for y in 0..height {
                let source_pixel = image.get_pixel(x, y);
                let output_pixel = output_image.get_pixel_mut(x, y);

                let uv = glam::Vec2 {
                    x: x as f32 / width as f32,
                    y: y as f32 / height as f32,
                };

                let uv2 = glam::Vec2 {
                    x: source_pixel.0[0],
                    y: source_pixel.0[1],
                }
                .abs();

                let distance = (uv - uv2).length();

                output_pixel.0[0] = (distance.clamp(0.0, 1.0) * 255.0) as u8;
                output_pixel.0[1] = (distance.clamp(0.0, 1.0) * 255.0) as u8;
                output_pixel.0[2] = (distance.clamp(0.0, 1.0) * 255.0) as u8;
                output_pixel.0[3] = 255;
            }
        }

        output_image
    }
}
