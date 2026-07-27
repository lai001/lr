pub mod error;

use std::any::{Any, TypeId};
use std::ptr::NonNull;

pub use error::DynCastError;

/// Similar to C++ `dynamic_cast` for trait objects.
pub trait DynCast: Any {
    fn dyn_cast_ref_erased(
        &self,
        _type_id: TypeId,
        req: &'static str,
    ) -> Result<NonNull<dyn Any>, DynCastError> {
        Err(DynCastError {
            requested: req,
            supported: &[],
        })
    }

    fn dyn_cast_mut_erased(
        &mut self,
        _type_id: TypeId,
        req: &'static str,
    ) -> Result<NonNull<dyn Any>, DynCastError> {
        Err(DynCastError {
            requested: req,
            supported: &[],
        })
    }
}

///Usage:
/// ```
/// use rs_foundation::dyn_cast::DynCast;
/// use rs_foundation::impl_dyn_cast;
///
/// trait MyTrait: DynCast {}
/// trait AnotherTrait: DynCast {}
///
/// struct MyStruct;
/// impl MyTrait for MyStruct {}
/// impl AnotherTrait for MyStruct {}
///
/// impl DynCast for MyStruct {
///     impl_dyn_cast!(dyn MyTrait, dyn AnotherTrait);
/// }
/// ```
#[macro_export]
macro_rules! impl_dyn_cast {
    ($($trait:ty),* $(,)?) => {
        fn dyn_cast_ref_erased(
            &self,
            type_id: std::any::TypeId,
            req: &'static str,
        ) -> Result<std::ptr::NonNull<dyn std::any::Any>, $crate::dyn_cast::DynCastError> {
            $(
                if type_id == std::any::TypeId::of::<$trait>() {
                    let tr: &$trait = self;
                    let ptr: *const $trait = tr;
                    let nn = unsafe { std::ptr::NonNull::new_unchecked(ptr as *mut $trait) };
                    return Ok(unsafe { std::mem::transmute(nn) });
                }
            )*
            Err($crate::dyn_cast::DynCastError {
                requested: req,
                supported: &[$(stringify!($trait)),*],
            })
        }

        fn dyn_cast_mut_erased(
            &mut self,
            type_id: std::any::TypeId,
            req: &'static str,
        ) -> Result<std::ptr::NonNull<dyn std::any::Any>, $crate::dyn_cast::DynCastError> {
            $(
                if type_id == std::any::TypeId::of::<$trait>() {
                    let tr: &mut $trait = self;
                    let ptr: *mut $trait = tr;
                    let nn = unsafe { std::ptr::NonNull::new_unchecked(ptr) };
                    return Ok(unsafe { std::mem::transmute(nn) });
                }
            )*
            Err($crate::dyn_cast::DynCastError {
                requested: req,
                supported: &[$(stringify!($trait)),*],
            })
        }
    };
}

/// Dynamically cast via `<dyn Trait>::from_dyn_cast` / `from_dyn_cast_mut`.
///
/// ```
/// use rs_foundation::dyn_cast::DynCast;
/// use rs_foundation::{impl_dyn_cast, dyn_cast_wrapper};
/// use rs_foundation::dyn_cast::DynCastError;
///
/// trait MyTrait: DynCast {}
/// struct MyStruct;
/// impl MyTrait for MyStruct {}
/// impl DynCast for MyStruct { impl_dyn_cast!(dyn MyTrait); }
/// dyn_cast_wrapper!(MyTrait);
///
/// let mut obj: Box<dyn MyTrait> = Box::new(MyStruct);
/// let result: Result<&mut dyn MyTrait, DynCastError> =
///     <dyn MyTrait>::from_dyn_cast_mut(obj.as_mut());
/// assert!(result.is_ok());
/// ```
#[macro_export]
macro_rules! dyn_cast_wrapper {
    ($trait:path) => {
        #[allow(unused)]
        impl dyn $trait + '_ {
            pub fn from_dyn_cast(
                obj: &dyn $crate::dyn_cast::DynCast,
            ) -> Result<&dyn $trait, $crate::dyn_cast::DynCastError> {
                let nn = obj.dyn_cast_ref_erased(
                    std::any::TypeId::of::<dyn $trait>(),
                    stringify!($trait),
                )?;
                let ptr: *const dyn $trait = unsafe { std::mem::transmute(nn) };
                Ok(unsafe { &*ptr })
            }

            pub fn from_dyn_cast_mut(
                obj: &mut dyn $crate::dyn_cast::DynCast,
            ) -> Result<&mut dyn $trait, $crate::dyn_cast::DynCastError> {
                let nn = obj.dyn_cast_mut_erased(
                    std::any::TypeId::of::<dyn $trait>(),
                    stringify!($trait),
                )?;
                let ptr: *mut dyn $trait = unsafe { std::mem::transmute(nn) };
                Ok(unsafe { &mut *ptr })
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::dyn_cast::DynCast;

    trait TraitA: DynCast {
        fn value(&self) -> i32 {
            100
        }
    }

    trait TraitB: DynCast {
        fn value(&self) -> i32 {
            200
        }
    }

    trait TraitC: DynCast {}

    struct TraitImpl {}

    impl TraitA for TraitImpl {}

    impl TraitB for TraitImpl {}

    impl DynCast for TraitImpl {
        impl_dyn_cast!(dyn TraitA, dyn TraitB,);
    }

    dyn_cast_wrapper!(TraitA);
    dyn_cast_wrapper!(TraitB);
    dyn_cast_wrapper!(TraitC);

    #[test]
    fn test_dyn_cast_success() {
        assert_eq!(100, (Box::new(TraitImpl {}) as Box<dyn TraitA>).value());
        assert_eq!(200, (Box::new(TraitImpl {}) as Box<dyn TraitB>).value());
        let mut trait_impl: Box<dyn TraitA> = Box::new(TraitImpl {});
        let trait_b = <dyn TraitB>::from_dyn_cast_mut(trait_impl.as_mut());
        assert!(trait_b.is_ok());
        assert_eq!(trait_b.unwrap().value(), 200);
    }

    #[test]
    fn test_dyn_cast_unsupported() {
        let mut trait_impl: Box<dyn TraitA> = Box::new(TraitImpl {});
        let trait_c = <dyn TraitC>::from_dyn_cast_mut(trait_impl.as_mut());
        assert!(trait_c.is_err());
        let err = match trait_c {
            Err(e) => e,
            Ok(_) => panic!("expected error"),
        };
        assert_eq!(err.requested, "TraitC");
        assert!(!err.supported.is_empty());
        assert!(err.supported.contains(&"dyn TraitA"));
        assert!(err.supported.contains(&"dyn TraitB"));
    }

    #[test]
    fn test_dyn_cast_error_display() {
        let mut trait_impl: Box<dyn TraitA> = Box::new(TraitImpl {});
        let trait_c = <dyn TraitC>::from_dyn_cast_mut(trait_impl.as_mut());
        let err = match trait_c {
            Err(e) => e,
            Ok(_) => panic!("expected error"),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("TraitC"));
        assert!(msg.contains("dyn TraitA"));
        assert!(msg.contains("dyn TraitB"));
    }

    #[test]
    fn test_dyn_cast_default_error() {
        struct NoSupport;
        impl DynCast for NoSupport {}

        let obj = NoSupport;
        let err = obj.dyn_cast_ref_erased(std::any::TypeId::of::<dyn DynCast>(), "DynCast");
        assert!(err.is_err());
        let err = match err {
            Err(e) => e,
            Ok(_) => panic!("expected error"),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("DynCast"));
    }
}
