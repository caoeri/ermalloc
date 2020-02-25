#![no_std]

extern crate alloc;

struct Weak<'a, T> {
    weak: Option<&'a T>,
}

impl<'a, T> Weak<'a, T> {
    
}
