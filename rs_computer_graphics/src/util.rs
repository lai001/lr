use crate::{actor::Actor, camera::Camera, yuv420p_image::YUV420pImage};
use core::ffi;
use glam::{Vec3Swizzles, Vec4Swizzles};
use std::io::Write;

#[derive(Clone, Debug)]
pub struct RayHitTestResult {
    pub mesh_name: String,
    pub intersection_point: glam::Vec3,
}

#[repr(C)]
#[derive(Clone, Default, PartialEq, Eq, Hash, Debug)]
pub struct Range<T: Copy> {
    pub start: T,
    pub end: T,
}

impl<T> Range<T>
where
    T: Copy,
{
    pub fn to_std_range(&self) -> std::ops::Range<T> {
        std::ops::Range::<T> {
            start: self.start,
            end: self.end,
        }
    }
}

pub fn ffi_to_rs_string(c_str: *const std::ffi::c_char) -> Option<String> {
    if c_str.is_null() {
        None
    } else {
        let rs_string = unsafe { ffi::CStr::from_ptr(c_str).to_str().unwrap().to_owned() };
        Some(rs_string)
    }
}

pub fn math_remap_value_range(
    value: f64,
    from_range: std::ops::Range<f64>,
    to_range: std::ops::Range<f64>,
) -> f64 {
    (value - from_range.start) / (from_range.end - from_range.start)
        * (to_range.end - to_range.start)
        + to_range.start
}

pub fn screent_space_to_world_space(
    point: glam::Vec3,
    model: glam::Mat4,
    view: glam::Mat4,
    projection: glam::Mat4,
) -> glam::Vec3 {
    let point = glam::Vec4::new(point.x, point.y, point.z, 1.0);
    let mvp = projection * view * model;
    let inv_vp = mvp.inverse();
    let point_at_world_space = inv_vp * point;
    let point_at_world_space = point_at_world_space / point_at_world_space.w;
    point_at_world_space.xyz()
}

pub fn triangle_plane_normal_vector(a: glam::Vec3, b: glam::Vec3, c: glam::Vec3) -> glam::Vec3 {
    let u = b - a;
    let v = c - a;
    u.cross(v)
}

pub fn is_same_side(a: glam::Vec3, b: glam::Vec3, c: glam::Vec3, p: glam::Vec3) -> bool {
    let ab = b - a;
    let ac = c - a;
    let ap = p - a;
    let v1 = ab.cross(ac);
    let v2 = ab.cross(ap);
    let result = v1.dot(v2) > 0.0;
    result
}

pub fn is_point_in_triangle(a: glam::Vec3, b: glam::Vec3, c: glam::Vec3, p: glam::Vec3) -> bool {
    is_same_side(a, b, c, p) && is_same_side(b, c, a, p) && is_same_side(c, a, b, p)
}

pub fn triangle_plane_ray_intersection(
    a: glam::Vec3,
    b: glam::Vec3,
    c: glam::Vec3,
    origin: glam::Vec3,
    direction: glam::Vec3,
) -> Option<glam::Vec3> {
    let direction = direction.normalize();
    let normal_vector = triangle_plane_normal_vector(a, b, c).normalize();
    let t = (a - origin).dot(normal_vector) / (direction.dot(normal_vector));

    let target_location = origin + direction * t;
    let is_point_in_triangle = is_point_in_triangle(a, b, c, target_location);
    if is_point_in_triangle {
        Some(target_location)
    } else {
        None
    }
}

pub fn shape(
    a: glam::Vec3,
    b: glam::Vec3,
    c: glam::Vec3,
    d: glam::Vec3,
) -> Vec<(glam::Vec3, glam::Vec3, glam::Vec3)> {
    let mut array: Vec<(glam::Vec3, glam::Vec3, glam::Vec3)> = vec![];
    array.push((a, b, c));
    array.push((a, c, d));
    array
}

pub fn init_log() {
    let log_env =
        env_logger::Env::default().default_filter_or("rs_computer_graphics,rs_dotnet,rs_media");
    env_logger::Builder::from_env(log_env)
        .format(|buf, record| {
            let level = record.level();
            let level_style = buf.default_level_style(level);
            writeln!(
                buf,
                "[{}] {}:{} {} {}",
                level_style.value(level),
                record.file().unwrap_or("Unknown"),
                record.line().unwrap_or(0),
                buf.timestamp_millis(),
                record.args()
            )
        })
        .init();
}

pub fn change_working_directory() {
    if let (Ok(current_dir), Ok(current_exe)) = (std::env::current_dir(), std::env::current_exe()) {
        let current_exe_dir = std::path::Path::new(&current_exe)
            .parent()
            .unwrap()
            .to_str()
            .unwrap();
        let current_dir = current_dir.to_str().unwrap();
        if current_dir != current_exe_dir {
            std::env::set_current_dir(current_exe_dir).unwrap();
            log::trace!("current_dir: {}", current_exe_dir);
        }
    }
}

pub fn get_object_address<T>(object: &T) -> String {
    let raw_ptr = object as *const T;
    std::format!("{:?}", raw_ptr)
}

pub fn cast_to_raw_buffer<'a, T>(vec: &[T]) -> &'a [u8] {
    let buffer = vec.as_ptr() as *const u8;
    let size = std::mem::size_of::<T>() * vec.len();
    let buffer = unsafe { std::slice::from_raw_parts(buffer, size) };
    buffer
}

pub fn cast_to_raw_type_buffer<'a, U>(buffer: *const u8, len: usize) -> &'a [U] {
    unsafe {
        let len = len / std::mem::size_of::<U>();
        std::slice::from_raw_parts(buffer as *const U, len)
    }
}

pub fn cast_to_type_buffer<'a, U>(buffer: &'a [u8]) -> &'a [U] {
    unsafe {
        let len = buffer.len() / std::mem::size_of::<U>();
        std::slice::from_raw_parts(buffer.as_ptr() as *const U, len)
    }
}

pub fn alignment(n: isize, align: isize) -> isize {
    return ((n) + (align) - 1) & !((align) - 1);
}

pub fn next_highest_power_of_two(v: isize) -> isize {
    let mut v = v;
    v = v - 1;
    v |= v >> 1;
    v |= v >> 2;
    v |= v >> 4;
    v |= v >> 8;
    v |= v >> 16;
    v = v + 1;
    v
}

pub fn calculate_mipmap_level(length: u32) -> u32 {
    let mut mipmap_level: u32 = 1;
    let mut length = length;
    while length > 4 {
        length /= 2;
        mipmap_level += 1;
    }
    return mipmap_level;
}

pub fn map_texture_cpu_sync(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    width: u32,
    height: u32,
    color_type: image::ColorType,
) -> Vec<u8> {
    assert_eq!(color_type, image::ColorType::Rgba32F);
    let bytes_per_pixel: usize = 4 * std::mem::size_of::<f32>();
    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    let buffer_dimensions = crate::buffer_dimensions::BufferDimensions::new(
        width as usize,
        height as usize,
        bytes_per_pixel,
    );
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
        texture.as_image_copy(),
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
        let deep_copy_data = padded_buffer.to_vec();
        drop(padded_buffer);
        output_buffer.unmap();
        return deep_copy_data;
    } else {
        panic!()
    }
}

pub fn map_texture_cube_cpu_sync(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    cube_map_texture: &wgpu::Texture,
    width: u32,
    height: u32,
    color_type: image::ColorType,
) -> Vec<Vec<u8>> {
    assert_eq!(color_type, image::ColorType::Rgba32F);
    let bytes_per_pixel: usize = 4 * std::mem::size_of::<f32>();
    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    let buffer_dimensions = crate::buffer_dimensions::BufferDimensions::new(
        width as usize,
        height as usize,
        bytes_per_pixel,
    );

    let copy_texutre = wgpu::ImageCopyTexture {
        texture: cube_map_texture,
        mip_level: 0,
        origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
        aspect: wgpu::TextureAspect::All,
    };

    let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: (buffer_dimensions.padded_bytes_per_row * buffer_dimensions.height * 6) as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    encoder.copy_texture_to_buffer(
        copy_texutre,
        wgpu::ImageCopyBuffer {
            buffer: &staging_buffer,
            layout: wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(buffer_dimensions.padded_bytes_per_row as u32),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width: width,
            height: height,
            depth_or_array_layers: 6,
        },
    );
    let submission_index = queue.submit(Some(encoder.finish()));
    let single_length = buffer_dimensions.padded_bytes_per_row * height as usize;
    let buffer_slice = staging_buffer.slice(..);
    let (sender, receiver) = std::sync::mpsc::channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());
    device.poll(wgpu::Maintain::WaitForSubmissionIndex(submission_index));
    let mut image_datas: Vec<Vec<u8>> = vec![];
    if let Ok(Ok(_)) = receiver.recv() {
        let data = buffer_slice.get_mapped_range();
        let mut chunk = data.chunks_exact(single_length);
        while let Some(data) = chunk.next() {
            let deep_copy_data = data.to_vec();
            image_datas.push(deep_copy_data);
        }
        drop(data);
        staging_buffer.unmap();
    } else {
        panic!()
    }
    assert_eq!(image_datas.len(), 6);
    return image_datas;
}

pub fn texture2d_from_rgba_rgba32_fimage(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    image: &image::Rgba32FImage,
    mip_level_count: u32,
) -> wgpu::Texture {
    let texture_extent = wgpu::Extent3d {
        depth_or_array_layers: 1,
        width: image.width(),
        height: image.height(),
    };

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: texture_extent,
        mip_level_count: mip_level_count,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba32Float,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let image_data = image.as_raw().as_slice();
    let image_data = cast_to_raw_buffer(image_data);
    queue.write_texture(
        texture.as_image_copy(),
        image_data,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(4 * std::mem::size_of::<f32>() as u32 * image.width()),
            rows_per_image: None,
        },
        texture_extent,
    );
    texture
}

pub fn texture2d_from_rgba_image_file(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    is_flipv: bool,
    path: &str,
) -> Option<(wgpu::Texture, image::RgbaImage)> {
    match image::open(path) {
        Ok(dynamic_image) => {
            let dynamic_image = dynamic_image.to_rgba8();
            if is_flipv {
                let dynamic_image = image::DynamicImage::ImageRgba8(dynamic_image.clone())
                    .flipv()
                    .to_rgba8();
                let gpu_texture = texture2d_from_rgba_image(device, queue, &dynamic_image);
                Some((gpu_texture, dynamic_image))
            } else {
                let gpu_texture = texture2d_from_rgba_image(device, queue, &dynamic_image);
                Some((gpu_texture, dynamic_image))
            }
        }
        Err(error) => {
            log::warn!("{error}");
            None
        }
    }
}

pub fn save_fft_result(filename: &str, buffer: &[f32]) {
    let root = plotters::prelude::IntoDrawingArea::into_drawing_area(
        plotters::prelude::BitMapBackend::new(filename, (1280, 360)),
    );
    root.fill(&plotters::style::WHITE).unwrap();
    let mut chart = plotters::prelude::ChartBuilder::on(&root)
        .margin(15)
        .x_label_area_size(25)
        .y_label_area_size(25)
        .build_cartesian_2d(0_f32..buffer.len() as f32, 0_f32..1_f32)
        .unwrap();

    chart.configure_mesh().draw().unwrap();

    chart
        .draw_series(plotters::series::LineSeries::new(
            buffer
                .iter()
                .enumerate()
                .map(|(i, v)| (i as f32, *v as f32)),
            &plotters::style::RED,
        ))
        .unwrap();

    root.present().unwrap();
}

pub fn texture2d_from_gray_image(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    image: &image::GrayImage,
) -> wgpu::Texture {
    let texture_extent = wgpu::Extent3d {
        depth_or_array_layers: 1,
        width: image.width(),
        height: image.height(),
    };

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: texture_extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::R8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    queue.write_texture(
        texture.as_image_copy(),
        image,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(1 * image.width()),
            rows_per_image: None,
        },
        texture_extent,
    );
    texture
}

pub fn texture2d_from_rgba_image(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    image: &image::RgbaImage,
) -> wgpu::Texture {
    let texture_extent = wgpu::Extent3d {
        depth_or_array_layers: 1,
        width: image.width(),
        height: image.height(),
    };

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: texture_extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    queue.write_texture(
        texture.as_image_copy(),
        image,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(4 * image.width()),
            rows_per_image: None,
        },
        texture_extent,
    );
    texture
}

pub fn textures_from_yuv420p_image(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    yuv_image: &YUV420pImage,
) -> (wgpu::Texture, wgpu::Texture, wgpu::Texture) {
    let size = yuv_image.get_size();
    let create_texture = |width: u32, height: u32, buffer: &[u8]| {
        let texture_extent = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            texture.as_image_copy(),
            buffer,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(1 * width),
                rows_per_image: None,
            },
            texture_extent,
        );
        texture
    };
    let y_texture = create_texture(size.x, size.y, yuv_image.get_y_buffer());
    let u_texture = create_texture(size.x / 2, size.y / 2, yuv_image.get_u_buffer());
    let v_texture = create_texture(size.x / 2, size.y / 2, yuv_image.get_v_buffer());
    (y_texture, u_texture, v_texture)
}

pub fn create_gpu_vertex_buffer_from<T>(
    device: &wgpu::Device,
    vertex: &Vec<T>,
    label: Option<&str>,
) -> wgpu::Buffer {
    let vertex_buf = wgpu::util::DeviceExt::create_buffer_init(
        device,
        &wgpu::util::BufferInitDescriptor {
            label,
            contents: cast_to_raw_buffer(vertex),
            usage: wgpu::BufferUsages::VERTEX,
        },
    );
    vertex_buf
}

pub fn create_gpu_index_buffer_from(
    device: &wgpu::Device,
    index_data: &Vec<u32>,
    label: Option<&str>,
) -> wgpu::Buffer {
    let unsafe_index_data_raw_buffer: &[u8] = unsafe {
        std::slice::from_raw_parts(
            index_data.as_ptr() as *const u8,
            index_data.len() * std::mem::size_of::<u32>(),
        )
    };
    let index_buf = wgpu::util::DeviceExt::create_buffer_init(
        device,
        &wgpu::util::BufferInitDescriptor {
            label,
            contents: unsafe_index_data_raw_buffer,
            usage: wgpu::BufferUsages::INDEX,
        },
    );
    index_buf
}

pub fn create_gpu_uniform_buffer_from<T>(
    device: &wgpu::Device,
    data: &T,
    label: Option<&str>,
) -> wgpu::Buffer {
    let unsafe_uniform_raw_buffer: &[u8] = unsafe {
        std::slice::from_raw_parts((data as *const T) as *const u8, std::mem::size_of::<T>())
    };
    let uniform_buf = wgpu::util::DeviceExt::create_buffer_init(
        device,
        &wgpu::util::BufferInitDescriptor {
            label,
            contents: unsafe_uniform_raw_buffer,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        },
    );
    uniform_buf
}

#[macro_export]
macro_rules! VertexBufferLayout {
    ($Type:ident, $vertex_formats:expr) => {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<$Type>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &{
                let mut attributes: Vec<wgpu::VertexAttribute> = vec![];
                let mut array_stride: u64 = 0;
                for (index, vertex_format) in $vertex_formats.iter().enumerate() {
                    let size = vertex_format.size();
                    let vertex_attribute = wgpu::VertexAttribute {
                        format: vertex_format.clone(),
                        offset: array_stride,
                        shader_location: index as u32,
                    };
                    attributes.push(vertex_attribute);
                    array_stride += size;
                }
                attributes
            },
        }
    };
}

pub fn create_pure_color_rgba8_image(
    width: u32,
    height: u32,
    color: &wgpu::Color,
) -> image::DynamicImage {
    let mut image = image::DynamicImage::new_rgba8(width, height);
    {
        let image = image.as_mut_rgba8().unwrap();
        for pixel in image.pixels_mut() {
            let pixel = &mut pixel.0;
            pixel[0] = (color.r * 255.0).clamp(0.0, 255.0) as u8;
            pixel[1] = (color.g * 255.0).clamp(0.0, 255.0) as u8;
            pixel[2] = (color.b * 255.0).clamp(0.0, 255.0) as u8;
            pixel[3] = (color.a * 255.0).clamp(0.0, 255.0) as u8;
        }
    }
    image
}

pub fn create_pure_color_rgba8_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    width: u32,
    height: u32,
    color: &wgpu::Color,
) -> wgpu::Texture {
    let image = create_pure_color_rgba8_image(width, height, color);
    texture2d_from_rgba_image(device, queue, image.as_rgba8().unwrap())
}

pub fn get_resource_path(resource_name: &str) -> String {
    crate::file_manager::FileManager::default()
        .lock()
        .unwrap()
        .get_resource_path(resource_name)
}

pub fn russimp_mat4_to_glam_mat4(transformation: &russimp::Matrix4x4) -> glam::Mat4 {
    glam::mat4(
        glam::vec4(
            transformation.a1,
            transformation.a2,
            transformation.a3,
            transformation.a4,
        ),
        glam::vec4(
            transformation.b1,
            transformation.b2,
            transformation.b3,
            transformation.b4,
        ),
        glam::vec4(
            transformation.c1,
            transformation.c2,
            transformation.c3,
            transformation.c4,
        ),
        glam::vec4(
            transformation.d1,
            transformation.d2,
            transformation.d3,
            transformation.d4,
        ),
    )
}

pub fn ray_intersection_hit_test(
    actor: &Actor,
    mouse_position: winit::dpi::PhysicalPosition<f64>,
    window_size: winit::dpi::PhysicalSize<u32>,
    model_matrix: glam::Mat4,
    camera: &Camera,
) -> Vec<RayHitTestResult> {
    let x = math_remap_value_range(
        mouse_position.x as f64,
        std::ops::Range::<f64> {
            start: 0.0,
            end: window_size.width as f64,
        },
        std::ops::Range::<f64> {
            start: -1.0,
            end: 1.0,
        },
    ) as f32;
    let y = -math_remap_value_range(
        mouse_position.y as f64,
        std::ops::Range::<f64> {
            start: 0.0,
            end: window_size.height as f64,
        },
        std::ops::Range::<f64> {
            start: -1.0,
            end: 1.0,
        },
    ) as f32;
    let near_point = glam::Vec3::new(x, y, 0.0);
    let far_point = glam::Vec3::new(x, y, 1.0);
    let near_point_at_world_space = screent_space_to_world_space(
        near_point,
        model_matrix,
        camera.get_view_matrix(),
        camera.get_projection_matrix(),
    );

    let far_point_at_world_space = screent_space_to_world_space(
        far_point,
        model_matrix,
        camera.get_view_matrix(),
        camera.get_projection_matrix(),
    );
    let mut results = Vec::<RayHitTestResult>::new();
    for static_mesh in actor.get_static_meshs() {
        let name = static_mesh.get_name();
        let triangles_view = static_mesh.get_triangles_view();
        for triangle in triangles_view {
            let intersection_point = triangle_plane_ray_intersection(
                glam::vec4(triangle.0.x, triangle.0.y, triangle.0.z, 1.0).xyz(),
                glam::vec4(triangle.1.x, triangle.1.y, triangle.1.z, 1.0).xyz(),
                glam::vec4(triangle.2.x, triangle.2.y, triangle.2.z, 1.0).xyz(),
                near_point_at_world_space,
                far_point_at_world_space - near_point_at_world_space,
            );
            if let Some(intersection_point) = intersection_point {
                let result = RayHitTestResult {
                    mesh_name: name.to_string(),
                    intersection_point,
                };
                results.push(result);
                break;
            }
        }
    }
    results
}

// http://holger.dammertz.org/stuff/notes_HammersleyOnHemisphere.html
pub fn radical_inverse_vdc(bits: u32) -> f32 {
    let mut bits = bits;
    bits = (bits << 16) | (bits >> 16);
    bits = ((bits & 0x55555555) << 1) | ((bits & 0xAAAAAAAA) >> 1);
    bits = ((bits & 0x33333333) << 2) | ((bits & 0xCCCCCCCC) >> 2);
    bits = ((bits & 0x0F0F0F0F) << 4) | ((bits & 0xF0F0F0F0) >> 4);
    bits = ((bits & 0x00FF00FF) << 8) | ((bits & 0xFF00FF00) >> 8);
    bits as f32 * 2.3283064365386963e-10 // / 0x100000000
}

pub fn hammersley_2d(i: u32, n: u32) -> glam::Vec2 {
    glam::vec2(i as f32 / n as f32, radical_inverse_vdc(i))
}

pub fn hemisphere_sample_uniform(u: f32, v: f32) -> glam::Vec3 {
    let phi = v * std::f32::consts::TAU;
    let cos_theta = 1.0 - u;
    let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
    glam::vec3(phi.cos() * sin_theta, phi.sin() * sin_theta, cos_theta)
}

pub fn importance_sample_ggx(xi: glam::Vec2, roughness: f32) -> glam::Vec3 {
    let a = roughness * roughness;
    let phi = std::f32::consts::TAU * xi.x;
    let cos_theta = ((1.0 - xi.y) / (1.0 + (a * a - 1.0) * xi.y)).sqrt();
    let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
    glam::vec3(phi.cos() * sin_theta, phi.sin() * sin_theta, cos_theta)
}

pub fn sample_equirectangular_map(sample_picker: glam::Vec3) -> glam::Vec2 {
    let x = ((sample_picker.z.atan2(sample_picker.x) + std::f32::consts::PI)
        / std::f32::consts::TAU)
        .clamp(0.0, 1.0);
    let y = (sample_picker.y.acos() / std::f32::consts::PI).clamp(0.0, 1.0);
    glam::vec2(x, y)
}

pub fn geometry_schlick_ggx(n_dot_v: f32, roughness: f32) -> f32 {
    let a = roughness;
    let k = (a * a) / 2.0f32;
    let nom = n_dot_v;
    let denom = n_dot_v * (1.0 - k) + k;
    return nom / denom;
}

pub fn geometry_smith(n: glam::Vec3, v: glam::Vec3, l: glam::Vec3, roughness: f32) -> f32 {
    let n_dot_v = n.dot(v).max(0.0);
    let n_dot_l = n.dot(l).max(0.0);
    let ggx2 = geometry_schlick_ggx(n_dot_v, roughness);
    let ggx1 = geometry_schlick_ggx(n_dot_l, roughness);
    return ggx1 * ggx2;
}

pub fn convert_coordinate_system(
    v: glam::Vec3,
    x: glam::Vec3,
    y: glam::Vec3,
    z: glam::Vec3,
) -> glam::Vec3 {
    let mut x_axis = x.xyzx();
    x_axis.w = 0.0;
    let mut y_axis = y.xyzx();
    y_axis.w = 0.0;
    let mut z_axis = z.xyzx();
    z_axis.w = 0.0;
    let mut v = v.xyzx();
    v.w = 1.0;
    (glam::mat4(x_axis, y_axis, z_axis, glam::Vec4::W) * v).xyz()
}

pub fn reflect_vec3(i: glam::Vec3, n: glam::Vec3) -> glam::Vec3 {
    i - 2.0 * n.dot(i) * n
}

#[cfg(test)]
pub mod test {
    use super::{alignment, math_remap_value_range, triangle_plane_ray_intersection};
    use crate::util::next_highest_power_of_two;

    #[test]
    pub fn next_highest_power_of_two_test() {
        assert_eq!(next_highest_power_of_two(418), 512);
    }

    #[test]
    pub fn alignment_test() {
        assert_eq!(alignment(418, 4), 420);
    }

    #[test]
    pub fn math_remap_value_range_test() {
        let mapped_value = math_remap_value_range(
            1.0,
            std::ops::Range::<f64> {
                start: 0.0,
                end: 2.0,
            },
            std::ops::Range::<f64> {
                start: 0.0,
                end: 100.0,
            },
        );
        assert_eq!(mapped_value, 50.0_f64);

        let mapped_value = math_remap_value_range(
            0.0,
            std::ops::Range::<f64> {
                start: 0.0,
                end: 2.0,
            },
            std::ops::Range::<f64> {
                start: 0.0,
                end: 100.0,
            },
        );
        assert_eq!(mapped_value, 0.0_f64);

        let mapped_value = math_remap_value_range(
            2.0,
            std::ops::Range::<f64> {
                start: 0.0,
                end: 2.0,
            },
            std::ops::Range::<f64> {
                start: 0.0,
                end: 100.0,
            },
        );
        assert_eq!(mapped_value, 100.0_f64);

        let mapped_value = math_remap_value_range(
            -1.0,
            std::ops::Range::<f64> {
                start: 0.0,
                end: 2.0,
            },
            std::ops::Range::<f64> {
                start: 0.0,
                end: 100.0,
            },
        );
        assert_eq!(mapped_value, -50.0_f64);
    }

    #[test]
    pub fn triangle_plane_ray_intersection_test() {
        let a = glam::Vec3 {
            x: -1.0,
            y: 1.0,
            z: 1.0,
        };
        let b = glam::Vec3 {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        };
        let c = glam::Vec3 {
            x: 0.0,
            y: -1.0,
            z: 1.0,
        };
        let origin = glam::Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let direction = glam::Vec3 {
            x: 0.0,
            y: -1.0,
            z: 1.0,
        } - origin;
        let intersection_point = triangle_plane_ray_intersection(a, b, c, origin, direction);
        println!("{:?}", intersection_point);
    }
}
