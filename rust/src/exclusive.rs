use std::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
};

pub struct Exclusive<T>(UnsafeCell<T>);

impl<T> Exclusive<T> {
    pub unsafe fn new(t: T) -> Self {
        Exclusive(UnsafeCell::new(t))
    }

    pub fn get(&self) -> &mut T {
        unsafe { &mut *self.0.get() }
    }
}

impl<T> Deref for Exclusive<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<T> DerefMut for Exclusive<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get()
    }
}

unsafe impl<T> Sync for Exclusive<T> {}
