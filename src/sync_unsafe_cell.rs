use std::cell::UnsafeCell;

pub struct SyncUnsafeCell<T>(UnsafeCell<T>);

impl<T> SyncUnsafeCell<T> {
    pub const fn new(value: T) -> Self {
        Self(UnsafeCell::new(value))
    }

    pub const fn get(&self) -> *mut T {
        self.0.get()
    }
}

unsafe impl<T> Sync for SyncUnsafeCell<T> {}
