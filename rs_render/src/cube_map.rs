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
