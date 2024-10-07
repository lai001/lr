use crate::{
    camera_component::CameraComponent, skeleton_mesh_component::SkeletonMeshComponent,
    static_mesh_component::StaticMeshComponent,
};
use rs_foundation::new::{SingleThreadMut, SingleThreadMutType};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
struct SceneComponentRuntime {
    pub parent_final_transformation: glam::Mat4,
    pub final_transformation: glam::Mat4,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SceneComponent {
    pub name: String,
    pub transformation: glam::Mat4,
    #[serde(skip)]
    run_time: Option<SceneComponentRuntime>,
}

impl SceneComponent {
    pub fn new(name: String, transformation: glam::Mat4) -> SceneComponent {
        SceneComponent {
            name,
            transformation,
            run_time: Some(SceneComponentRuntime {
                final_transformation: glam::Mat4::IDENTITY,
                parent_final_transformation: glam::Mat4::IDENTITY,
            }),
        }
    }

    pub fn initialize(&mut self) {
        self.run_time = Some(SceneComponentRuntime {
            final_transformation: glam::Mat4::IDENTITY,
            parent_final_transformation: glam::Mat4::IDENTITY,
        });
    }

    pub fn set_parent_final_transformation(&mut self, parent_final_transformation: glam::Mat4) {
        let Some(run_time) = self.run_time.as_mut() else {
            return;
        };
        run_time.parent_final_transformation = parent_final_transformation;
    }

    pub fn get_parent_final_transformation(&self) -> glam::Mat4 {
        let Some(run_time) = self.run_time.as_ref() else {
            return glam::Mat4::IDENTITY;
        };
        run_time.parent_final_transformation
    }

    pub fn get_transformation_mut(&mut self) -> &mut glam::Mat4 {
        &mut self.transformation
    }

    pub fn get_transformation(&self) -> &glam::Mat4 {
        &self.transformation
    }

    pub fn set_final_transformation(&mut self, final_transformation: glam::Mat4) {
        let Some(run_time) = self.run_time.as_mut() else {
            return;
        };
        run_time.final_transformation = final_transformation;
    }

    pub fn get_final_transformation(&self) -> glam::Mat4 {
        let final_transformation = self
            .run_time
            .as_ref()
            .map(|x| x.final_transformation)
            .unwrap_or_default();
        final_transformation
    }

    pub fn on_post_update_transformation(
        &mut self,
        level_physics: Option<&mut crate::content::level::Physics>,
    ) {
        let _ = level_physics;
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum EComponentType {
    SceneComponent(SingleThreadMutType<SceneComponent>),
    StaticMeshComponent(SingleThreadMutType<StaticMeshComponent>),
    SkeletonMeshComponent(SingleThreadMutType<SkeletonMeshComponent>),
    CameraComponent(SingleThreadMutType<CameraComponent>),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SceneNode {
    pub component: EComponentType,
    pub childs: Vec<SingleThreadMutType<SceneNode>>,
}

macro_rules! common_fn {
    ($($x:tt),*) => {
        pub fn get_name(&self) -> String {
            match &self.component {
                $(
                    EComponentType::$x(component) => {
                        let component = component.borrow();
                        component.name.clone()
                    }
                )*
            }
        }

        pub fn get_final_transformation(&self) -> glam::Mat4 {
            match &self.component {
                $(
                    EComponentType::$x(component) => {
                        let component = component.borrow();
                        component.get_final_transformation()
                    }
                )*
            }
        }


        pub fn set_transformation(&mut self, transformation: glam::Mat4) {
            match &mut self.component {
                $(
                    EComponentType::$x(component) => {
                        let mut component = component.borrow_mut();
                        component.transformation = transformation;
                    }
                )*
            }
        }

        pub fn get_transformation(&self) -> glam::Mat4 {
            match &self.component {
                $(
                    EComponentType::$x(component) => {
                        let component = component.borrow();
                        component.transformation
                    }
                )*
            }
        }

        pub fn on_post_update_transformation(&mut self, level_physics: Option<&mut crate::content::level::Physics>) {
            match &mut self.component {
                $(
                    EComponentType::$x(component) => {
                        let mut component = component.borrow_mut();
                        component.on_post_update_transformation(level_physics);
                    }
                )*
            }
        }
    };
}

impl SceneNode {
    pub fn new(name: String) -> SceneNode {
        SceneNode {
            component: EComponentType::SceneComponent(SingleThreadMut::new(SceneComponent::new(
                name,
                glam::Mat4::IDENTITY,
            ))),
            childs: vec![],
        }
    }

    pub fn new_sp(name: String) -> SingleThreadMutType<SceneNode> {
        SingleThreadMut::new(Self::new(name))
    }

    common_fn!(
        SceneComponent,
        StaticMeshComponent,
        SkeletonMeshComponent,
        CameraComponent
    );
}
