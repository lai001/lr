use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

pub struct UnsafeTypeWrapper<T: ?Sized> {
    ptr: *mut T,
    _marker: PhantomData<T>,
}

impl<T: ?Sized> UnsafeTypeWrapper<T> {
    pub fn from_raw(ptr: *mut T) -> Self {
        Self {
            ptr,
            _marker: PhantomData,
        }
    }

    pub fn from_mut_ref(r: &mut T) -> Self {
        Self {
            ptr: r as *mut T,
            _marker: PhantomData,
        }
    }

    pub fn raw_ptr(&self) -> *mut T {
        self.ptr
    }

    pub fn mut_ref(&self) -> &mut T {
        unsafe { &mut *self.ptr }
    }
}

impl<T: ?Sized> Deref for UnsafeTypeWrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr }
    }
}

impl<T: ?Sized> DerefMut for UnsafeTypeWrapper<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.ptr }
    }
}

unsafe impl<T: ?Sized> Send for UnsafeTypeWrapper<T> {}
unsafe impl<T: ?Sized> Sync for UnsafeTypeWrapper<T> {}
