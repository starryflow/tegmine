use std::sync::atomic::{AtomicI32, Ordering};

pub struct SemaphoreUtil {
    num_slots: i32,
    available_slots: AtomicI32,
}

impl SemaphoreUtil {
    pub fn new(num_slots: i32) -> Self {
        Self {
            num_slots,
            available_slots: AtomicI32::new(num_slots),
        }
    }

    pub fn complete_processing(&self, num_slots: i32) {
        self.available_slots.fetch_add(num_slots, Ordering::SeqCst);
    }

    pub fn available_slots(&self) -> i32 {
        self.available_slots.load(Ordering::SeqCst)
    }

    pub fn acquire_slots(&self, num_slots: i32) -> bool {
        let current = self.available_slots.load(Ordering::SeqCst);
        if current < num_slots {
            return false;
        }
        return current
            == self.available_slots.compare_and_swap(
                current,
                current - num_slots,
                Ordering::SeqCst,
            );
    }
}
