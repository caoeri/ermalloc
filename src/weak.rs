#![no_std]

extern crate alloc;

pub struct Weak<'a, T> {
    weak: Option<&'a T>,
}

impl<'a, T> From<&'a T> for Weak<'a, T> {
    fn from(r: &'a T) -> Self {
        Weak { weak: Some(r) }
    }
}

impl<'a, T> Default for Weak<'a, T> {
    fn default() -> Self {
        Weak { weak: None }
    }
}

impl<'a, T> Weak<'a, T> {
    pub unsafe fn from_ptr(ptr: *const T) -> Self {
        Weak { weak: Some(& *ptr) }
    }

    pub fn invalidate(&mut self) {
        self.weak = None;
    }

    pub fn get_ref(&self) -> Option<&T> {
        self.weak
    }
}

pub struct WeakMut<'a, T> {
    weak: Option<&'a mut T>,
}

impl<'a, T> From<&'a mut T> for WeakMut<'a, T> {
    fn from(r: &'a mut T) -> Self {
        WeakMut { weak: Some(r) }
    }
}

impl<'a, T> Default for WeakMut<'a, T> {
    fn default() -> Self {
        WeakMut { weak: None }
    }
}

impl<'a, T> WeakMut<'a, T> {
    pub unsafe fn from_ptr(ptr: *mut T) -> Self {
        WeakMut { weak: Some(&mut *ptr) }
    }

    pub fn invalidate(&mut self) {
        self.weak = None;
    }

    pub fn get_ref_mut(&mut self) -> Option<&mut T> {
        self.weak.take()
    }
}

