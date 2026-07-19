use rs_engine::{
    camera_component::CameraComponent,
    collision_componenet::CollisionComponent,
    components::{
        component::Component, point_light_component::PointLightComponent,
        spot_light_component::SpotLightComponent, text_component::TextComponent,
    },
    scene_node::SceneComponent,
    skeleton_mesh_component::SkeletonMeshComponent,
    static_mesh_component::StaticMeshComponent,
};
use rust_i18n::t;
use std::{any::type_name, collections::BTreeMap};

pub trait ComponentCreator {
    fn create(
        &self,
        name: String,
        transformation: glam::Mat4,
    ) -> crate::error::Result<Box<dyn Component>>;

    fn display_name(&self) -> String {
        let name = self.name();
        t!(name).to_string()
    }

    fn name(&self) -> &'static str;
}

struct SpotLightComponentCreator {}

impl ComponentCreator for SpotLightComponentCreator {
    fn create(
        &self,
        name: String,
        transformation: glam::Mat4,
    ) -> crate::error::Result<Box<dyn Component>> {
        Ok(Box::new(SpotLightComponent::new(name, transformation)))
    }

    fn name(&self) -> &'static str {
        type_name::<SpotLightComponent>()
    }

    fn display_name(&self) -> String {
        t!("Spot Light").to_string()
    }
}

struct StaticMeshComponentCreator {}

impl ComponentCreator for StaticMeshComponentCreator {
    fn create(
        &self,
        name: String,
        transformation: glam::Mat4,
    ) -> crate::error::Result<Box<dyn Component>> {
        Ok(Box::new(StaticMeshComponent::new(
            name,
            None,
            None,
            transformation,
        )))
    }

    fn name(&self) -> &'static str {
        type_name::<StaticMeshComponent>()
    }

    fn display_name(&self) -> String {
        t!("Static Mesh").to_string()
    }
}

struct SkeletonMeshComponentCreator {}

impl ComponentCreator for SkeletonMeshComponentCreator {
    fn create(
        &self,
        name: String,
        transformation: glam::Mat4,
    ) -> crate::error::Result<Box<dyn Component>> {
        Ok(Box::new(SkeletonMeshComponent::new(
            name,
            None,
            vec![],
            None,
            None,
            transformation,
        )))
    }

    fn name(&self) -> &'static str {
        type_name::<SkeletonMeshComponent>()
    }

    fn display_name(&self) -> String {
        t!("Skeleton Mesh").to_string()
    }
}

struct SceneComponentCreator {}

impl ComponentCreator for SceneComponentCreator {
    fn create(
        &self,
        name: String,
        transformation: glam::Mat4,
    ) -> crate::error::Result<Box<dyn Component>> {
        Ok(Box::new(SceneComponent::new(name, transformation)))
    }

    fn name(&self) -> &'static str {
        type_name::<SceneComponent>()
    }

    fn display_name(&self) -> String {
        t!("Scene").to_string()
    }
}

struct PointLightComponentCreator {}

impl ComponentCreator for PointLightComponentCreator {
    fn create(
        &self,
        name: String,
        transformation: glam::Mat4,
    ) -> crate::error::Result<Box<dyn Component>> {
        Ok(Box::new(PointLightComponent::new(name, transformation)))
    }

    fn name(&self) -> &'static str {
        type_name::<PointLightComponent>()
    }

    fn display_name(&self) -> String {
        t!("Point Light").to_string()
    }
}

struct CollisionComponentCreator {}

impl ComponentCreator for CollisionComponentCreator {
    fn create(
        &self,
        name: String,
        transformation: glam::Mat4,
    ) -> crate::error::Result<Box<dyn Component>> {
        Ok(Box::new(CollisionComponent::new(name, transformation)))
    }

    fn name(&self) -> &'static str {
        type_name::<CollisionComponent>()
    }

    fn display_name(&self) -> String {
        t!("Collision").to_string()
    }
}

struct CameraComponentCreator {}

impl ComponentCreator for CameraComponentCreator {
    fn create(
        &self,
        name: String,
        transformation: glam::Mat4,
    ) -> crate::error::Result<Box<dyn Component>> {
        Ok(Box::new(CameraComponent::new(name, transformation)))
    }

    fn name(&self) -> &'static str {
        type_name::<CameraComponent>()
    }

    fn display_name(&self) -> String {
        t!("Camera").to_string()
    }
}

struct TextComponentCreator {}

impl ComponentCreator for TextComponentCreator {
    fn create(
        &self,
        name: String,
        transformation: glam::Mat4,
    ) -> crate::error::Result<Box<dyn Component>> {
        Ok(Box::new(TextComponent::new(name, transformation)))
    }

    fn name(&self) -> &'static str {
        type_name::<TextComponent>()
    }

    fn display_name(&self) -> String {
        t!("Text").to_string()
    }
}

pub struct ComponentFactory {
    creators: BTreeMap<String, Box<dyn ComponentCreator>>,
}

impl ComponentFactory {
    pub fn new() -> ComponentFactory {
        let creators = BTreeMap::new();
        let mut this = ComponentFactory { creators };
        let _ = this.register(Box::new(SpotLightComponentCreator {}));
        let _ = this.register(Box::new(StaticMeshComponentCreator {}));
        let _ = this.register(Box::new(SkeletonMeshComponentCreator {}));
        let _ = this.register(Box::new(SceneComponentCreator {}));
        let _ = this.register(Box::new(PointLightComponentCreator {}));
        let _ = this.register(Box::new(CollisionComponentCreator {}));
        let _ = this.register(Box::new(CameraComponentCreator {}));
        let _ = this.register(Box::new(TextComponentCreator {}));
        return this;
    }

    pub fn register(&mut self, component_creator: Box<dyn ComponentCreator>) -> anyhow::Result<()> {
        let component_type_name = component_creator.name().to_string();
        if self.creators.contains_key(&component_type_name) {
            return Err(anyhow::anyhow!("Already exists"));
        }

        self.creators.insert(component_type_name, component_creator);
        return Ok(());
    }

    pub fn creators(&self) -> &BTreeMap<String, Box<dyn ComponentCreator>> {
        &self.creators
    }
}
