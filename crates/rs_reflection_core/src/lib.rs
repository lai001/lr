use downcast_rs::Downcast;
use dyn_clone::DynClone;

#[derive(Clone)]
pub struct TypeMeta {
    pub name: String,
}

#[derive(Clone)]
pub struct ParamMeta {
    pub name: Option<String>,
    pub type_meta: TypeMeta,
}

#[derive(Clone)]
pub struct FunctionMeta {
    pub name: String,
    pub params: Vec<ParamMeta>,
    pub return_ty: TypeMeta,
}

#[derive(Clone)]
pub struct StructFieldMeta {
    pub name: String,
    pub type_meta: TypeMeta,
}

#[derive(Clone)]
pub struct StructMeta {
    pub name: String,
    pub fields: Vec<StructFieldMeta>,
    pub functions: Vec<FunctionMeta>,
}

pub trait StructMetaContainer: DynClone + Downcast {
    fn exec_without_self(
        &mut self,
        name: &str,
        params: Vec<Box<dyn std::any::Any>>,
    ) -> Option<Box<dyn std::any::Any>>;

    fn exec_with_mut_self(
        &mut self,
        name: &str,
        self_param: &mut dyn std::any::Any,
        params: Vec<Box<dyn std::any::Any>>,
    ) -> Option<Box<dyn std::any::Any>>;

    fn exec_with_self(
        &mut self,
        name: &str,
        self_param: &dyn std::any::Any,
        params: Vec<Box<dyn std::any::Any>>,
    ) -> Option<Box<dyn std::any::Any>>;

    fn get_struct_meta(&self) -> &StructMeta;
}
dyn_clone::clone_trait_object!(StructMetaContainer);
