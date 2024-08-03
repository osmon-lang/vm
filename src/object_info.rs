use crate::object::Object;
use std::{cell::Cell, ops::Deref, rc::Rc};

pub struct ObjectInfo {
    object: Box<dyn Object>,
    native_ref_info: ObjectNativeRefInfo,
}

pub struct ObjectHandle<'a> {
    object: &'a dyn Object,
    _native_ref_info: ObjectNativeRefInfo,
}

impl<'a> Deref for ObjectHandle<'a> {
    type Target = &'a dyn Object;
    fn deref(&self) -> &&'a dyn Object {
        &self.object
    }
}

pub struct ObjectNativeRefInfo {
    // TODO: Remove Rc
    n_refs: Rc<Cell<usize>>,

    // in case n_refs becomes zero
    gc_notified: bool,
}

impl ObjectInfo {
    pub fn new(obj: Box<dyn Object>) -> ObjectInfo {
        ObjectInfo {
            object: obj,
            native_ref_info: ObjectNativeRefInfo {
                n_refs: Rc::new(Cell::new(0)),
                gc_notified: false,
            },
        }
    }

    pub fn gc_notify(&mut self) {
        self.native_ref_info.gc_notified = true;
    }

    pub fn as_object(&self) -> &dyn Object {
        &*self.object
    }

    pub fn has_native_refs(&self) -> bool {
        self.native_ref_info.n_refs.get() != 0
    }

    pub fn handle<'a>(&self) -> ObjectHandle<'a> {
        ObjectHandle {
            object: unsafe {
                ::std::mem::transmute::<&dyn Object, &'static dyn Object>(&*self.object)
            },
            _native_ref_info: self.native_ref_info.clone(),
        }
    }
}

impl Drop for ObjectInfo {
    fn drop(&mut self) {
        if self.native_ref_info.n_refs.get() != 0 {
            eprintln!("Attempting to drop object with alive references");
            ::std::process::abort();
        }
    }
}

impl Clone for ObjectNativeRefInfo {
    fn clone(&self) -> Self {
        self.n_refs.replace(self.n_refs.get() + 1);
        ObjectNativeRefInfo {
            n_refs: self.n_refs.clone(),
            gc_notified: false,
        }
    }
}

impl Drop for ObjectNativeRefInfo {
    fn drop(&mut self) {
        let n_refs = self.n_refs.get();

        if self.gc_notified {
            assert_eq!(n_refs, 0);
        } else {
            assert!(n_refs > 0);
            self.n_refs.replace(n_refs - 1);
        }
    }
}

pub struct TypedObjectHandle<'a, T> {
    _handle: ObjectHandle<'a>,
    value: &'a T,
}

impl<'a, T> Deref for TypedObjectHandle<'a, T>
where
    T: 'a,
{
    type Target = &'a T;
    fn deref(&self) -> &&'a T {
        &self.value
    }
}

impl<'a, T> TypedObjectHandle<'a, T>
where
    T: 'static,
{
    pub fn downcast_from(other: ObjectHandle<'a>) -> Option<TypedObjectHandle<'a, T>> {
        let value = match other.object.as_any().downcast_ref::<T>() {
            Some(v) => v,
            None => return None,
        };
        Some(TypedObjectHandle {
            _handle: other,
            value,
        })
    }
}
