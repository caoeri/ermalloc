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

impl<'a, T> Weak<'a, T> {
    
}
