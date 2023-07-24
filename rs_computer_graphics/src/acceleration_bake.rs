use std::sync::Arc;

use crate::{
    bake_info::BakeInfo,
    compute_pipeline::{
        brdf_lut_pipeline::BrdfLutPipeline, irradiance_cube_map::IrradianceCubeMapPipeline,
        panorama_to_cube_pipeline::PanoramaToCubePipeline,
        pre_filter_environment_cube_map_compute_pipeline::PreFilterEnvironmentCubeMapComputePipeline,
    },
    cube_map::CubeMap,
    util::{calculate_mipmap_level, texture2d_from_rgba_rgba32_fimage},
};

pub struct AccelerationBaker {
    bake_info: BakeInfo,
    equirectangular_hdr_texture: wgpu::Texture,
    pre_filter_cube_maps: Option<Vec<CubeMap<image::Rgba<f32>, Vec<f32>>>>,
    environment_cube_map: Option<CubeMap<image::Rgba<f32>, Vec<f32>>>,
    brdflut_image: Option<image::ImageBuffer<image::Rgba<f32>, Vec<f32>>>,
    irradiance_cube_map: Option<CubeMap<image::Rgba<f32>, Vec<f32>>>,
    environment_cube_texture: Arc<Option<wgpu::Texture>>,
}

impl AccelerationBaker {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        file_path: String,
        bake_info: BakeInfo,
    ) -> AccelerationBaker {
        assert!(bake_info.brdflutmap_length > 0);
        assert!(bake_info.environment_cube_map_length > 0);
        assert!(bake_info.irradiance_cube_map_length > 0);
        assert!(bake_info.pre_filter_cube_map_length > 4);
        assert!(bake_info.pre_filter_cube_map_max_mipmap_level > 0);
        match image::open(&file_path) {
            Ok(image) => {
                let max_mipmap_level = calculate_mipmap_level(bake_info.pre_filter_cube_map_length)
                    .min(bake_info.pre_filter_cube_map_max_mipmap_level);
                assert!(max_mipmap_level > 0);

                let equirectangular_hdr_image = image.into_rgba32f();
                let equirectangular_hdr_texture =
                    texture2d_from_rgba_rgba32_fimage(device, queue, &equirectangular_hdr_image, 1);
                AccelerationBaker {
                    bake_info,
                    equirectangular_hdr_texture,
                    pre_filter_cube_maps: None,
                    environment_cube_map: None,
                    brdflut_image: None,
                    irradiance_cube_map: None,
                    environment_cube_texture: Arc::new(None),
                }
            }
            Err(error) => {
                log::warn!("{:?}", error);
                panic!()
            }
        }
    }

    pub fn bake(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        if self.bake_info.is_bake_pre_filter {
            self.pre_filter_cube_maps = Some(self.bake_pre_filter_cube_maps(device, queue));
        }
        if self.bake_info.is_bake_environment {
            let (cube_map, texture) = self.bake_environment_cube_map(device, queue);
            self.environment_cube_map = Some(cube_map);
            self.environment_cube_texture = Arc::new(Some(texture));
        }
        if self.bake_info.is_bake_brdflut {
            self.brdflut_image = Some(self.bake_brdflut_image(device, queue));
        }
        if self.bake_info.is_bake_irradiance {
            self.irradiance_cube_map = Some(self.bake_irradiance_cube_map(device, queue));
        }
    }

    fn bake_irradiance_cube_map(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> CubeMap<image::Rgba<f32>, Vec<f32>> {
        let irradiance_cube_map_pipeline = IrradianceCubeMapPipeline::new(device);
        let images = irradiance_cube_map_pipeline.execute(
            device,
            queue,
            &self.equirectangular_hdr_texture,
            self.bake_info.irradiance_cube_map_length,
            self.bake_info.irradiance_sample_count,
        );
        let cube_map = CubeMap {
            negative_x: images[0].to_owned(),
            positive_x: images[1].to_owned(),
            negative_y: images[2].to_owned(),
            positive_y: images[3].to_owned(),
            negative_z: images[4].to_owned(),
            positive_z: images[5].to_owned(),
        };
        cube_map
    }

    fn bake_brdflut_image(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> image::ImageBuffer<image::Rgba<f32>, Vec<f32>> {
        let brdf_lut_pipeline = BrdfLutPipeline::new(device);
        brdf_lut_pipeline.execute(
            device,
            queue,
            self.bake_info.brdflutmap_length,
            self.bake_info.brdf_sample_count,
        )
    }

    fn bake_environment_cube_map(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> (CubeMap<image::Rgba<f32>, Vec<f32>>, wgpu::Texture) {
        let panorama_to_cube_pipeline = PanoramaToCubePipeline::new(device);
        let texture = panorama_to_cube_pipeline.execute(
            device,
            queue,
            &self.equirectangular_hdr_texture,
            self.bake_info.environment_cube_map_length,
        );

        let image_datas = crate::util::map_texture_cube_cpu_sync(
            device,
            queue,
            &texture,
            self.bake_info.environment_cube_map_length,
            self.bake_info.environment_cube_map_length,
            image::ColorType::Rgba32F,
        );
        let mut images: Vec<image::ImageBuffer<image::Rgba<f32>, Vec<f32>>> = vec![];
        for image_data in &image_datas {
            let f32_data: &[f32] = crate::util::cast_to_type_buffer(image_data);
            let imgae = image::Rgba32FImage::from_vec(
                self.bake_info.environment_cube_map_length,
                self.bake_info.environment_cube_map_length,
                f32_data.to_vec(),
            )
            .unwrap();
            images.push(imgae);
        }

        let cube_map = CubeMap {
            negative_x: images[0].to_owned(),
            positive_x: images[1].to_owned(),
            negative_y: images[2].to_owned(),
            positive_y: images[3].to_owned(),
            negative_z: images[4].to_owned(),
            positive_z: images[5].to_owned(),
        };
        (cube_map, texture)
    }

    fn bake_pre_filter_cube_maps(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Vec<CubeMap<image::Rgba<f32>, Vec<f32>>> {
        let pre_filter_environment_cube_map_compute_pipeline =
            PreFilterEnvironmentCubeMapComputePipeline::new(device);

        let max_mipmap_level = calculate_mipmap_level(self.bake_info.pre_filter_cube_map_length)
            .min(self.bake_info.pre_filter_cube_map_max_mipmap_level);
        assert!(max_mipmap_level > 0);
        let roughness_delta: f32;
        if max_mipmap_level == 1 {
            roughness_delta = 0.0;
        } else {
            roughness_delta = 1.0_f32 / (max_mipmap_level as f32 - 1.0);
        }
        let mut cube_maps: Vec<CubeMap<image::Rgba<f32>, Vec<f32>>> = vec![];

        for mipmap_level in 0..max_mipmap_level {
            let length = self.bake_info.pre_filter_cube_map_length / (1 << mipmap_level);
            let sample_count = self.bake_info.pre_filter_sample_count;
            let roughness = roughness_delta * mipmap_level as f32;

            let images = pre_filter_environment_cube_map_compute_pipeline.execute(
                device,
                queue,
                &self.equirectangular_hdr_texture,
                length,
                roughness,
                sample_count,
            );
            let cube_map = CubeMap {
                negative_x: images[0].to_owned(),
                positive_x: images[1].to_owned(),
                negative_y: images[2].to_owned(),
                positive_y: images[3].to_owned(),
                negative_z: images[4].to_owned(),
                positive_z: images[5].to_owned(),
            };
            cube_maps.push(cube_map);
        }
        cube_maps
    }

    pub fn save_to_disk_sync(&self, dir: &str) {
        let dir_path = std::path::Path::new(dir);
        if let Some(cube_maps) = &self.pre_filter_cube_maps {
            for (index, cube_map) in cube_maps.iter().enumerate() {
                AccelerationBaker::save_cube_map(
                    &dir_path.join(format!("pre_filter_cube_map_{}", index)),
                    cube_map,
                );
            }
        }
        if let Some(cube_map) = &self.environment_cube_map {
            AccelerationBaker::save_cube_map(&dir_path.join("environment_cube_map"), cube_map);
        }
        if let Some(brdflut_image) = &self.brdflut_image {
            let path = dir_path.join("brdflut.exr");
            Self::save_image_to_disk(brdflut_image, path);
        }
        if let Some(cube_map) = &self.irradiance_cube_map {
            AccelerationBaker::save_cube_map(&dir_path.join("irradiance_cube_map"), cube_map);
        }
    }

    fn save_cube_map(dir_path: &std::path::Path, cube_map: &CubeMap<image::Rgba<f32>, Vec<f32>>) {
        {
            let path = dir_path.join("negative_x.exr");
            Self::save_image_to_disk(&cube_map.negative_x, path);
        }
        {
            let path = dir_path.join("positive_x.exr");
            Self::save_image_to_disk(&cube_map.positive_x, path);
        }
        {
            let path = dir_path.join("negative_y.exr");
            Self::save_image_to_disk(&cube_map.negative_y, path);
        }
        {
            let path = dir_path.join("positive_y.exr");
            Self::save_image_to_disk(&cube_map.positive_y, path);
        }
        {
            let path = dir_path.join("negative_z.exr");
            Self::save_image_to_disk(&cube_map.negative_z, path);
        }
        {
            let path = dir_path.join("positive_z.exr");
            Self::save_image_to_disk(&cube_map.positive_z, path);
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
                    // let image = image::DynamicImage::ImageRgba32F(image.clone()).to_rgba8();
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

    pub fn get_environment_cube_texture(&self) -> Arc<Option<wgpu::Texture>> {
        self.environment_cube_texture.clone()
    }
}
