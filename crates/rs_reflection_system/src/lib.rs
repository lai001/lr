use rs_reflection_core::*;

pub struct ReflectionSystem {
    struct_meta_containers: Vec<Box<dyn StructMetaContainer>>,
}

impl ReflectionSystem {
    pub fn new(struct_meta_containers: Vec<Box<dyn StructMetaContainer>>) -> Self {
        Self {
            struct_meta_containers,
        }
    }

    pub fn register(&mut self, struct_meta_container: Box<dyn StructMetaContainer>) {
        self.struct_meta_containers.push(struct_meta_container);
    }

    pub fn find(&self, name: &str) -> Option<&Box<dyn StructMetaContainer>> {
        for struct_meta_container in self.struct_meta_containers.iter() {
            let struct_meta = struct_meta_container.get_struct_meta();
            if struct_meta.name == name {
                return Some(struct_meta_container);
            }
        }
        return None;
    }

    pub fn find_mut(&mut self, name: &str) -> Option<&mut Box<dyn StructMetaContainer>> {
        for struct_meta_container in self.struct_meta_containers.iter_mut() {
            let struct_meta = struct_meta_container.get_struct_meta();
            if struct_meta.name == name {
                return Some(struct_meta_container);
            }
        }
        return None;
    }
}
