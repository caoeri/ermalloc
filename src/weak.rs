#![no_std]

extern crate alloc;

pub trait Weakable {
    fn weak_exists(&self) -> bool;
    fn set_weak_exists(&mut self);
    fn reset_weak_exists(&mut self);
}

pub struct Weak<'a, T> where T: Weakable {
    weak: Option<&'a T>,
}

impl<'a, T> From<&'a T> for Weak<'a, T> where T: Weakable {
    fn from(r: &'a T) -> Self {
        if r.weak_exists() {
            panic!("Tried to create Weak when WeakMut exists");
        }
        Weak { weak: Some(r) }
    }
}

impl<'a, T> Default for Weak<'a, T> where T: Weakable {
    fn default() -> Self {
        Weak { weak: None }
    }
}

impl<'a, T> Weak<'a, T> where T: Weakable {
    pub unsafe fn from_ptr(ptr: *const T) -> Self {
        let r = & *ptr;
        Weak::from(r)
    }

    pub unsafe fn as_ptr(mut self) -> *const T {
        match self.get_ref() {
            Some(r) => {
                r as *const T
            },
            None => panic!("Called as_ptr on invalid Weak"),
        }
    }

    pub fn invalidate(&mut self) {
        self.weak = None;
    }

    pub fn get_ref(&mut self) -> Option<&T> {
        if let Some(r) = &self.weak {
            if r.weak_exists() {
                // Automagically invalidate
                // all Weaks if a WeakMut
                // is created
                self.invalidate();
            }
        }
        self.weak
    }

    pub fn upgrade(self) -> WeakMut<'a, T> {
        match self.weak {
            Some(r) => unsafe {
                // This will check if another WeakMut exists
                WeakMut::from(&mut *(r as *const T as *mut T))
            },
            None => WeakMut::default(),
        }
    }
}

pub struct WeakMut<'a, T> where T: Weakable {
    weak: Option<&'a mut T>,
}

impl<'a, T> From<&'a mut T> for WeakMut<'a, T> where T: Weakable {
    fn from(r: &'a mut T) -> Self {
        if r.weak_exists() {
            panic!("Tried to create WeakMut when WeakMut exists");
        }
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
        WeakMut::from(r)
    }

    pub unsafe fn as_ptr(mut self) -> *mut T {
        match self.get_ref_mut() {
            Some(r) => {
                r.reset_weak_exists();
                r as *mut T
            },
            None => panic!("Called as_ptr on invalid WeakMut"),
        }
    }

    pub fn get_ref_mut(mut self) -> Option<&'a mut T> {
        if let Some(r) = &mut self.weak {
            r.reset_weak_exists();
        }
        self.weak
    }

    pub fn downgrade(mut self) -> Weak<'a, T> {
        match self.get_ref_mut() {
            Some(r) => unsafe {
                Weak::from(& *(r as *mut T as *const T))
            },
            None => Weak::default(),
        }
    }
}

