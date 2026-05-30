use crate::parallel::ComputeDispatcher;
use rayon::iter::*;
use rs_foundation::unsafe_type_wrapper::UnsafeTypeWrapper;

pub fn bitonic_sort<T: Ord>(datas: &mut [T]) {
    let mut k: usize = 2;
    let n = datas.len();
    assert_eq!(n % 2, 0);
    while k <= n {
        let mut j = k / 2;
        while j > 0 {
            for i in 0..n {
                let l = i ^ j;
                if l > i {
                    let c1 = (i & k == 0) && datas[i] > datas[l];
                    let c2 = (i & k != 0) && datas[i] < datas[l];
                    if c1 || c2 {
                        datas.swap(i, l);
                    }
                }
            }
            j /= 2;
        }
        k *= 2;
    }
}

pub fn bitonic_sort_parallel<const BLOCK_SIZE: u32, T: Ord + Copy + Send + Sync + 'static>(
    datas: &mut [T],
) {
    let mut k: usize = 2;
    let n = datas.len();
    assert_eq!(n % 2, 0);
    while k <= n {
        let mut j = k / 2;
        while j > 0 {
            let wrapper = UnsafeTypeWrapper::from_mut_ref(datas);
            let compute_dispatcher = ComputeDispatcher::new(glam::uvec3(BLOCK_SIZE, 1, 1));
            compute_dispatcher.dispatch_workgroups(
                glam::uvec3(n as u32 / BLOCK_SIZE, 1, 1),
                move |_, _, dispatch_thread_id, _| {
                    let wrapper = &wrapper;
                    let datas = wrapper.mut_ref();
                    let i = dispatch_thread_id.x as usize;
                    let l = i ^ j;
                    if l > i {
                        let c0 = (i & k == 0) && datas[i] > datas[l];
                        let c1 = (i & k != 0) && datas[i] < datas[l];
                        if c0 || c1 {
                            datas.swap(i, l);
                        }
                    }
                },
            );
            j /= 2;
        }
        k *= 2;
    }
}

pub fn bitonic_sort_non_power_of_two_parallel<T: Ord + Copy + Send + Sync + 'static>(
    nums: &mut [T],
) {
    let n = nums.len();
    let m = n / 2;
    let mut stride = 1;
    while stride < n {
        let mut step = stride;
        while step > 0 {
            let wrapper = UnsafeTypeWrapper::from_mut_ref(nums);
            (0..m).into_par_iter().for_each({
                move |idx| {
                    let wrapper = &wrapper;
                    let p = wrapper.mut_ref().as_mut_ptr();
                    let a = 2 * step * (idx / step);
                    let b = idx % step;
                    let u = if step == stride {
                        a + step - 1 - b
                    } else {
                        a + b
                    };
                    let d = a + b + step;
                    if d < n {
                        unsafe {
                            let pu = p.add(u);
                            let pd = p.add(d);

                            if *pu > *pd {
                                std::ptr::swap(pu, pd);
                            }
                        }
                    }
                }
            });
            step /= 2;
        }
        stride *= 2;
    }
}

#[cfg(test)]
mod test {
    use crate::bitonic_sort::{
        bitonic_sort, bitonic_sort_non_power_of_two_parallel, bitonic_sort_parallel,
    };
    use rand::RngExt;

    fn generate_datas(nums: usize) -> Vec<u32> {
        let mut datas = Vec::with_capacity(nums);
        let mut rng = rand::rng();
        for _ in 0..nums {
            let value = rng.random_range(0..1024);
            datas.push(value);
        }
        datas
    }

    #[test]
    fn bitonic_sort_test() {
        let mut datas = generate_datas(1 << 22);
        bitonic_sort(&mut datas);
        let is_sorted = datas.windows(2).all(|w| w[0] <= w[1]);
        assert!(is_sorted);
    }

    #[test]
    fn bitonic_sort_parallel_test_0() {
        let mut datas = generate_datas(1 << 22);
        bitonic_sort_parallel::<4096, _>(&mut datas);
        let is_sorted = datas.windows(2).all(|w| w[0] <= w[1]);
        assert!(is_sorted);
    }

    #[test]
    fn bitonic_sort_parallel_test_1() {
        let mut datas = generate_datas(1 << 20);
        bitonic_sort_parallel::<4096, _>(&mut datas);
        let is_sorted = datas.windows(2).all(|w| w[0] <= w[1]);
        assert!(is_sorted);
    }

    #[test]
    fn bitonic_sort_non_power_of_two_parallel_test() {
        for nums in [1595, 3156, 5489, 2152, 87616, 92123] {
            let mut datas = generate_datas(nums);
            bitonic_sort_non_power_of_two_parallel::<_>(&mut datas);
            let is_sorted = datas.windows(2).all(|w| w[0] <= w[1]);
            assert!(is_sorted);
        }
    }
}
