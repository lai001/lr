use crate::util::sample_equirectangular_map;

pub struct CubeMap<P: image::Pixel, Container> {
    pub negative_x: image::ImageBuffer<P, Container>,
    pub positive_x: image::ImageBuffer<P, Container>,
    pub negative_y: image::ImageBuffer<P, Container>,
    pub positive_y: image::ImageBuffer<P, Container>,
    pub negative_z: image::ImageBuffer<P, Container>,
    pub positive_z: image::ImageBuffer<P, Container>,
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
