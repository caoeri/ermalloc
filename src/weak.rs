#![no_std]

extern crate alloc;

pub trait Weakable {
    fn weak_exists(&self) -> bool;
    fn set_weak_exists(&mut self);
    fn reset_weak_exists(&mut self);
}

pub struct WeakMut<'a, T> where T: Weakable {
    weak: Option<&'a mut T>,
}

impl<'a, T> From<&'a mut T> for WeakMut<'a, T> where T: Weakable {
    fn from(r: &'a mut T) -> Self {
        r.set_weak_exists();
        WeakMut { weak: Some(r) }
    }
}

impl<'a, T> Default for WeakMut<'a, T> where T: Weakable {
    fn default() -> Self {
        WeakMut { weak: None }
    }
}

impl<'a, T> WeakMut<'a, T> where T: Weakable {
    pub unsafe fn from_ptr(ptr: *mut T) -> Self {
        let r = &mut *ptr;
        r.set_weak_exists();
        WeakMut { weak: Some(r) }
    }

    pub fn invalidate(&mut self) {
        if let Some(r) = &mut self.weak {
            r.reset_weak_exists();
        }
        self.weak = None;
    }

    pub fn get_ref_mut(&mut self) -> Option<&mut T> {
        if let Some(r) = &mut self.weak {
            r.reset_weak_exists();
        }
        self.weak.take()
    }
}

