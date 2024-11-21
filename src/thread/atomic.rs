use std::sync::atomic::{AtomicI32, Ordering};

// ===========================================================================
// AtomicCounter
//
pub struct AtomicInteger {
    integer: AtomicI32,
}

impl AtomicInteger {
    // -----------------------------------------------------------------------

    pub const fn new(value: i32) -> Self {
        AtomicInteger {
            integer: AtomicI32::new(value),
        }
    }

    // -----------------------------------------------------------------------

    pub fn add(&self, value: i32) -> i32 {
        self.integer.fetch_add(value, Ordering::AcqRel)
    }

    // -----------------------------------------------------------------------

    pub fn sub(&self, value: i32) -> i32 {
        self.integer.fetch_sub(value, Ordering::AcqRel)
    }

    // -----------------------------------------------------------------------

    pub fn increment(&self) -> i32 {
        self.integer.fetch_add(1, Ordering::AcqRel)
    }

    // -----------------------------------------------------------------------

    pub fn decrement(&self) -> i32 {
        self.integer.fetch_sub(1, Ordering::AcqRel)
    }

    // -----------------------------------------------------------------------

    pub fn get(&self) -> i32 {
        self.integer.load(Ordering::Acquire)
    }

    // -----------------------------------------------------------------------

    pub fn set(&self, value: i32) {
        self.integer.store(value, Ordering::Release);
    }
}

// ===========================================================================
// ** TESTS **
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // ensure AtomicCounter increments & decrements correctly

    #[test]
    fn validate_atomic_counter() {
        let counter = AtomicInteger::new(13);
        counter.increment();
        counter.increment();
        assert_eq!(counter.get(), 15);

        counter.decrement();
        assert_eq!(counter.get(), 14);

        counter.set(100);
        assert_eq!(counter.get(), 100);
        counter.decrement();
        assert_eq!(counter.get(), 99);
    }
}
