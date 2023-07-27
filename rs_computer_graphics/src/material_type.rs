use crate::{material::Material, pbr_material::PBRMaterial};

pub enum EMaterialType {
    Phong(Material),
    Pbr(PBRMaterial),
}
