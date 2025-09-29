use downcast_rs::{impl_downcast, Downcast};
use dyn_clone::{clone_trait_object, DynClone};
use serde::{Deserialize, Serialize};

#[typetag::serde]
pub trait SerdeUserValueTrait: DynClone + Downcast {}

impl_downcast!(SerdeUserValueTrait);

clone_trait_object!(SerdeUserValueTrait);

#[derive(Serialize, Deserialize, Clone)]
pub struct SerdeUserData {
    pub value: Box<dyn SerdeUserValueTrait>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TextValue {
    pub text: String,
}

#[typetag::serde]
impl SerdeUserValueTrait for TextValue {}
