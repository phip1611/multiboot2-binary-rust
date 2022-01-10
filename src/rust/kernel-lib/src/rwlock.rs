use core::cell::UnsafeCell;
use core::sync::atomic::{Ordering, AtomicU64};
use core::ops::{Deref, DerefMut};
use crate::mutex::SimpleMutex;

/// A simple read write lock. Allows either n readers or one writer.
#[derive(Debug)]
pub struct SimpleRwLock<T> {
    data: UnsafeCell<T>,
    critical_section: SimpleMutex<()>,
    write_count: AtomicU64,
    read_count: AtomicU64,
}

unsafe impl <T> Send for SimpleRwLock<T> {}
unsafe impl <T> Sync for SimpleRwLock<T> {}

impl <T> SimpleRwLock<T> {
    pub const fn new(data: T) -> Self {
        Self {
            data: UnsafeCell::new(data),
            critical_section: SimpleMutex::new(()),
            read_count: AtomicU64::new(0),
            write_count: AtomicU64::new(0),
        }
    }

    /*pub fn into_inner(self) -> T {
        if self.lock.load(Ordering::SeqCst) == LOCKED {
            panic!("Still in use!");
        }
        self.data.into_inner()
    }*/


    pub fn try_lock_read(&self) -> Result<SimpleRwLockReadGuard<T>, ()> {
        let lock = self.critical_section.lock();
        lock.execute_while_locked(&|| {
            if self.can_read() {
                Ok(
                    SimpleRwLockReadGuard {
                        lock: &self
                    }
                )
            } else {
                Err(())
            }
        })
    }

    pub fn try_lock_write(&self) -> Result<SimpleRwLockWriteGuard<T>, ()> {
        let lock = self.critical_section.lock();
        lock.execute_while_locked(&|| {
            if self.can_write() {
                Ok(
                    SimpleRwLockWriteGuard {
                        lock: &self
                    }
                )
            } else {
                Err(())
            }

        })
    }

    pub fn lock_read(&self) -> SimpleRwLockReadGuard<T> {
        loop {
            if let Ok(l) = self.try_lock_read() {
                return l;
            }
        }
    }

    pub fn lock_write(&self) -> SimpleRwLockWriteGuard<T> {
        loop {
            if let Ok(l) = self.try_lock_write() {
                return l;
            }
        }
    }

    /// NOTE THAT THIS IS JUST A SNAPSHOT DURING THE FUNCTION CALL! During the time you call
    /// "lock_write" already everything can be changed! This is useful for testing.
    fn can_write(&self) -> bool {
        self.read_count.load(Ordering::SeqCst) == 0 && self.write_count.load(Ordering::SeqCst) == 0
    }

    fn can_read(&self) -> bool {
        self.write_count.load(Ordering::SeqCst) == 0
    }
}

#[derive(Debug)]
pub struct SimpleRwLockWriteGuard<'a, T> {
    lock: &'a SimpleRwLock<T>,
}

impl<T> Deref for SimpleRwLockWriteGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> DerefMut for SimpleRwLockWriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T> Drop for SimpleRwLockWriteGuard<'_, T> {
    #[inline]
    fn drop(&mut self) {
        self.lock.write_count.fetch_sub(1, Ordering::SeqCst);
    }
}

#[derive(Debug)]
pub struct SimpleRwLockReadGuard<'a, T> {
    lock: &'a SimpleRwLock<T>,
}

impl<T> Deref for SimpleRwLockReadGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> Drop for SimpleRwLockReadGuard<'_, T> {
    #[inline]
    fn drop(&mut self) {
        self.lock.read_count.fetch_sub(1, Ordering::SeqCst);
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_mutex() {
        let std_mutex = std::sync::Mutex::new(0);
        let my_mutex = SimpleMutex::new(0);

        for _i in 0..1_000_000 {
            let mut std_lock = std_mutex.lock().unwrap();
            let mut my_lock = my_mutex.lock();

            *std_lock = *std_lock + 1;
            *my_lock = *my_lock + 1;
        }

        assert_eq!(1_000_000, *std_mutex.lock().unwrap());
        assert_eq!(1_000_000, *my_mutex.lock());
    }
}