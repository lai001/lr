use crate::{
    thread_pool,
    util::{
        convert_coordinate_system, geometry_smith, hammersley_2d, hemisphere_sample_uniform,
        importance_sample_ggx, reflect_vec3, sample_equirectangular_map,
    },
};
use glam::Vec3Swizzles;
use image::GenericImageView;

pub struct CubeMap<P: image::Pixel, Container> {
    negative_x: image::ImageBuffer<P, Container>,
    positive_x: image::ImageBuffer<P, Container>,
    negative_y: image::ImageBuffer<P, Container>,
    positive_y: image::ImageBuffer<P, Container>,
    negative_z: image::ImageBuffer<P, Container>,
    positive_z: image::ImageBuffer<P, Container>,
}

impl<P, Container> CubeMap<P, Container>
where
    P: image::Pixel,
    Container: std::ops::Deref<Target = [P::Subpixel]>,
{
    pub fn to_mut_array(&mut self) -> Vec<&mut image::ImageBuffer<P, Container>> {
        let images = vec![
            &mut self.positive_x,
            &mut self.negative_x,
            &mut self.positive_y,
            &mut self.negative_y,
            &mut self.positive_z,
            &mut self.negative_z,
        ];
        images
    }

    pub fn to_array(&self) -> Vec<&image::ImageBuffer<P, Container>> {
        let images = vec![
            &self.positive_x,
            &self.negative_x,
            &self.positive_y,
            &self.negative_y,
            &self.positive_z,
            &self.negative_z,
        ];
        images
    }

    pub fn sample(&self, location: glam::Vec3) -> &P {
        let location_abs = location.abs();
        let mag = location_abs.max_element();
        if mag == location_abs.x {
            if location.x > 0.0 {
                let x = 1.0 - (location.z + 1.0) / 2.0;
                let y = (location.y + 1.0) / 2.0;
                let x = x * (self.positive_x.width() - 1) as f32;
                let y = y * (self.positive_x.height() - 1) as f32;
                let pixel = (&self.positive_x).get_pixel(x as u32, y as u32);
                return pixel;
            } else if location.x < 0.0 {
                let x = (location.z + 1.0) / 2.0;
                let y = (location.y + 1.0) / 2.0;
                let x = x * (self.positive_x.width() - 1) as f32;
                let y = y * (self.positive_x.height() - 1) as f32;
                let pixel = (&self.negative_x).get_pixel(x as u32, y as u32);
                return pixel;
            }
        } else if mag == location_abs.y {
            if location.y > 0.0 {
                let x = (location.x + 1.0) / 2.0;
                let y = 1.0 - (location.z + 1.0) / 2.0;
                let x = x * (self.positive_x.width() - 1) as f32;
                let y = y * (self.positive_x.height() - 1) as f32;
                let pixel = (&self.positive_y).get_pixel(x as u32, y as u32);
                return pixel;
            } else if location.y < 0.0 {
                let x = (location.x + 1.0) / 2.0;
                let y = (location.z + 1.0) / 2.0;
                let x = x * (self.positive_x.width() - 1) as f32;
                let y = y * (self.positive_x.height() - 1) as f32;
                let pixel = (&self.negative_y).get_pixel(x as u32, y as u32);
                return pixel;
            }
        } else if mag == location_abs.z {
            if location.z > 0.0 {
                let x = (location.x + 1.0) / 2.0;
                let y = (location.y + 1.0) / 2.0;
                let x = x * (self.positive_x.width() - 1) as f32;
                let y = y * (self.positive_x.height() - 1) as f32;
                let pixel = (&self.positive_z).get_pixel(x as u32, y as u32);
                return pixel;
            } else if location.z < 0.0 {
                let x = 1.0 - (location.x + 1.0) / 2.0;
                let y = (location.y + 1.0) / 2.0;
                let x = x * (self.positive_x.width() - 1) as f32;
                let y = y * (self.positive_x.height() - 1) as f32;
                let pixel = (&self.negative_z).get_pixel(x as u32, y as u32);
                return pixel;
            }
        }
        panic!()
    }
}

pub fn cube_map_to_equirectangular(
    cube_map: &CubeMap<image::Rgba<f32>, Vec<f32>>,
    target_width: u32,
    target_height: u32,
) -> image::ImageBuffer<image::Rgba<f32>, Vec<f32>> {
    let mut equirectangular =
        image::ImageBuffer::<image::Rgba<f32>, Vec<f32>>::new(target_width, target_height);
    let images = vec![
        &cube_map.positive_x,
        &cube_map.negative_x,
        &cube_map.positive_y,
        &cube_map.negative_y,
        &cube_map.positive_z,
        &cube_map.negative_z,
    ];
    let length = cube_map.positive_x.width();
    for (index, image) in images.iter().enumerate() {
        for height_idx in 0..length {
            for width_idx in 0..length {
                let uv = glam::vec2(
                    width_idx as f32 / length as f32,
                    height_idx as f32 / length as f32,
                ) * 2.0_f32
                    - 1.0_f32;
                let sample_picker: glam::Vec3;
                if index == 0 {
                    sample_picker = glam::vec3(1.0, uv.y, -uv.x);
                } else if index == 1 {
                    sample_picker = glam::vec3(-1.0, uv.y, uv.x);
                } else if index == 2 {
                    sample_picker = glam::vec3(uv.x, 1.0, -uv.y);
                } else if index == 3 {
                    sample_picker = glam::vec3(uv.x, -1.0, uv.y);
                } else if index == 4 {
                    sample_picker = glam::vec3(uv.x, uv.y, 1.0);
                } else if index == 5 {
                    sample_picker = glam::vec3(-uv.x, uv.y, -1.0);
                } else {
                    panic!()
                }
                let sample_picker = sample_picker.normalize();
                // let color = cube_map.sample(sample_picker);
                let color = image.get_pixel(width_idx, height_idx);
                let equirectangular_tex_coord = sample_equirectangular_map(sample_picker);
                let equirectangular_tex_coord = glam::uvec2(
                    (equirectangular_tex_coord.x * (target_width as f32 - 1.0)) as u32,
                    (equirectangular_tex_coord.y * (target_height as f32 - 1.0)) as u32,
                );
                let target_pixel = equirectangular
                    .get_pixel_mut(equirectangular_tex_coord.x, equirectangular_tex_coord.y);
                target_pixel.0.copy_from_slice(color.0.as_slice());
            }
        }
    }
    equirectangular
}

pub fn sample_equirectangular(
    image: &image::ImageBuffer<image::Rgba<f32>, Vec<f32>>,
    location: glam::Vec3,
) -> &image::Rgba<f32> {
    let sample_picker = sample_equirectangular_map(location);
    let sample_picker = glam::vec2(
        sample_picker.x * (image.width() - 1) as f32,
        sample_picker.y * (image.height() - 1) as f32,
    );
    let source_pixel = image.get_pixel(sample_picker.x as u32, sample_picker.y as u32);
    return source_pixel;
}

pub struct BakeInfo {
    pub is_bake_environment: bool,
    pub is_bake_irradiance: bool,
    pub is_bake_brdflut: bool,
    pub is_bake_pre_filter: bool,
    pub environment_cube_map_length: u32,
    pub irradiance_cube_map_length: u32,
    pub irradiance_sample_count: u32,
    pub pre_filter_cube_map_length: u32,
    pub pre_filter_cube_map_max_mipmap_level: u32,
    pub pre_filter_sample_count: u32,
    pub brdflutmap_length: u32,
    pub brdf_sample_count: u32,
}

pub struct Baker {
    file_path: String,
    bake_info: BakeInfo,
    equirectangular_hdr_image: image::ImageBuffer<image::Rgba<f32>, Vec<f32>>,
    environment_cube_map: Option<CubeMap<image::Rgba<f32>, Vec<f32>>>,
    irradiance_cube_map: Option<CubeMap<image::Rgba<f32>, Vec<f32>>>,
    pre_filter_cube_maps: Option<Vec<CubeMap<image::Rgba<f32>, Vec<f32>>>>,
    brdflut_image: Option<image::ImageBuffer<image::Rgba<f32>, Vec<f32>>>,
}

impl Baker {
    pub fn new(file_path: String, bake_info: BakeInfo) -> Baker {
        assert!(bake_info.brdflutmap_length > 0);
        assert!(bake_info.environment_cube_map_length > 0);
        assert!(bake_info.irradiance_cube_map_length > 0);
        assert!(bake_info.pre_filter_cube_map_length > 4);
        assert!(bake_info.pre_filter_cube_map_max_mipmap_level > 0);
        if let Ok(image) = image::open(&file_path) {
            let equirectangular_hdr_image = image.into_rgba32f();
            Baker {
                file_path,
                bake_info,
                environment_cube_map: None,
                equirectangular_hdr_image,
                brdflut_image: None,
                irradiance_cube_map: None,
                pre_filter_cube_maps: None,
            }
        } else {
            panic!()
        }
    }

    pub fn bake(&mut self) {
        if self.bake_info.is_bake_environment {
            self.environment_cube_map = Some(self.bake_environment_cube_map());
        }
        if self.bake_info.is_bake_brdflut {
            self.brdflut_image = Some(self.bake_brdflut_image());
        }
        if self.bake_info.is_bake_irradiance {
            self.irradiance_cube_map = Some(self.bake_irradiance_cube_map());
        }
        if self.bake_info.is_bake_pre_filter {
            self.pre_filter_cube_maps = Some(self.bake_pre_filter_cube_maps());
        }
    }

    fn calculate_mipmap_level(&mut self, length: u32) -> u32 {
        let mut mipmap_level: u32 = 1;
        let mut length = length;
        while length > 4 {
            length /= 2;
            mipmap_level += 1;
        }
        return mipmap_level;
    }

    fn bake_pre_filter_cube_maps(&mut self) -> Vec<CubeMap<image::Rgba<f32>, Vec<f32>>> {
        let max_mipmap_level = self
            .calculate_mipmap_level(self.bake_info.pre_filter_cube_map_length)
            .min(self.bake_info.pre_filter_cube_map_max_mipmap_level);
        assert!(max_mipmap_level > 0);
        let roughness_delta: f32;
        if max_mipmap_level == 1 {
            roughness_delta = 0.0;
        } else {
            roughness_delta = 1.0_f32 / (max_mipmap_level as f32 - 1.0);
        }

        let mut cube_maps: Vec<CubeMap<image::Rgba<f32>, Vec<f32>>> = vec![];

        let (tx, rx) = std::sync::mpsc::channel();
        for mipmap_level in 0..max_mipmap_level {
            let tx = tx.clone();
            let length = self.bake_info.pre_filter_cube_map_length / (1 << mipmap_level);
            let sample_count = self.bake_info.pre_filter_sample_count;
            let equirectangular_hdr_image = self.equirectangular_hdr_image.clone();
            let roughness = roughness_delta * mipmap_level as f32;

            thread_pool::ThreadPool::global()
                .lock()
                .unwrap()
                .spawn(move || {
                    let negative_x =
                        image::ImageBuffer::<image::Rgba<f32>, Vec<f32>>::new(length, length);
                    let positive_x =
                        image::ImageBuffer::<image::Rgba<f32>, Vec<f32>>::new(length, length);
                    let negative_y =
                        image::ImageBuffer::<image::Rgba<f32>, Vec<f32>>::new(length, length);
                    let positive_y =
                        image::ImageBuffer::<image::Rgba<f32>, Vec<f32>>::new(length, length);
                    let negative_z =
                        image::ImageBuffer::<image::Rgba<f32>, Vec<f32>>::new(length, length);
                    let positive_z =
                        image::ImageBuffer::<image::Rgba<f32>, Vec<f32>>::new(length, length);

                    let mut cube_map = CubeMap {
                        negative_x: negative_x,
                        positive_x: positive_x,
                        negative_y: negative_y,
                        positive_y: positive_y,
                        negative_z: negative_z,
                        positive_z: positive_z,
                    };

                    for index in 0..cube_map.to_mut_array().len() {
                        for height_idx in 0..length {
                            for width_idx in 0..length {
                                let uv = glam::vec2(
                                    width_idx as f32 / length as f32,
                                    height_idx as f32 / length as f32,
                                ) * 2.0_f32
                                    - 1.0_f32;
                                let mut sample_picker: glam::Vec3;
                                if index == 0 {
                                    sample_picker = glam::vec3(1.0, uv.y, -uv.x);
                                } else if index == 1 {
                                    sample_picker = glam::vec3(-1.0, uv.y, uv.x);
                                } else if index == 2 {
                                    sample_picker = glam::vec3(uv.x, 1.0, -uv.y);
                                } else if index == 3 {
                                    sample_picker = glam::vec3(uv.x, -1.0, uv.y);
                                } else if index == 4 {
                                    sample_picker = glam::vec3(uv.x, uv.y, 1.0);
                                } else if index == 5 {
                                    sample_picker = glam::vec3(-uv.x, uv.y, -1.0);
                                } else {
                                    panic!()
                                }
                                sample_picker = sample_picker.normalize();
                                let mut total_weight = 0.0_f32;
                                let mut prefiltered_color = glam::Vec3::ZERO;

                                for i in 0..sample_count {
                                    let xi = hammersley_2d(i, sample_count);
                                    let g = importance_sample_ggx(xi, roughness);
                                    let up_vector = glam::vec3(0.0, 1.0, 0.0);
                                    let tangent_vector = sample_picker.cross(up_vector).normalize();
                                    let bitangent_vector =
                                        sample_picker.cross(tangent_vector).normalize();
                                    let h = convert_coordinate_system(
                                        g,
                                        tangent_vector,
                                        bitangent_vector,
                                        sample_picker,
                                    );
                                    let l = reflect_vec3(-h, sample_picker);
                                    let n_dot_l = sample_picker.dot(l);
                                    if n_dot_l > 0.0 {
                                        let color =
                                            sample_equirectangular(&equirectangular_hdr_image, l);
                                        let color = glam::Vec3::from_slice(color.0.as_slice());
                                        prefiltered_color = prefiltered_color + (color * n_dot_l);
                                        total_weight = total_weight + n_dot_l;
                                    }
                                }
                                prefiltered_color = prefiltered_color / total_weight;

                                let source_pixel = prefiltered_color;
                                let mut source_pixel = source_pixel.xyzx();
                                source_pixel.w = 1.0;
                                if !source_pixel.is_nan() {
                                    let mut cube_map = cube_map.to_mut_array();
                                    let target_pixel =
                                        cube_map[index].get_pixel_mut(width_idx, height_idx);
                                    target_pixel.0 = source_pixel.to_array();
                                }
                            }
                        }
                    }
                    tx.send(cube_map).unwrap();
                });
        }

        drop(tx);
        for cube_map in rx {
            cube_maps.push(cube_map);
        }
        cube_maps.sort_by(|left, right| {
            if right.negative_x.width() > left.negative_x.width() {
                std::cmp::Ordering::Greater
            } else if right.negative_x.width() == left.negative_x.width() {
                std::cmp::Ordering::Equal
            } else {
                std::cmp::Ordering::Less
            }
        });
        cube_maps
    }

    fn bake_irradiance_cube_map(&mut self) -> CubeMap<image::Rgba<f32>, Vec<f32>> {
        let length = self.bake_info.irradiance_cube_map_length;
        let mut negative_x = image::ImageBuffer::<image::Rgba<f32>, Vec<f32>>::new(length, length);
        let mut positive_x = image::ImageBuffer::<image::Rgba<f32>, Vec<f32>>::new(length, length);
        let mut negative_y = image::ImageBuffer::<image::Rgba<f32>, Vec<f32>>::new(length, length);
        let mut positive_y = image::ImageBuffer::<image::Rgba<f32>, Vec<f32>>::new(length, length);
        let mut negative_z = image::ImageBuffer::<image::Rgba<f32>, Vec<f32>>::new(length, length);
        let mut positive_z = image::ImageBuffer::<image::Rgba<f32>, Vec<f32>>::new(length, length);

        let mut images = vec![
            &mut positive_x,
            &mut negative_x,
            &mut positive_y,
            &mut negative_y,
            &mut positive_z,
            &mut negative_z,
        ];
        let total = (images.len() * length as usize) as f32;

        for (index, image) in images.iter_mut().enumerate() {
            for height_idx in 0..length {
                for width_idx in 0..length {
                    let uv = glam::vec2(
                        width_idx as f32 / length as f32,
                        height_idx as f32 / length as f32,
                    ) * 2.0_f32
                        - 1.0_f32;
                    let mut sample_picker: glam::Vec3;
                    if index == 0 {
                        sample_picker = glam::vec3(1.0, uv.y, -uv.x);
                    } else if index == 1 {
                        sample_picker = glam::vec3(-1.0, uv.y, uv.x);
                    } else if index == 2 {
                        sample_picker = glam::vec3(uv.x, 1.0, -uv.y);
                    } else if index == 3 {
                        sample_picker = glam::vec3(uv.x, -1.0, uv.y);
                    } else if index == 4 {
                        sample_picker = glam::vec3(uv.x, uv.y, 1.0);
                    } else if index == 5 {
                        sample_picker = glam::vec3(-uv.x, uv.y, -1.0);
                    } else {
                        panic!()
                    }
                    sample_picker = sample_picker.normalize();
                    let up_vector = glam::vec3(0.0, 1.0, 0.0);
                    let tangent_vector = sample_picker.cross(up_vector).normalize();
                    let bitangent_vector = sample_picker.cross(tangent_vector).normalize();
                    let mut irradiance = glam::Vec3::ZERO;

                    let sample_count = self.bake_info.irradiance_sample_count;
                    for sample_index in 0..sample_count {
                        let h = hammersley_2d(sample_index, sample_count);
                        let r = hemisphere_sample_uniform(h.x, h.y);
                        let l = convert_coordinate_system(
                            r,
                            bitangent_vector,
                            tangent_vector,
                            sample_picker,
                        );
                        let source_sample_picker = sample_equirectangular_map(l);
                        let source_sample_picker = glam::vec2(
                            source_sample_picker.x
                                * (self.equirectangular_hdr_image.width() - 1) as f32,
                            source_sample_picker.y
                                * (self.equirectangular_hdr_image.height() - 1) as f32,
                        );
                        let source_sample_picker = glam::uvec2(
                            source_sample_picker.x as u32,
                            source_sample_picker.y as u32,
                        );
                        let source_pixel = self
                            .equirectangular_hdr_image
                            .get_pixel(source_sample_picker.x, source_sample_picker.y);
                        let source_pixel = source_pixel.0.as_slice();
                        let add = 2.0
                            * glam::vec3(source_pixel[0], source_pixel[1], source_pixel[2])
                            * l.dot(sample_picker).max(0.0);
                        irradiance = irradiance + add;
                    }
                    irradiance = irradiance / sample_count as f32;
                    irradiance = irradiance.clamp(glam::Vec3::ZERO, glam::Vec3::ONE);
                    let target_pixel = image.get_pixel_mut(width_idx, height_idx);
                    let source_pixel = irradiance;
                    let mut source_pixel = source_pixel.xyzx();
                    source_pixel.w = 1.0;
                    target_pixel.0 = source_pixel.to_array();
                }
                log::trace!(
                    "Bake irradiance progress: {}",
                    (index * length as usize + height_idx as usize) as f32 / total
                );
            }
        }

        CubeMap {
            negative_x: negative_x,
            positive_x: positive_x,
            negative_y: negative_y,
            positive_y: positive_y,
            negative_z: negative_z,
            positive_z: positive_z,
        }
    }

    fn bake_brdflut_image(&mut self) -> image::ImageBuffer<image::Rgba<f32>, Vec<f32>> {
        let integrate_brdf = |n_dot_v: f32, roughness: f32| {
            let v = glam::vec3((1.0 - n_dot_v * n_dot_v).sqrt(), 0.0, n_dot_v);

            let mut a: f32 = 0.0;
            let mut b: f32 = 0.0;

            let n = glam::vec3(0.0, 0.0, 1.0);
            let tangent_vector = glam::vec3(1.0, 0.0, 0.0).cross(n).normalize();
            let bitangent_vector = n.cross(tangent_vector).normalize();

            let sample_count = self.bake_info.brdf_sample_count;
            for i in 0..sample_count {
                let xi = hammersley_2d(i, sample_count);
                let h = importance_sample_ggx(xi, roughness);
                let h = convert_coordinate_system(h, tangent_vector, bitangent_vector, n);

                let l = reflect_vec3(-v, h);

                let n_dot_l = l.z.max(0.0);
                let n_dot_h = h.z.max(0.0);
                let v_dot_h = v.dot(h).max(0.0);

                if n_dot_l > 0.0 {
                    let g = geometry_smith(n, v, l, roughness);
                    let g_vis = (g * v_dot_h) / (n_dot_h * n_dot_v);
                    let fc = (1.0 - v_dot_h).powf(5.0);
                    a += (1.0 - fc) * g_vis;
                    b += fc * g_vis;
                }
            }
            a /= sample_count as f32;
            b /= sample_count as f32;
            let mut result = glam::vec2(a, b);
            if result.is_nan() {
                result = glam::Vec2::ZERO;
            }
            result
        };

        let mut brdflut_image = image::ImageBuffer::<image::Rgba<f32>, Vec<f32>>::new(
            self.bake_info.brdflutmap_length,
            self.bake_info.brdflutmap_length,
        );

        for height_idx in 0..self.bake_info.brdflutmap_length {
            for width_idx in 0..self.bake_info.brdflutmap_length {
                let texcoord = glam::vec2(
                    width_idx as f32 / self.bake_info.brdflutmap_length as f32,
                    height_idx as f32 / self.bake_info.brdflutmap_length as f32,
                );
                let pixel_color = integrate_brdf(texcoord.x, texcoord.y);
                let pixel_color = glam::vec4(pixel_color.x, pixel_color.y, 0.0, 1.0);
                let mut color = brdflut_image.get_pixel_mut(width_idx, height_idx);
                color.0 = pixel_color.to_array();
            }
            log::trace!(
                "Bake brdf progress: {}",
                height_idx as f32 / self.bake_info.brdflutmap_length as f32
            );
        }

        brdflut_image
    }

    fn bake_environment_cube_map(&mut self) -> CubeMap<image::Rgba<f32>, Vec<f32>> {
        let length = self.bake_info.environment_cube_map_length;
        let mut negative_x = image::ImageBuffer::<image::Rgba<f32>, Vec<f32>>::new(length, length);
        let mut positive_x = image::ImageBuffer::<image::Rgba<f32>, Vec<f32>>::new(length, length);
        let mut negative_y = image::ImageBuffer::<image::Rgba<f32>, Vec<f32>>::new(length, length);
        let mut positive_y = image::ImageBuffer::<image::Rgba<f32>, Vec<f32>>::new(length, length);
        let mut negative_z = image::ImageBuffer::<image::Rgba<f32>, Vec<f32>>::new(length, length);
        let mut positive_z = image::ImageBuffer::<image::Rgba<f32>, Vec<f32>>::new(length, length);

        let mut images = vec![
            &mut positive_x,
            &mut negative_x,
            &mut positive_y,
            &mut negative_y,
            &mut positive_z,
            &mut negative_z,
        ];

        for (index, image) in images.iter_mut().enumerate() {
            for height_idx in 0..length {
                for width_idx in 0..length {
                    let uv = glam::vec2(
                        width_idx as f32 / length as f32,
                        height_idx as f32 / length as f32,
                    ) * 2.0_f32
                        - 1.0_f32;
                    let mut sample_picker: glam::Vec3;
                    if index == 0 {
                        sample_picker = glam::vec3(1.0, uv.y, -uv.x);
                    } else if index == 1 {
                        sample_picker = glam::vec3(-1.0, uv.y, uv.x);
                    } else if index == 2 {
                        sample_picker = glam::vec3(uv.x, 1.0, -uv.y);
                    } else if index == 3 {
                        sample_picker = glam::vec3(uv.x, -1.0, uv.y);
                    } else if index == 4 {
                        sample_picker = glam::vec3(uv.x, uv.y, 1.0);
                    } else if index == 5 {
                        sample_picker = glam::vec3(-uv.x, uv.y, -1.0);
                    } else {
                        panic!()
                    }
                    sample_picker = sample_picker.normalize();

                    let sample_picker = sample_equirectangular_map(sample_picker);
                    let sample_picker = glam::vec2(
                        sample_picker.x * (self.equirectangular_hdr_image.width() - 1) as f32,
                        sample_picker.y * (self.equirectangular_hdr_image.height() - 1) as f32,
                    );
                    let mut target_pixel = image.get_pixel_mut(width_idx, height_idx);
                    let source_pixel = self
                        .equirectangular_hdr_image
                        .get_pixel(sample_picker.x as u32, sample_picker.y as u32);
                    target_pixel.0 = source_pixel.0;
                }
            }
        }

        CubeMap {
            negative_x: negative_x,
            positive_x: positive_x,
            negative_y: negative_y,
            positive_y: positive_y,
            negative_z: negative_z,
            positive_z: positive_z,
        }
    }

    fn save_image_to_disk(
        image: &image::ImageBuffer<image::Rgba<f32>, Vec<f32>>,
        path: std::path::PathBuf,
    ) {
        let dir_path = path.parent();
        match dir_path {
            Some(dir_path) => match std::fs::create_dir_all(dir_path) {
                Ok(_) => {
                    let rgba32f_image = image::DynamicImage::ImageRgba32F(image.clone());
                    let image = rgba32f_image.to_rgba8();
                    let result = image.save(path.clone());
                    match result {
                        Ok(_) => log::trace!("Save to {}", path.to_str().unwrap()),
                        Err(error) => log::warn!("{}", error),
                    }
                }
                Err(error) => log::warn!("{}", error),
            },
            None => panic!(),
        }
    }

    fn save_cube_map(
        &self,
        dir_path: &std::path::Path,
        cube_map: &CubeMap<image::Rgba<f32>, Vec<f32>>,
    ) {
        {
            let path = dir_path.join("negative_x.png");
            Self::save_image_to_disk(&cube_map.negative_x, path);
        }
        {
            let path = dir_path.join("positive_x.png");
            Self::save_image_to_disk(&cube_map.positive_x, path);
        }
        {
            let path = dir_path.join("negative_y.png");
            Self::save_image_to_disk(&cube_map.negative_y, path);
        }
        {
            let path = dir_path.join("positive_y.png");
            Self::save_image_to_disk(&cube_map.positive_y, path);
        }
        {
            let path = dir_path.join("negative_z.png");
            Self::save_image_to_disk(&cube_map.negative_z, path);
        }
        {
            let path = dir_path.join("positive_z.png");
            Self::save_image_to_disk(&cube_map.positive_z, path);
        }
    }

    pub fn save_to_disk_sync(&self, dir: &str) {
        let dir_path = std::path::Path::new(dir);
        if let Some(cube_map) = &self.environment_cube_map {
            self.save_cube_map(&dir_path.join("environment_cube_map"), cube_map);
        }
        if let Some(brdflut_image) = &self.brdflut_image {
            let path = dir_path.join("brdflut.png");
            Self::save_image_to_disk(brdflut_image, path);
        }
        if let Some(cube_map) = &self.irradiance_cube_map {
            self.save_cube_map(&dir_path.join("irradiance_cube_map"), cube_map);
        }
        if let Some(cube_maps) = &self.pre_filter_cube_maps {
            for (index, cube_map) in cube_maps.iter().enumerate() {
                self.save_cube_map(
                    &dir_path.join(format!("pre_filter_cube_map_{}", index)),
                    cube_map,
                );
            }
        }
    }

    pub fn get_environment_cube_map(&self) -> Option<&CubeMap<image::Rgba<f32>, Vec<f32>>> {
        self.environment_cube_map.as_ref()
    }
}
