use std::sync::Arc;

pub struct ComputeDispatcher {
    workgroup_size: glam::UVec3,
    is_single_thread: bool,
}

impl ComputeDispatcher {
    pub fn new(workgroup_size: glam::UVec3) -> ComputeDispatcher {
        ComputeDispatcher {
            workgroup_size,
            is_single_thread: false,
        }
    }

    pub fn dispatch_workgroups(
        &self,
        num_work_groups: glam::UVec3,
        work: impl Fn(glam::UVec3, glam::UVec3, glam::UVec3, u32) + Send + Sync + 'static,
    ) {
        if self.is_single_thread {
            for gz in 0..num_work_groups.z {
                for gy in 0..num_work_groups.y {
                    for gx in 0..num_work_groups.x {
                        let group_id = glam::uvec3(gx, gy, gz);
                        Self::do_work(group_id, &self.workgroup_size, &work);
                    }
                }
            }
        } else {
            let (sender, receiver) = std::sync::mpsc::channel();
            let work = Arc::new(work);
            for gz in 0..num_work_groups.z {
                for gy in 0..num_work_groups.y {
                    for gx in 0..num_work_groups.x {
                        let group_id = glam::uvec3(gx, gy, gz);
                        crate::thread_pool::ThreadPool::global().spawn({
                            let sender = sender.clone();
                            let workgroup_size = self.workgroup_size;
                            let work = work.clone();
                            move || {
                                Self::do_work(group_id, &workgroup_size, work.as_ref());
                                sender.send(()).unwrap();
                            }
                        });
                    }
                }
            }
            for _ in 0..num_work_groups.element_product() {
                receiver.recv().unwrap();
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

    pub fn set_is_single_thread(mut self, is_single_thread: bool) -> Self {
        self.is_single_thread = is_single_thread;
        return self;
    }

    fn do_work(
        group_id: glam::UVec3,
        workgroup_size: &glam::UVec3,
        work: &(impl Fn(glam::UVec3, glam::UVec3, glam::UVec3, u32) + Send + Sync + 'static),
    ) {
        for tz in 0..workgroup_size.z {
            for ty in 0..workgroup_size.y {
                for tx in 0..workgroup_size.x {
                    let group_thread_id = glam::uvec3(tx, ty, tz);
                    let group_index =
                        tz * workgroup_size.x * workgroup_size.y + ty * workgroup_size.x + tx;
                    let dispatch_thread_id = glam::uvec3(
                        group_id.x * workgroup_size.x + tx,
                        group_id.y * workgroup_size.y + ty,
                        group_id.z * workgroup_size.z + tz,
                    );
                    work(group_thread_id, group_id, dispatch_thread_id, group_index);
                }
            }
        }
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
                assert!(group_index < workgroup_size.x * workgroup_size.y);
                let image = unsafe { wrapper_type.0.as_mut().unwrap() };
                image.as_raw();
                if let Some(pixel) = image.get_pixel_mut_checked(group_index, 0) {
                    *pixel = image::Rgba::<u8>([255, 0, 0, 255]);
                }
            }
        });
        assert_eq!(
            *image.get_pixel(workgroup_size.x * workgroup_size.y - 1, 0),
            image::Rgba::<u8>([255, 0, 0, 255])
        );
        assert_eq!(
            *image.get_pixel(workgroup_size.x * workgroup_size.y, 0),
            image::Rgba::<u8>([0, 0, 0, 0])
        );
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

    #[test]
    fn test3() {
        struct Wrapper(*mut Vec<i32>);
        unsafe impl Send for Wrapper {}
        unsafe impl Sync for Wrapper {}
        let workgroup_size = glam::UVec3::splat(1);
        let num_work_groups = glam::UVec3::splat(10);
        let mut datas = vec![0; 10 * 10 * 10];
        let wrapper = Wrapper((&mut datas) as *mut _);
        ComputeDispatcher::new(workgroup_size).dispatch_workgroups(num_work_groups, {
            move |_, _, dispatch_thread_id, _| {
                // https://users.rust-lang.org/t/how-to-share-a-raw-pointer-between-threads/77596
                let wrapper = &wrapper;
                let datas: &mut Vec<i32> = unsafe { wrapper.0.as_mut().unwrap() };
                let index = 10 * 10 * dispatch_thread_id.z
                    + 10 * dispatch_thread_id.y
                    + dispatch_thread_id.x;
                datas[index as usize] += 1;
            }
        });
        for data in datas {
            assert_eq!(data, 1);
        }
    }
}
