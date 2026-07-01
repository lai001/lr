use crate::kinds::TaskKind;
use crate::types::RawKey;
use crate::types::TaskKey;
use std::any::{Any, type_name};
use std::fmt::Debug;
use std::sync::Arc;

pub trait TaskNode: Send + Sync {
    fn name(&self) -> &str;

    fn run(
        &self,
        inputs: &[Arc<dyn Any + Send + Sync>],
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, String>;

    fn kind(&self) -> TaskKind {
        TaskKind::Map
    }

    fn type_info(&self) -> Option<String> {
        None
    }

    fn format_input(&self, input: &Arc<dyn Any + Send + Sync>) -> String {
        let _ = input;
        "<unknown>".into()
    }

    fn format_output(&self, output: &Arc<dyn Any + Send + Sync>) -> String {
        let _ = output;
        "<unknown>".into()
    }
}

pub struct TypedTask<I, O, F> {
    pub name: String,
    pub kind: TaskKind,
    pub f: F,
    pub _marker: std::marker::PhantomData<(I, O)>,
}

impl<I, O, F> TaskNode for TypedTask<I, O, F>
where
    I: Send + Sync + 'static + Debug,
    O: Send + Sync + 'static + Debug,
    F: Fn(&I) -> Result<O, String> + Send + Sync + 'static,
{
    fn name(&self) -> &str {
        &self.name
    }

    fn kind(&self) -> TaskKind {
        self.kind.clone()
    }

    fn run(
        &self,
        inputs: &[Arc<dyn Any + Send + Sync>],
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, String> {
        let any = inputs.get(0).ok_or_else(|| "no input".to_string())?;
        let i = any
            .downcast_ref::<I>()
            .ok_or_else(|| "input type mismatch".to_string())?;
        let o = (self.f)(i)?;
        Ok(Some(Arc::new(o) as Arc<dyn Any + Send + Sync>))
    }

    fn type_info(&self) -> Option<String> {
        Some(format!("I={}, O={}", type_name::<I>(), type_name::<O>()))
    }

    fn format_input(&self, input: &Arc<dyn Any + Send + Sync>) -> String {
        if let Some(v) = input.downcast_ref::<I>() {
            format!("{:?}", v)
        } else {
            "<bad input type>".into()
        }
    }

    fn format_output(&self, output: &Arc<dyn Any + Send + Sync>) -> String {
        if let Some(v) = output.downcast_ref::<O>() {
            format!("{:?}", v)
        } else {
            "<bad output type>".into()
        }
    }
}

pub struct JoinTask<O, F> {
    pub name: String,
    pub f: F,
    pub _marker: std::marker::PhantomData<O>,
}

impl<O, F> TaskNode for JoinTask<O, F>
where
    O: Send + Sync + 'static,
    F: Fn(&[Arc<dyn Any + Send + Sync>]) -> Result<O, String> + Send + Sync + 'static,
{
    fn name(&self) -> &str {
        &self.name
    }

    fn kind(&self) -> TaskKind {
        TaskKind::Join
    }

    fn run(
        &self,
        inputs: &[Arc<dyn Any + Send + Sync>],
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, String> {
        let out = (self.f)(inputs)?;
        Ok(Some(Arc::new(out)))
    }

    fn type_info(&self) -> Option<String> {
        Some(format!("I={}, O={}", "()", type_name::<O>()))
    }
}

pub struct TypedJoinTask<T, O, F> {
    pub name: String,
    pub f: F,
    pub _marker: std::marker::PhantomData<(T, O)>,
}

impl<T, O, F> TaskNode for TypedJoinTask<T, O, F>
where
    T: FromInputs + Send + Sync + 'static,
    O: Send + Sync + 'static,
    F: Fn(T) -> Result<O, String> + Send + Sync + 'static,
{
    fn name(&self) -> &str {
        &self.name
    }

    fn kind(&self) -> TaskKind {
        TaskKind::Join
    }

    fn run(
        &self,
        inputs: &[Arc<dyn Any + Send + Sync>],
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, String> {
        let typed_inputs = T::from_inputs(inputs)?;
        let out = (self.f)(typed_inputs)?;
        Ok(Some(Arc::new(out)))
    }

    fn type_info(&self) -> Option<String> {
        Some(format!("I={}, O={}", type_name::<T>(), type_name::<O>()))
    }
}

pub struct TypedJoinTasks<O, F> {
    pub name: String,
    pub f: F,
    pub _marker: std::marker::PhantomData<O>,
}

impl<O, F> TaskNode for TypedJoinTasks<O, F>
where
    O: Send + Sync + 'static,
    F: Fn(&[Arc<dyn Any + Send + Sync>]) -> Result<O, String> + Send + Sync + 'static,
{
    fn name(&self) -> &str {
        &self.name
    }

    fn kind(&self) -> TaskKind {
        TaskKind::Join
    }

    fn run(
        &self,
        inputs: &[Arc<dyn Any + Send + Sync>],
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, String> {
        let out = (self.f)(inputs)?;
        Ok(Some(Arc::new(out)))
    }
}

pub trait FromInputs: Sized {
    fn from_inputs(inputs: &[Arc<dyn Any + Send + Sync>]) -> Result<Self, String>;
}

impl<A> FromInputs for (A,)
where
    A: 'static + Send + Sync + Clone,
{
    fn from_inputs(inputs: &[Arc<dyn Any + Send + Sync>]) -> Result<Self, String> {
        let a = inputs[0].downcast_ref::<A>().ok_or("type mismatch")?;
        Ok((a.clone(),))
    }
}

impl<A, B> FromInputs for (A, B)
where
    A: 'static + Send + Sync + Clone,
    B: 'static + Send + Sync + Clone,
{
    fn from_inputs(inputs: &[Arc<dyn Any + Send + Sync>]) -> Result<Self, String> {
        let a = inputs[0].downcast_ref::<A>().ok_or("type mismatch")?;
        let b = inputs[1].downcast_ref::<B>().ok_or("type mismatch")?;
        Ok((a.clone(), b.clone()))
    }
}

impl<A, B, C> FromInputs for (A, B, C)
where
    A: 'static + Send + Sync + Clone,
    B: 'static + Send + Sync + Clone,
    C: 'static + Send + Sync + Clone,
{
    fn from_inputs(inputs: &[Arc<dyn Any + Send + Sync>]) -> Result<Self, String> {
        let a = inputs[0].downcast_ref::<A>().ok_or("type mismatch")?;
        let b = inputs[1].downcast_ref::<B>().ok_or("type mismatch")?;
        let c = inputs[2].downcast_ref::<C>().ok_or("type mismatch")?;
        Ok((a.clone(), b.clone(), c.clone()))
    }
}

pub trait IntoRawKey {
    fn into_raw(self) -> RawKey;
}

impl<I, O> IntoRawKey for TaskKey<I, O> {
    fn into_raw(self) -> RawKey {
        self.raw
    }
}

impl IntoRawKey for RawKey {
    fn into_raw(self) -> RawKey {
        self
    }
}
