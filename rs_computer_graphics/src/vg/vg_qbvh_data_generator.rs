use parry3d::{bounding_volume::Aabb, partitioning::QbvhDataGenerator};

pub struct VGQbvhDataGenerator {
    pub aabbs: Vec<Aabb>,
}

impl QbvhDataGenerator<usize> for VGQbvhDataGenerator {
    fn size_hint(&self) -> usize {
        self.aabbs.len()
    }

    fn for_each(&mut self, mut f: impl FnMut(usize, Aabb)) {
        for (index, aabb) in self.aabbs.iter().enumerate() {
            f(index, *aabb);
        }
    }
}
