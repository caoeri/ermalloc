#![no_std]

extern crate alloc;

struct Weak<'a, T> {
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
        Weak { weak: unsafe { Some(& *ptr) } }
    }

    pub fn invalidate(&mut self) {
        self.weak = None;
    }

    pub fn get_ref(&self) -> Option<&T> {
        self.weak
    }
}
