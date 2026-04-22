use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

struct Data {
    time: Instant,
    bytes: usize,
}

pub struct BandwidthMeter {
    datas: VecDeque<Data>,
    bandwidth: usize,
}

impl BandwidthMeter {
    pub fn new() -> Self {
        Self {
            datas: VecDeque::with_capacity(256),
            bandwidth: 0,
        }
    }

    pub fn send(&mut self, bytes: usize) -> usize {
        self.bandwidth = Self::estimate_continuously(&mut self.datas, bytes);
        self.bandwidth
    }

    fn estimate_continuously(datas: &mut VecDeque<Data>, bytes: usize) -> usize {
        let now = Instant::now();
        datas.push_back(Data {
            time: now,
            bytes: bytes,
        });
        let one_sec_ago = now - Duration::from_secs(1);
        while datas.front().map_or(false, |d| d.time < one_sec_ago) {
            datas.pop_front();
        }
        let bytes = datas.iter().map(|x| x.bytes).fold(0, |acc, x| acc + x);
        bytes
    }

    pub fn bandwidth(&self) -> usize {
        self.bandwidth
    }
}
