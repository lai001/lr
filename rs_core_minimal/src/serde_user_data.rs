use downcast_rs::{Downcast, impl_downcast};
use dyn_clone::{DynClone, clone_trait_object};
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
