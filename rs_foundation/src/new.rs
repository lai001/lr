use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
};

pub type SingleThreadMutType<T> = Rc<RefCell<T>>;
pub type MultipleThreadMutType<T> = Arc<Mutex<T>>;

pub struct SingleThreadMut {}

impl SingleThreadMut {
    pub fn new<T>(value: T) -> SingleThreadMutType<T> {
        Rc::new(RefCell::new(value))
    }
}

pub struct MultipleThreadMut {}

impl MultipleThreadMut {
    pub fn new<T>(value: T) -> MultipleThreadMutType<T> {
        Arc::new(Mutex::new(value))
    }
}
