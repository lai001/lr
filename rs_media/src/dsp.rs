use std::{f64::consts::PI, vec};

pub struct DSP {}

impl DSP {
    pub fn dft(buffer: &[f64]) -> Vec<f64> {
        let mut dft_buffer: Vec<f64> = vec![];
        dft_buffer.resize(buffer.len(), 0.0_f64);
        let n = buffer.len();
        for i in 0..n {
            let mut result = 0.0_f64;
            for j in i..n {
                let a = (2.0 * PI * i as f64 * j as f64 / n as f64).cos();
                let b = (2.0 * PI * i as f64 * j as f64 / n as f64).sin();
                result = result + buffer[j] * (a - b);
            }
            dft_buffer[i] = result;
        }
        dft_buffer
    }

    pub fn idft(buffer: &[f64]) -> Vec<f64> {
        Self::dft(buffer)
            .iter()
            .map(|x| x / buffer.len() as f64)
            .collect()
    }
}

pub struct ProceduralSignal {}

impl ProceduralSignal {
    pub fn sin(amplitude: f64, frequency: f64, phase: f64, size: usize) -> Vec<f64> {
        let mut buffer: Vec<f64> = vec![];
        buffer.resize(size, 0.0);
        for i in 0..size {
            buffer[i] = amplitude * (frequency * 2.0 * PI * i as f64 + phase).sin();
        }
        buffer
    }

    pub fn cos(amplitude: f64, frequency: f64, phase: f64, size: usize) -> Vec<f64> {
        let mut buffer: Vec<f64> = vec![];
        buffer.resize(size, 0.0);
        for i in 0..size {
            buffer[i] = amplitude * (frequency * 2.0 * PI * i as f64 + phase).cos();
        }
        buffer
    }
}

pub fn next_number_power_of_two(n: i32) -> i32 {
    let mut val: u32 = 0;
    let mut num = n;
    num = num - 1;
    while val <= 4 {
        num = num | (num >> 2_i32.pow(val));
        val = val + 1;
    }
    num = num + 1;
    num
}
