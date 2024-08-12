use crate::actor::Actor;
use crate::directional_light::DirectionalLight;
use crate::engine::Engine;
use crate::{build_content_file_url, url_extension::UrlExtension};
use rapier3d::prelude::*;
use rs_artifact::{asset::Asset, resource_type::EResourceType};
use rs_foundation::new::SingleThreadMutType;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;

pub struct Physics {
    pub rigid_body_set: RigidBodySet,
    pub collider_set: ColliderSet,
    pub gravity: nalgebra::Vector3<f32>,
    pub integration_parameters: IntegrationParameters,
    pub physics_pipeline: PhysicsPipeline,
    pub island_manager: IslandManager,
    pub broad_phase: BroadPhaseMultiSap,
    pub narrow_phase: NarrowPhase,
    pub impulse_joint_set: ImpulseJointSet,
    pub multibody_joint_set: MultibodyJointSet,
    pub ccd_solver: CCDSolver,
    pub query_pipeline: QueryPipeline,
    pub physics_hooks: (),
    pub event_handler: (),
    pub is_simulate: bool,
}

impl Physics {
    pub fn step(&mut self) {
        if !self.is_simulate {
            return;
        }
        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            Some(&mut self.query_pipeline),
            &self.physics_hooks,
            &self.event_handler,
        );
    }
}

pub struct Runtime {
    pub physics: Physics,
}

#[derive(Serialize, Deserialize)]
pub struct Level {
    pub url: url::Url,
    pub actors: Vec<Rc<RefCell<crate::actor::Actor>>>,
    pub directional_lights: Vec<SingleThreadMutType<DirectionalLight>>,

    #[serde(skip)]
    runtime: Option<Runtime>,
}

impl Asset for Level {
    fn get_url(&self) -> url::Url {
        self.url.clone()
    }

    fn get_resource_type(&self) -> EResourceType {
        EResourceType::Content(rs_artifact::content_type::EContentType::Level)
    }
}

impl Level {
    pub fn empty_level() -> Self {
        Self {
            actors: vec![],
            url: build_content_file_url("Empty").unwrap(),
            directional_lights: vec![],
            runtime: None,
        }
    }

    pub fn get_name(&self) -> String {
        self.url.get_name_in_editor()
    }

    pub fn initialize(&mut self, engine: &mut Engine) {
        for light in self.directional_lights.iter_mut() {
            let mut light = light.borrow_mut();
            light.initialize(engine);
        }

        let rigid_body_set: RigidBodySet = RigidBodySet::new();
        let collider_set: ColliderSet = ColliderSet::new();

        let gravity: nalgebra::Vector3<f32> = vector![0.0, -9.81, 0.0];
        let integration_parameters: IntegrationParameters = IntegrationParameters::default();
        let physics_pipeline: PhysicsPipeline = PhysicsPipeline::new();
        let island_manager: IslandManager = IslandManager::new();
        let broad_phase: BroadPhaseMultiSap = DefaultBroadPhase::new();
        let narrow_phase: NarrowPhase = NarrowPhase::new();
        let impulse_joint_set: ImpulseJointSet = ImpulseJointSet::new();
        let multibody_joint_set: MultibodyJointSet = MultibodyJointSet::new();
        let ccd_solver: CCDSolver = CCDSolver::new();
        let query_pipeline: QueryPipeline = QueryPipeline::new();
        let physics_hooks: () = ();
        let event_handler: () = ();

        let physics = Physics {
            rigid_body_set,
            collider_set,
            gravity,
            integration_parameters,
            physics_pipeline,
            island_manager,
            broad_phase,
            narrow_phase,
            impulse_joint_set,
            multibody_joint_set,
            ccd_solver,
            query_pipeline,
            physics_hooks,
            event_handler,
            is_simulate: false,
        };
        self.runtime = Some(Runtime { physics });
        let actors = self.actors.clone();
        for actor in actors {
            self.init_actor_physics(actor.clone());
        }
    }

    pub fn init_actor_physics(&mut self, actor: SingleThreadMutType<Actor>) {
        let physics = self.get_physics_mut().unwrap();
        let rigid_body_set = &mut physics.rigid_body_set;
        let collider_set = &mut physics.collider_set;
        let actor = actor.borrow_mut();
        let mut scene_node = actor.scene_node.borrow_mut();
        for child_scene_node in scene_node.childs.iter_mut() {
            let mut child_scene_node = child_scene_node.borrow_mut();
            match &mut child_scene_node.component {
                crate::scene_node::EComponentType::SceneComponent(_) => {}
                crate::scene_node::EComponentType::StaticMeshComponent(component) => {
                    let mut component = component.borrow_mut();
                    component.init_physics(rigid_body_set, collider_set);
                }
                crate::scene_node::EComponentType::SkeletonMeshComponent(_) => {}
            }
        }
        match &mut scene_node.component {
            crate::scene_node::EComponentType::SceneComponent(_) => {}
            crate::scene_node::EComponentType::StaticMeshComponent(component) => {
                let mut component = component.borrow_mut();
                component.init_physics(rigid_body_set, collider_set);
            }
            crate::scene_node::EComponentType::SkeletonMeshComponent(_) => {}
        }
    }

    pub fn tick(&mut self) {
        let Some(physics) = self.runtime.as_mut().map(|x| &mut x.physics) else {
            return;
        };
        physics.step();
    }

    pub fn get_rigid_body_set_mut(&mut self) -> Option<&mut RigidBodySet> {
        self.runtime.as_mut().map(|x| &mut x.physics.rigid_body_set)
    }

    pub fn set_physics_simulate(&mut self, enable: bool) {
        let Some(physics) = self.runtime.as_mut().map(|x| &mut x.physics) else {
            return;
        };
        physics.is_simulate = enable;
    }

    pub fn get_physics_mut(&mut self) -> Option<&mut Physics> {
        self.runtime.as_mut().map(|x| &mut x.physics)
    }
}
