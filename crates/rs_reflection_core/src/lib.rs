pub mod error;

use downcast_rs::Downcast;
use std::any::{Any, TypeId};

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

pub enum ReflectArg<'a> {
    Owned(Box<dyn Any>),
    Ref(&'a dyn Any),
    MutRef(&'a mut dyn Any),
    OptionRef(Option<&'a dyn Any>),
    OptionMutRef(Option<&'a mut dyn Any>),
}

pub enum FunctionExecType {
    Exec(
        Box<
            dyn for<'a> Fn(
                &'a dyn Any,
                &'a mut Vec<ReflectArg<'a>>,
            ) -> crate::error::Result<Option<ReflectArg<'a>>>,
        >,
    ),
    ExecMut(
        Box<
            dyn for<'a> Fn(
                &'a mut dyn Any,
                &'a mut Vec<ReflectArg<'a>>,
            ) -> crate::error::Result<Option<ReflectArg<'a>>>,
        >,
    ),
    StaticExec(
        Box<dyn Fn(&mut Vec<ReflectArg>) -> crate::error::Result<Option<ReflectArg<'static>>>>,
    ),
}

pub struct Function {
    pub meta: FunctionMeta,
    pub exec_type: FunctionExecType,
}

pub struct StructMeta {
    pub name: String,
    pub fields: Vec<StructFieldMeta>,
    pub functions: Vec<Function>,
}

impl StructMeta {
    pub fn new(name: String, fields: Vec<StructFieldMeta>, functions: Vec<Function>) -> StructMeta {
        StructMeta {
            name,
            fields,
            functions,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn fields(&self) -> &[StructFieldMeta] {
        &self.fields
    }

    pub fn functions(&self) -> &Vec<Function> {
        &self.functions
    }

    pub fn find_function_by_name(&self, name: &str) -> Option<&Function> {
        self.functions.iter().find(|x| x.meta.name == name)
    }
}

pub fn get_type_id(this: &(dyn Any + 'static)) -> TypeId {
    this.type_id()
}

pub trait StructMetaContainer: Downcast {
    fn get_struct_meta(&self) -> &StructMeta;
}
