use glam::Vec3Swizzles;

pub struct ComputeDispatcher {
    workgroup_size: glam::UVec3,
}

impl ComputeDispatcher {
    pub fn new(workgroup_size: glam::UVec3) -> ComputeDispatcher {
        ComputeDispatcher { workgroup_size }
    }

    pub fn dispatch_workgroups(
        &self,
        num_work_groups: glam::UVec3,
        work: impl FnMut(glam::UVec3, glam::UVec3, glam::UVec3, u32) + Send + Clone + 'static,
    ) {
        let (sender, receiver) = std::sync::mpsc::channel();
        let groups = num_work_groups.element_product();
        let mut finish_grous = 0;
        for group_index in 0..groups {
            crate::thread_pool::ThreadPool::global().spawn({
                let workgroup_size = self.workgroup_size;
                let mut work = work.clone();
                let sender = sender.clone();
                move || {
                    let a = num_work_groups.xy().element_product();
                    let b = group_index % a;
                    let group_id = glam::uvec3(
                        b % num_work_groups.y,
                        b / num_work_groups.x,
                        group_index / a,
                    );
                    for x in 0..workgroup_size.x {
                        for y in 0..workgroup_size.y {
                            for z in 0..workgroup_size.z {
                                let group_thread_id = glam::uvec3(x, y, z);
                                let dispatch_thread_id =
                                    group_id * workgroup_size + group_thread_id;
                                work(group_thread_id, group_id, dispatch_thread_id, group_index);
                            }
                        }
                    }
                    sender.send(()).unwrap();
                }
            });
        }
        for _ in receiver {
            finish_grous += 1;
            if finish_grous == groups {
                break;
            }
        }
    }

    pub fn estimate_num_work_groups(
        works_hint: &glam::UVec3,
        workgroup_size: &glam::UVec3,
    ) -> glam::UVec3 {
        glam::uvec3(
            works_hint.x.div_ceil(workgroup_size.x),
            works_hint.y.div_ceil(workgroup_size.y),
            works_hint.z.div_ceil(workgroup_size.z),
        )
    }
}

#[cfg(test)]
mod test {
    use super::ComputeDispatcher;
    use crate::misc::is_point_in_polygon;
    use std::sync::Arc;

    struct UnsafeWrapperType(*mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>);
    unsafe impl Send for UnsafeWrapperType {}
    unsafe impl Sync for UnsafeWrapperType {}

    #[test]
    fn compute_dispatcher_test() {
        let size = glam::uvec3(4096, 4096, 1);
        let mut image = image::RgbaImage::new(size.x, size.y);
        let raw_image = (&mut image) as *mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>;
        let wrapper_type = Arc::new(UnsafeWrapperType(raw_image));
        let workgroup_size = glam::UVec2::splat(32).extend(1);
        let num_work_groups = ComputeDispatcher::estimate_num_work_groups(&size, &workgroup_size);
        ComputeDispatcher::new(workgroup_size).dispatch_workgroups(num_work_groups, {
            move |_, _, dispatch_thread_id, _| {
                let image = unsafe { wrapper_type.0.as_mut().unwrap() };
                if let Some(pixel) =
                    image.get_pixel_mut_checked(dispatch_thread_id.x, dispatch_thread_id.y)
                {
                    *pixel = image::Rgba::<u8>([255, 0, 0, 255]);
                }
            }
        });
        for pixel in image.pixels() {
            assert_eq!(*pixel, image::Rgba::<u8>([255, 0, 0, 255]));
        }
    }

    #[test]
    fn compute_dispatcher_test1() {
        let size = glam::uvec3(4096, 4096, 1);
        let mut image = image::RgbaImage::new(size.x, size.y);
        let raw_image = (&mut image) as *mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>;
        let wrapper_type = Arc::new(UnsafeWrapperType(raw_image));
        let workgroup_size = glam::UVec2::splat(32).extend(1);
        let num_work_groups = ComputeDispatcher::estimate_num_work_groups(&size, &workgroup_size);
        ComputeDispatcher::new(workgroup_size).dispatch_workgroups(num_work_groups, {
            move |_, _, _, group_index| {
                assert!(group_index < num_work_groups.element_product());
                let image = unsafe { wrapper_type.0.as_mut().unwrap() };
                image.as_raw();
                if let Some(pixel) = image.get_pixel_mut_checked(group_index, 0) {
                    *pixel = image::Rgba::<u8>([255, 0, 0, 255]);
                }
            }
        });
        for (i, pixel) in image.pixels().enumerate() {
            if (0..size.x).contains(&(i as u32)) {
                assert_eq!(*pixel, image::Rgba::<u8>([255, 0, 0, 255]));
            }
        }
    }

    #[test]
    fn compute_dispatcher_test2() {
        let size = glam::uvec3(4096, 4096, 1);
        let polygon = crate::misc::generate_circle_points(glam::vec2(2048.0, 2048.0), 1024.0, 256);
        let mut image = image::RgbaImage::new(size.x, size.y);
        let raw_image = (&mut image) as *mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>;
        let wrapper_type = Arc::new(UnsafeWrapperType(raw_image));
        let workgroup_size = glam::UVec2::splat(32).extend(1);
        let num_work_groups = ComputeDispatcher::estimate_num_work_groups(&size, &workgroup_size);
        let inside = image::Rgba::<u8>([0, 0, 0, 255]);
        let outside = image::Rgba::<u8>([255, 255, 255, 255]);
        ComputeDispatcher::new(workgroup_size).dispatch_workgroups(num_work_groups, {
            move |_, _, dispatch_thread_id, _| {
                let image = unsafe { wrapper_type.0.as_mut().unwrap() };
                if let Some(pixel) =
                    image.get_pixel_mut_checked(dispatch_thread_id.x, dispatch_thread_id.y)
                {
                    let is_inside = is_point_in_polygon(
                        glam::vec2(dispatch_thread_id.x as f32, dispatch_thread_id.y as f32),
                        &polygon,
                        true,
                    );
                    if is_inside {
                        *pixel = inside;
                    } else {
                        *pixel = outside;
                    }
                }
            }
        });
        assert_eq!(*image.get_pixel(2048, 2048), inside);
        assert_eq!(*image.get_pixel(1023, 2048), outside);
    }
}
