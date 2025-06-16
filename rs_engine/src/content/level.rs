use super::content_file_type::EContentFileType;
use crate::actor::Actor;
use crate::camera_component::CameraComponent;
use crate::components::point_light_component::PointLightComponent;
use crate::components::spot_light_component::SpotLightComponent;
use crate::directional_light::DirectionalLight;
use crate::drawable::EDrawObjectType;
use crate::engine::Engine;
use crate::misc::{compute_appropriate_offset_look_and_projection_matrix, merge_aabb};
#[cfg(feature = "network")]
use crate::network::NetworkReplicated;
use crate::player_viewport::PlayerViewport;
use crate::scene_node::{EComponentType, SceneNode};
use crate::{build_content_file_url, url_extension::UrlExtension};
use rapier3d::prelude::*;
use rs_artifact::{asset::Asset, resource_type::EResourceType};
use rs_core_minimal::name_generator::make_unique_name;
use rs_foundation::new::{SingleThreadMut, SingleThreadMutType};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::VecDeque;
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
    pub event_handler: ChannelEventCollector,
    pub collision_recv: rapier3d::crossbeam::channel::Receiver<CollisionEvent>,
    pub contact_force_recv: rapier3d::crossbeam::channel::Receiver<ContactForceEvent>,
    pub collision_events: VecDeque<CollisionEvent>,
    pub contact_force_events: VecDeque<ContactForceEvent>,
}

impl Physics {
    pub fn step(&mut self) {
        let span = tracy_client::span!();
        span.emit_text(&format!(
            "collider len: {}, rigid body len: {}",
            self.collider_set.len(),
            self.rigid_body_set.len()
        ));
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

        while let Ok(collision_event) = self.collision_recv.try_recv() {
            self.collision_events.push_back(collision_event);
        }

        while let Ok(contact_force_event) = self.contact_force_recv.try_recv() {
            self.contact_force_events.push_back(contact_force_event);
        }
    }

    pub fn query_update(&mut self) {
        self.query_pipeline.update(&self.collider_set);
    }

    pub fn find_the_contact_pair(
        &self,
        collider_handle1: ColliderHandle,
        collider_handle2: ColliderHandle,
    ) -> Option<&ContactPair> {
        self.narrow_phase
            .contact_pair(collider_handle1, collider_handle2)
    }

    pub fn intersections_with_shape(
        &self,
        collider_handle: ColliderHandle,
        callback: impl FnMut(ColliderHandle) -> bool,
    ) {
        let collider = &self.collider_set[collider_handle];
        let shape = collider.shape();
        let shape_pos = collider.position();
        let filter = QueryFilter::default();
        self.query_pipeline.intersections_with_shape(
            &self.rigid_body_set,
            &self.collider_set,
            &shape_pos,
            shape,
            filter,
            callback,
        );
    }

    pub fn remove_colliders(&mut self, collider_handle: ColliderHandle) {
        if collider_handle == ColliderHandle::invalid() {
            return;
        }
        self.collider_set.remove(
            collider_handle,
            &mut self.island_manager,
            &mut self.rigid_body_set,
            true,
        );
    }

    pub fn remove_rigid_body(&mut self, rigid_body_handle: RigidBodyHandle) {
        if rigid_body_handle == RigidBodyHandle::invalid() {
            return;
        }
        self.rigid_body_set.remove(
            rigid_body_handle,
            &mut self.island_manager,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            true,
        );
    }
}

pub struct Runtime {
    pub physics: Physics,
    pub is_simulate: bool,
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
            runtime: Some(Runtime {
                physics: Self::default_physics(),
                is_simulate: false,
            }),
        }
    }

    pub fn get_name(&self) -> String {
        self.url.get_name_in_editor()
    }

    fn default_physics() -> Physics {
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
        let (collision_send, collision_recv) = rapier3d::crossbeam::channel::unbounded();
        let (contact_force_send, contact_force_recv) = rapier3d::crossbeam::channel::unbounded();
        let event_handler = ChannelEventCollector::new(collision_send, contact_force_send);

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
            collision_recv,
            contact_force_recv,
            collision_events: VecDeque::new(),
            contact_force_events: VecDeque::new(),
        };
        physics
    }

    pub fn initialize(
        &mut self,
        engine: &mut Engine,
        files: &[EContentFileType],
        player_viewport: &mut PlayerViewport,
    ) {
        for light in self.directional_lights.iter_mut() {
            let mut light = light.borrow_mut();
            light.initialize(engine, player_viewport);
        }

        // let rigid_body_set: RigidBodySet = RigidBodySet::new();
        // let collider_set: ColliderSet = ColliderSet::new();

        // let gravity: nalgebra::Vector3<f32> = vector![0.0, -9.81, 0.0];
        // let integration_parameters: IntegrationParameters = IntegrationParameters::default();
        // let physics_pipeline: PhysicsPipeline = PhysicsPipeline::new();
        // let island_manager: IslandManager = IslandManager::new();
        // let broad_phase: BroadPhaseMultiSap = DefaultBroadPhase::new();
        // let narrow_phase: NarrowPhase = NarrowPhase::new();
        // let impulse_joint_set: ImpulseJointSet = ImpulseJointSet::new();
        // let multibody_joint_set: MultibodyJointSet = MultibodyJointSet::new();
        // let ccd_solver: CCDSolver = CCDSolver::new();
        // let query_pipeline: QueryPipeline = QueryPipeline::new();
        // let physics_hooks: () = ();
        // let (collision_send, collision_recv) = rapier3d::crossbeam::channel::unbounded();
        // let (contact_force_send, contact_force_recv) = rapier3d::crossbeam::channel::unbounded();
        // let event_handler = ChannelEventCollector::new(collision_send, contact_force_send);

        // let physics = Physics {
        //     rigid_body_set,
        //     collider_set,
        //     gravity,
        //     integration_parameters,
        //     physics_pipeline,
        //     island_manager,
        //     broad_phase,
        //     narrow_phase,
        //     impulse_joint_set,
        //     multibody_joint_set,
        //     ccd_solver,
        //     query_pipeline,
        //     physics_hooks,
        //     event_handler,
        //     collision_recv,
        //     contact_force_recv,
        //     collision_events: VecDeque::new(),
        //     contact_force_events: VecDeque::new(),
        // };
        self.runtime = Some(Runtime {
            physics: Self::default_physics(),
            is_simulate: false,
        });
        let actors = self.actors.clone();
        self.init_actors(engine, actors, files, player_viewport);
        let actors = self.actors.clone();
        for actor in actors {
            self.init_actor_physics(actor.clone());
        }
    }

    pub fn init_actors(
        &mut self,
        engine: &mut crate::engine::Engine,
        actors: Vec<SingleThreadMutType<crate::actor::Actor>>,
        files: &[EContentFileType],
        player_viewport: &mut PlayerViewport,
    ) {
        for actor in actors {
            let mut actor = actor.borrow_mut();
            actor.initialize(engine, files, player_viewport);
        }
    }

    pub fn init_actor_physics(&mut self, actor: SingleThreadMutType<Actor>) {
        let Some(physics) = self.get_physics_mut() else {
            return;
        };
        let rigid_body_set = &mut physics.rigid_body_set;
        let collider_set = &mut physics.collider_set;
        let mut actor = actor.borrow_mut();
        actor.initialize_physics(rigid_body_set, collider_set);
    }

    // pub fn update_actor_physics(&mut self, actor: SingleThreadMutType<Actor>) {
    //     let Some(physics) = self.get_physics_mut() else {
    //         return;
    //     };
    //     let rigid_body_set = &mut physics.rigid_body_set;
    //     let collider_set = &mut physics.collider_set;
    //     let mut actor = actor.borrow_mut();
    //     actor.tick_physics(rigid_body_set, collider_set);
    // }

    pub fn tick(&mut self, time: f32, engine: &mut Engine, player_viewport: &mut PlayerViewport) {
        for light in self.directional_lights.clone() {
            let mut light = light.borrow_mut();
            light.update(engine);
            // player_viewport.update_light(&mut light);
        }
        {
            if let Some(offset_look_and_projection_matrix) =
                compute_appropriate_offset_look_and_projection_matrix(self)
            {
                player_viewport.update_light_concentrate_scene(
                    offset_look_and_projection_matrix,
                    self.directional_lights.clone(),
                );
            }
        }

        let Some(runtime) = self.runtime.as_mut() else {
            return;
        };
        if runtime.is_simulate {
            runtime.physics.step();
        } else {
            runtime.physics.query_update();
        }
        let rigid_body_set = &mut runtime.physics.rigid_body_set;
        let collider_set = &mut runtime.physics.collider_set;
        for actor in self.actors.clone() {
            let mut actor = actor.borrow_mut();
            actor.tick(time, engine, rigid_body_set, collider_set);
            // actor.tick_physics(rigid_body_set, collider_set);
        }

        let light_components = self.collect_point_light_components();
        player_viewport.update_point_lights(engine, light_components);
        let spot_light_components = self.collect_spot_light_components();
        player_viewport.update_spot_lights(spot_light_components);
    }

    pub fn get_rigid_body_set_mut(&mut self) -> Option<&mut RigidBodySet> {
        self.runtime.as_mut().map(|x| &mut x.physics.rigid_body_set)
    }

    pub fn set_physics_simulate(&mut self, enable: bool) {
        let Some(runtime) = self.runtime.as_mut() else {
            return;
        };
        runtime.is_simulate = enable;
    }

    pub fn get_physics_mut(&mut self) -> Option<&mut Physics> {
        self.runtime.as_mut().map(|x| &mut x.physics)
    }

    // #[cfg(feature = "editor")]
    pub fn make_copy_for_standalone(
        &self,
        engine: &mut Engine,
        files: &[EContentFileType],
        player_viewport: &mut PlayerViewport,
    ) -> Level {
        let ser_level = serde_json::to_string(self).unwrap();
        let mut copy_level: Level = serde_json::from_str(&ser_level).unwrap();
        copy_level.initialize(engine, files, player_viewport);
        copy_level
    }

    pub fn physics_step(&mut self) {
        let Some(runtime) = self.runtime.as_mut() else {
            return;
        };
        runtime.physics.step();
    }

    pub fn make_actor_name(&self, new_name: &str) -> String {
        let names = self
            .actors
            .iter()
            .map(|x| x.borrow().name.clone())
            .collect();
        let name = make_unique_name(names, new_name);
        name
    }

    pub fn create_and_insert_actor(&mut self) -> SingleThreadMutType<crate::actor::Actor> {
        let name = self.make_actor_name("Actor");
        let new_actor = Actor::new_sp(name);
        self.actors.push(new_actor.clone());
        new_actor
    }

    pub fn add_new_actors(
        &mut self,
        engine: &mut crate::engine::Engine,
        mut actors: Vec<SingleThreadMutType<crate::actor::Actor>>,
        files: &[EContentFileType],
        player_viewport: &mut PlayerViewport,
    ) {
        self.init_actors(engine, actors.clone(), files, player_viewport);

        for actor in actors.clone() {
            self.init_actor_physics(actor.clone());
        }

        self.actors.append(&mut actors);
    }

    pub fn ray_cast_find_node(
        &self,
        cursor_position: &glam::Vec2,
        window_size: &glam::Vec2,
        // camera: &mut Camera,
        camera_view_matrix: glam::Mat4,
        camera_projection_matrix: glam::Mat4,
    ) -> Option<SingleThreadMutType<SceneNode>> {
        let Some(physics) = self.runtime.as_ref().map(|x| &x.physics) else {
            return None;
        };
        let ndc_cursor = glam::vec2(
            cursor_position.x / window_size.x * 2.0 - 1.0,
            1.0 - cursor_position.y / window_size.y * 2.0,
        );
        let ndc_to_world = camera_projection_matrix * camera_view_matrix;
        let ndc_to_world = ndc_to_world.inverse();
        let ray_pt1 = ndc_to_world.project_point3(glam::vec3(ndc_cursor.x, ndc_cursor.y, 0.0));
        let ray_pt2 = ndc_to_world.project_point3(glam::vec3(ndc_cursor.x, ndc_cursor.y, 1.0));
        let ray_dir = ray_pt2 - ray_pt1;
        let ray_origin = rapier3d::na::Point3::new(ray_pt1.x, ray_pt1.y, ray_pt1.z);
        let ray_dir = rapier3d::na::Vector3::new(ray_dir.x, ray_dir.y, ray_dir.z);
        let ray = rapier3d::prelude::Ray::new(ray_origin, ray_dir);
        let hit = physics.query_pipeline.cast_ray(
            &physics.rigid_body_set,
            &physics.collider_set,
            &ray,
            f32::MAX,
            true,
            QueryFilter::new(),
        );
        if let Some((handle, _)) = hit {
            let mut search_node: Option<SingleThreadMutType<SceneNode>> = None;
            for actor in self.actors.clone() {
                let actor = actor.borrow_mut();
                self.find_node(actor.scene_node.clone(), handle, &mut search_node);
            }
            return search_node;
        }

        return None;
    }

    pub fn find_node(
        &self,
        scene_node: SingleThreadMutType<SceneNode>,
        handle: ColliderHandle,
        search_node: &mut Option<SingleThreadMutType<SceneNode>>,
    ) {
        if search_node.is_some() {
            return;
        }
        let scene_node_clone = scene_node.clone();

        let scene_node = scene_node.borrow();
        match &scene_node.component {
            EComponentType::SceneComponent(_) => {}
            EComponentType::StaticMeshComponent(static_mesh_component) => {
                let is_find = (|| {
                    let mut component = static_mesh_component.borrow_mut();
                    if let Some(physics) = component.get_physics_mut() {
                        if physics.get_collider_handles().contains(&handle) {
                            return true;
                        }
                    }
                    false
                })();
                if is_find {
                    *search_node = Some(scene_node_clone);
                    return;
                }
            }
            EComponentType::SkeletonMeshComponent(static_mesh_component) => {
                let is_find = (|| {
                    let mut component = static_mesh_component.borrow_mut();
                    if let Some(physics) = component.get_physics_mut() {
                        if physics.get_collider_handles().contains(&handle) {
                            return true;
                        }
                    }
                    false
                })();
                if is_find {
                    *search_node = Some(scene_node_clone);
                    return;
                }
            }
            EComponentType::CameraComponent(_) => {}
            EComponentType::CollisionComponent(collision_component) => {
                let is_find = (|| {
                    let mut component = collision_component.borrow_mut();
                    if let Some(physics) = component.get_physics_mut() {
                        if physics.get_collider_handles().contains(&handle) {
                            return true;
                        }
                    }
                    false
                })();
                if is_find {
                    *search_node = Some(scene_node_clone);
                    return;
                }
            }
            EComponentType::SpotLightComponent(_) => {}
            EComponentType::PointLightComponent(_) => {}
        }
        for child in scene_node.childs.clone() {
            self.find_node(child, handle, search_node);
        }
    }

    pub fn collect_draw_objects(&self) -> Vec<EDrawObjectType> {
        let mut draw_objects = vec![];
        for light in self.directional_lights.clone() {
            let light = light.borrow_mut();
            let mut sub_draw_objects = light
                .get_draw_objects()
                .iter()
                .map(|x| (*x).clone())
                .collect();
            draw_objects.append(&mut sub_draw_objects);
        }
        for actor in self.actors.clone() {
            let actor = actor.borrow_mut();
            let mut sub_draw_objects = actor.collect_draw_objects();
            draw_objects.append(&mut sub_draw_objects);
        }
        draw_objects
    }

    pub fn collect_camera_componenets(&self) -> Vec<SingleThreadMutType<CameraComponent>> {
        let mut camera_componenets = vec![];
        for actor in self.actors.clone() {
            let actor = actor.borrow_mut();
            Actor::walk_node_mut(actor.scene_node.clone(), &mut |node| {
                let node = node.borrow();
                if let EComponentType::CameraComponent(rc) = &node.component {
                    camera_componenets.push(rc.clone());
                }
            });
        }
        camera_componenets
    }

    pub fn delete_light(&mut self, light: SingleThreadMutType<DirectionalLight>) {
        self.directional_lights
            .retain(|element| !Rc::ptr_eq(&element, &light));
    }

    pub fn delete_actor(&mut self, actor: SingleThreadMutType<Actor>) {
        let delete_actors = self
            .actors
            .iter()
            .filter_map(|x| {
                if Rc::ptr_eq(&x, &actor) {
                    Some(actor.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<SingleThreadMutType<Actor>>>();
        if let Some(level_physics) = self.get_physics_mut() {
            for delete_actor in delete_actors {
                let delete_actor = delete_actor.borrow();
                Self::remove_actor_physics(level_physics, &delete_actor);
            }
        }
        self.actors.retain(|element| !Rc::ptr_eq(&element, &actor));
    }

    fn remove_actor_physics(level_physics: &mut Physics, actor: &Actor) {
        let node = actor.scene_node.clone();
        Actor::walk_node_mut(node, &mut |node| {
            let node = node.borrow();
            match &node.component {
                EComponentType::SceneComponent(_) => {}
                EComponentType::StaticMeshComponent(component) => {
                    let component = component.borrow();
                    if let Some(component_physics) = component.get_physics() {
                        level_physics.remove_rigid_body(component_physics.rigid_body_handle);
                    }
                }
                EComponentType::SkeletonMeshComponent(component) => {
                    let component = component.borrow();
                    if let Some(component_physics) = component.get_physics() {
                        level_physics.remove_rigid_body(component_physics.rigid_body_handle);
                    }
                }
                EComponentType::CameraComponent(_) => {}
                EComponentType::CollisionComponent(component) => {
                    let component = component.borrow();
                    if let Some(component_physics) = component.get_physics() {
                        level_physics.remove_rigid_body(component_physics.rigid_body_handle);
                    }
                }
                EComponentType::SpotLightComponent(_) => {}
                EComponentType::PointLightComponent(_) => {}
            }
        });
    }

    pub fn find_actor(&self, name: &str) -> Option<SingleThreadMutType<Actor>> {
        self.actors
            .iter()
            .find(|x| x.borrow().name == name)
            .cloned()
    }

    pub fn find_actor_by_collider_handle(
        &self,
        collider: &rapier3d::prelude::ColliderHandle,
    ) -> Option<(SingleThreadMutType<Actor>, SingleThreadMutType<SceneNode>)> {
        for actor in self.actors.clone() {
            let node = {
                let actor = actor.borrow();
                actor.find_node_by_collider_handle(collider)
            };
            if let Some(node) = node {
                return Some((actor, node));
            }
        }
        return None;
    }

    pub fn compute_scene_aabb(&self) -> Option<rapier3d::prelude::Aabb> {
        let mut aabbs: Vec<rapier3d::prelude::Aabb> = vec![];
        for actor in self.actors.clone() {
            let actor = actor.borrow();
            if let Some(aabb) = actor.compute_components_aabb() {
                aabbs.push(aabb);
            }
        }
        merge_aabb(&aabbs)
    }

    pub fn duplicate_actor(
        &mut self,
        actor: SingleThreadMutType<Actor>,
        engine: &mut crate::engine::Engine,
        files: &[EContentFileType],
        player_viewport: &mut PlayerViewport,
    ) {
        let actor = actor.borrow();
        let name = self.make_actor_name(&actor.name);
        let duplicated_actor = SingleThreadMut::new(actor.copy_without_initialization(name));
        self.add_new_actors(engine, vec![duplicated_actor], files, player_viewport);
    }

    pub fn collect_point_light_components(&self) -> Vec<SingleThreadMutType<PointLightComponent>> {
        let mut lights = vec![];
        for actor in self.actors.clone() {
            let actor = actor.borrow();
            let scene_node = actor.scene_node.clone();
            Actor::walk_node_mut(scene_node, &mut |node| {
                let node = node.borrow();
                match &node.component {
                    EComponentType::PointLightComponent(component) => {
                        lights.push(component.clone());
                    }
                    _ => {}
                }
            });
        }
        lights
    }

    pub fn collect_spot_light_components(&self) -> Vec<SingleThreadMutType<SpotLightComponent>> {
        let mut lights = vec![];
        for actor in self.actors.clone() {
            let actor = actor.borrow();
            let scene_node = actor.scene_node.clone();
            Actor::walk_node_mut(scene_node, &mut |node| {
                let node = node.borrow();
                match &node.component {
                    EComponentType::SpotLightComponent(component) => {
                        lights.push(component.clone());
                    }
                    _ => {}
                }
            });
        }
        lights
    }

    pub fn set_debug_show_flag(&mut self, flag: crate::debug_show_flag::DebugShowFlag) {
        for actor in self.actors.clone() {
            let actor = actor.borrow_mut();
            let scene_node = actor.scene_node.clone();
            Actor::walk_node_mut(scene_node, &mut |node| {
                let mut node = node.borrow_mut();
                match &mut node.component {
                    EComponentType::PointLightComponent(component) => {
                        let mut component = component.borrow_mut();
                        component.set_is_show_preview(
                            flag.contains(crate::debug_show_flag::DebugShowFlag::PointLightSphere),
                        );
                    }
                    EComponentType::CameraComponent(component) => {
                        let mut component = component.borrow_mut();
                        component.set_is_show_preview(
                            flag.contains(crate::debug_show_flag::DebugShowFlag::CameraFrustum),
                        );
                    }
                    _ => {}
                }
            });
        }
    }

    #[cfg(feature = "network")]
    pub fn visit_network_replicated_mut(
        &mut self,
        visit: &mut impl FnMut(&mut dyn NetworkReplicated),
    ) {
        for actor in self.actors.clone() {
            let mut actor = actor.borrow_mut();
            actor.visit_network_replicated_mut(visit);
        }
    }

    #[cfg(feature = "network")]
    pub fn visit_network_replicated(&self, visit: &impl Fn(&dyn NetworkReplicated)) {
        for actor in self.actors.clone() {
            let actor = actor.borrow();
            actor.visit_network_replicated(visit);
        }
    }
}
