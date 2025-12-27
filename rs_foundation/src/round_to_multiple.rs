pub trait RoundToMultiple {
    fn round_down_to_multiple(&self, multiple: Self) -> Self;
    fn round_up_to_multiple(&self, multiple: Self) -> Self;
}

impl RoundToMultiple for f32 {
    fn round_down_to_multiple(&self, multiple: Self) -> Self {
        (self / multiple).floor() * multiple
    }

    fn round_up_to_multiple(&self, multiple: Self) -> Self {
        (self / multiple).ceil() * multiple
    }
}

impl RoundToMultiple for f64 {
    fn round_down_to_multiple(&self, multiple: Self) -> Self {
        (self / multiple).floor() * multiple
    }

    fn round_up_to_multiple(&self, multiple: Self) -> Self {
        (self / multiple).ceil() * multiple
    }
}

impl RoundToMultiple for i32 {
    fn round_down_to_multiple(&self, multiple: Self) -> Self {
        self / multiple * multiple
    }

    fn round_up_to_multiple(&self, multiple: Self) -> Self {
        if *self % multiple == 0 {
            *self
        } else {
            (self / multiple + 1) * multiple
        }
    }
}

#[cfg(test)]
mod test {
    use crate::round_to_multiple::RoundToMultiple;

    #[test]
    fn round_down_to_multiple_test() {
        let test_values = vec![3.2, -0.7, 5.5, -2.3];
        let expect: Vec<f32> = vec![0.0, -4.0, 4.0, -4.0];
        let multiple = 4.0;
        for (&value, expect) in test_values.iter().zip(expect) {
            let result = value.round_down_to_multiple(multiple);
            assert_eq!(result, expect);
        }
    }

    #[test]
    fn round_down_to_multiple_test2() {
        let test_values = vec![3.2, -0.7, 5.5, -2.3];
        let expect: Vec<f32> = vec![0.0, -4.3, 4.3, -4.3];
        let multiple = 4.3;
        for (&value, expect) in test_values.iter().zip(expect) {
            let result = value.round_down_to_multiple(multiple);
            assert_eq!(result, expect);
        }
    }

    #[test]
    fn round_up_to_multiple_test() {
        let test_values = vec![3.2, -0.7, 5.5, -2.3];
        let expect: Vec<f32> = vec![4.0, 0.0, 8.0, 0.0];
        let multiple = 4.0;
        for (&value, expect) in test_values.iter().zip(expect) {
            let result = value.round_up_to_multiple(multiple);
            assert_eq!(result, expect);
        }
    }
}
