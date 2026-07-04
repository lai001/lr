use downcast_rs::Downcast;
use std::{
    any::{Any, type_name},
    cell::RefCell,
    marker::PhantomData,
    ptr::NonNull,
    rc::{Rc, Weak},
};

pub trait HasUrl {
    fn get_url(&self) -> url::Url;
}

/// # Safety Contract
///
/// The `Box<E>` inside the `RefCell` **must never be replaced**
/// (`take()`, `replace()`, `swap()`, etc.) while any `TypedRcRefCellBox` exists.
/// Replacing it invalidates cached pointers (use-after-free).
#[derive(Clone)]
pub struct TypedRcRefCellBox<E: Downcast + ?Sized, T: 'static> {
    #[allow(unused)]
    reference: Rc<RefCell<Box<E>>>,
    #[allow(unused)]
    value: NonNull<T>,
    phantom: PhantomData<*const ()>,
}

impl<E: Downcast + ?Sized, T: 'static> TypedRcRefCellBox<E, T> {
    pub fn new(reference: Rc<RefCell<Box<E>>>) -> crate::error::Result<Self> {
        let ptr = {
            let content_ref = reference.borrow();
            let any_ref: &dyn Any = (**content_ref).as_any();
            let value_ref: &T = any_ref.downcast_ref::<T>().ok_or_else(|| {
                crate::error::Error::TypeMismatch(format!(
                    "expected {}, got different type",
                    type_name::<T>(),
                ))
            })?;
            value_ref as *const T
        };
        Ok(Self {
            reference,
            value: unsafe { NonNull::new_unchecked(ptr as *mut T) },
            phantom: PhantomData,
        })
    }

    pub fn downgrade(&self) -> TypedRcRefCellBoxWeak<E, T> {
        TypedRcRefCellBoxWeak {
            reference: Rc::downgrade(&self.reference),
            _phantom: PhantomData,
        }
    }
}

#[cfg(not(debug_assertions))]
impl<E: Downcast + ?Sized, T: 'static> TypedRcRefCellBox<E, T> {
    /// # Safety Contract
    ///
    /// Bypasses `RefCell`. The caller must ensure:
    /// - No `&mut T` exists through any `TypedRcRefCellBox` referencing the same data.
    /// - The inner `Box<E>` has not been replaced.
    pub fn get(&self) -> &T {
        unsafe { self.value.as_ref() }
    }

    /// # Safety Contract
    ///
    /// Bypasses `RefCell`. The caller must ensure:
    /// - No other `&T` or `&mut T` exists (including from cloned `TypedRcRefCellBox`s).
    /// - The inner `Box<E>` has not been replaced.
    pub fn get_mut(&mut self) -> &mut T {
        unsafe { self.value.as_mut() }
    }
}

#[cfg(debug_assertions)]
impl<E: Downcast + ?Sized, T: 'static> TypedRcRefCellBox<E, T> {
    pub fn get(&self) -> std::cell::Ref<'_, T> {
        std::cell::Ref::map(self.reference.borrow(), |x| {
            (**x).as_any().downcast_ref::<T>().unwrap()
        })
    }

    pub fn get_mut(&mut self) -> std::cell::RefMut<'_, T> {
        std::cell::RefMut::map(self.reference.borrow_mut(), |x| {
            (**x).as_any_mut().downcast_mut::<T>().unwrap()
        })
    }
}

#[derive(Clone)]
pub struct TypedRcRefCellBoxWeak<E: Downcast + ?Sized, T: 'static> {
    reference: Weak<RefCell<Box<E>>>,
    _phantom: PhantomData<T>,
}

impl<E: Downcast + ?Sized, T: 'static> TypedRcRefCellBoxWeak<E, T> {
    pub fn upgrade(&self) -> Option<TypedRcRefCellBox<E, T>> {
        let reference = self.reference.upgrade()?;
        TypedRcRefCellBox::new(reference).ok()
    }
}
