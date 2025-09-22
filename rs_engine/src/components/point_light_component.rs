use crate::{
    content::level::LevelPhysics,
    drawable::{CustomDrawObject, EDrawObjectType},
    engine::Engine,
    player_viewport::PlayerViewport,
    scene_node::{EComponentType, SceneNode},
};
use rs_core_minimal::misc::point_light_radius;
use rs_foundation::new::{SingleThreadMut, SingleThreadMutType};
use rs_render::{
    command::{DrawObject, EBindingResource},
    constants,
    renderer::{EBuiltinPipelineType, EPipelineType},
    vertex_data_type::mesh_vertex::MeshVertex3,
};
use serde::{Deserialize, Serialize};
use std::num::NonZeroUsize;

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct PointLight {
    pub ambient: glam::Vec3,
    pub diffuse: glam::Vec3,
    pub specular: glam::Vec3,
    pub constant: f32,
    pub linear: f32,
    pub quadratic: f32,
}

impl Default for PointLight {
    fn default() -> Self {
        Self {
            ambient: glam::Vec3::ONE,
            diffuse: glam::Vec3::ONE,
            specular: glam::Vec3::ONE,
            constant: 1.0,
            linear: 0.09,
            quadratic: 0.032,
        }
    }
}

#[derive(Clone)]
pub struct PointLightComponentRuntime {
    pub parent_final_transformation: glam::Mat4,
    pub final_transformation: glam::Mat4,
    draw_object: EDrawObjectType,
    constants_handle: crate::handle::BufferHandle,
    constants: constants::Constants,
    pub is_show_preview: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PointLightComponent {
    pub name: String,
    pub transformation: glam::Mat4,
    pub is_visible: bool,
    pub point_light: PointLight,
    #[serde(skip)]
    pub run_time: Option<PointLightComponentRuntime>,
}

impl PointLightComponent {
    pub fn new(name: String, transformation: glam::Mat4) -> Self {
        Self {
            name,
            transformation,
            is_visible: true,
            run_time: None,
            point_light: PointLight::default(),
        }
    }

    pub fn new_scene_node(
        name: String,
        transformation: glam::Mat4,
    ) -> SingleThreadMutType<SceneNode> {
        let component = Self::new(name, transformation);
        let component = SingleThreadMut::new(component);
        let scene_node = SceneNode {
            component: EComponentType::PointLightComponent(component),
            childs: vec![],
        };
        let scene_node = SingleThreadMut::new(scene_node);
        scene_node
    }

    fn make_draw_object(
        engine: &mut Engine,
        player_viewport: &mut PlayerViewport,
        debug_group_label: String,
    ) -> (DrawObject, crate::handle::BufferHandle) {
        let sphere_data = rs_core_minimal::primitive_data::PrimitiveData::sphere(
            1.0,
            NonZeroUsize::new(16).unwrap(),
            NonZeroUsize::new(16).unwrap(),
            false,
        );

        let vertexes: Vec<MeshVertex3> = sphere_data
            .into_iter()
            .map(|x| MeshVertex3 {
                position: *x.1,
                vertex_color: rs_core_minimal::color::RED,
            })
            .collect();

        let vertex_count = vertexes.len();
        let vertex_buffer_handle =
            engine.create_vertex_buffer(&vertexes, Some(format!("rs.VertexBuffer")));
        let constants_handle = engine.create_constants_buffer(
            &vec![constants::Constants::default()],
            Some(format!("rs.Constants")),
        );
        let mut draw_object = DrawObject::new(
            0,
            vec![*vertex_buffer_handle],
            vertex_count as u32,
            EPipelineType::Builtin(EBuiltinPipelineType::Primitive(Some(
                wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    cull_mode: None,
                    polygon_mode: wgpu::PolygonMode::Line,
                    ..Default::default()
                },
            ))),
            None,
            None,
            vec![
                vec![EBindingResource::Constants(
                    *player_viewport.global_constants_handle,
                )],
                vec![EBindingResource::Constants(*constants_handle)],
            ],
        );
        draw_object.debug_group_label = Some(debug_group_label);
        (draw_object, constants_handle)
    }

    pub fn get_draw_objects(&self) -> Vec<&crate::drawable::EDrawObjectType> {
        let Some(run_time) = &self.run_time else {
            return vec![];
        };
        if !run_time.is_show_preview {
            return vec![];
        }
        vec![&run_time.draw_object]
    }

    pub fn get_radius(&self) -> f32 {
        point_light_radius(
            self.point_light.quadratic,
            self.point_light.linear,
            self.point_light.constant,
            0.0001,
        )
    }

    pub fn set_is_show_preview(&mut self, is_show_preview: bool) {
        if let Some(run_time) = &mut self.run_time {
            run_time.is_show_preview = is_show_preview;
        }
    }
}

impl super::component::Component for PointLightComponent {
    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn set_name(&mut self, new_name: String) {
        self.name = new_name;
    }

    fn get_final_transformation(&self) -> glam::Mat4 {
        let Some(run_time) = self.run_time.as_ref() else {
            return glam::Mat4::IDENTITY;
        };
        run_time.final_transformation
    }

    fn set_transformation(&mut self, transformation: glam::Mat4) {
        self.transformation = transformation;
    }

    fn get_transformation(&self) -> glam::Mat4 {
        self.transformation
    }

    fn on_post_update_transformation(&mut self, level_physics: Option<&mut LevelPhysics>) {
        let _ = level_physics;
    }

    fn set_final_transformation(&mut self, final_transformation: glam::Mat4) {
        let Some(run_time) = self.run_time.as_mut() else {
            return;
        };
        run_time.final_transformation = final_transformation;
    }

    fn set_parent_final_transformation(&mut self, parent_final_transformation: glam::Mat4) {
        let Some(run_time) = self.run_time.as_mut() else {
            return;
        };
        run_time.parent_final_transformation = parent_final_transformation;
    }

    fn get_parent_final_transformation(&self) -> glam::Mat4 {
        let Some(run_time) = self.run_time.as_ref() else {
            return glam::Mat4::IDENTITY;
        };
        run_time.parent_final_transformation
    }

    fn initialize(
        &mut self,
        engine: &mut crate::engine::Engine,
        files: &[crate::content::content_file_type::EContentFileType],
        player_viewport: &mut crate::player_viewport::PlayerViewport,
    ) {
        let _ = files;
        let (draw_object, constants_handle) = Self::make_draw_object(
            engine,
            player_viewport,
            format!("{} point light", &self.name),
        );
        let render_target_type = *player_viewport.get_render_target_type();
        self.run_time = Some(PointLightComponentRuntime {
            parent_final_transformation: glam::Mat4::IDENTITY,
            final_transformation: glam::Mat4::IDENTITY,
            draw_object: EDrawObjectType::Custom(CustomDrawObject {
                draw_object,
                render_target_type,
            }),
            constants_handle,
            constants: constants::Constants::default(),
            is_show_preview: true,
        })
    }

    fn initialize_physics(&mut self, level_physics: &mut LevelPhysics) {
        let _ = level_physics;
    }

    fn tick(
        &mut self,
        time: f32,
        engine: &mut crate::engine::Engine,
        level_physics: &mut LevelPhysics,
    ) {
        let _ = level_physics;
        let _ = engine;
        let _ = time;
        let radius = self.get_radius();
        let Some(run_time) = &mut self.run_time else {
            return;
        };
        let final_transformation = run_time.final_transformation;
        let (_, rotation, translation) = final_transformation.to_scale_rotation_translation();
        let final_transformation = glam::Mat4::from_scale_rotation_translation(
            glam::Vec3::splat(radius),
            rotation,
            translation,
        );
        run_time.constants.model = final_transformation;
        engine.update_buffer(
            run_time.constants_handle.clone(),
            rs_foundation::cast_any_as_u8_slice(&run_time.constants),
        );
    }
}
