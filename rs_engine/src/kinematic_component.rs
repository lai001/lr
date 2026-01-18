use crate::content::level;
use rapier3d::{control::KinematicCharacterController, prelude::*};

pub struct KinematicComponent {
    pub character_body: RigidBodyHandle,
    pub speed: f32,
    pub rigid_body: RigidBody,
}

impl KinematicComponent {
    pub fn from_builder(builder: RigidBodyBuilder) -> KinematicComponent {
        let rigid_body = builder.build();
        KinematicComponent {
            character_body: RigidBodyHandle::invalid(),
            speed: 0.1,
            rigid_body,
        }
    }

    pub fn get_location(&self) -> glam::Vec3 {
        let translation = self.rigid_body.position().translation;
        glam::vec3(translation.x, translation.y, translation.z)
    }

    pub fn init_physics(
        &mut self,
        rigid_body_set: &mut RigidBodySet,
        collider_set: &mut ColliderSet,
    ) {
        let _ = collider_set;
        self.character_body = rigid_body_set.insert(self.rigid_body.clone());
    }

    pub fn update(&mut self, desired_movement: &glam::Vec3, physics: &mut level::LevelPhysics) {
        let character_handle = self.character_body;
        let mut desired_movement = *desired_movement;

        desired_movement *= self.speed;
        desired_movement -= glam::Vec3::Y * self.speed;

        let controller = KinematicCharacterController::default();
        let character_body = &physics.rigid_body_set[character_handle];
        let character_collider = physics.collider_set[character_body.colliders()[0]].clone();
        let character_mass = character_body.mass();

        let mut collisions = vec![];
        let mvt = controller.move_shape(
            physics.integration_parameters.dt,
            &physics.query_pipeline(Some(
                QueryFilter::new().exclude_rigid_body(character_handle),
            )),
            character_collider.shape(),
            character_collider.position(),
            desired_movement,
            |c| collisions.push(c),
        );
        controller.solve_character_collision_impulses(
            physics.integration_parameters.dt,
            &mut physics.query_pipeline_mut(Some(
                QueryFilter::new().exclude_rigid_body(character_handle),
            )),
            character_collider.shape(),
            character_mass,
            &*collisions,
        );
        let character_body = &mut physics.rigid_body_set[character_handle];
        let pos = character_body.position();
        character_body.set_next_kinematic_translation(pos.translation + mvt.translation);
    }
}
