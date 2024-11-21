use crate::thread::AtomicInteger;
use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};

// ===========================================================================
// ** SharedData **
// ===========================================================================

struct SharedData<T> {
    trigger: Condvar,
    triggered_events: Mutex<VecDeque<T>>,
    event_count: AtomicInteger,
}

impl<T> SharedData<T> {
    // -----------------------------------------------------------------------

    pub fn new() -> Self {
        SharedData {
            trigger: Condvar::new(),
            triggered_events: Mutex::new(VecDeque::new()),
            event_count: AtomicInteger::new(0),
        }
    }

    // -----------------------------------------------------------------------

    pub fn trigger(&self, value: T) {
        let mut lock = self.triggered_events.lock().unwrap();
        lock.push_back(value);
        self.trigger.notify_all();
    }

    // -----------------------------------------------------------------------

    pub fn wait_one(&self) -> Option<T> {
        let mut lock = self.triggered_events.lock().unwrap();

        if lock.len() > 0 {
            return lock.pop_front();
        }

        if self.event_count.get() < 1 {
            return None;
        }

        let mut lock = self.trigger.wait(lock).unwrap();
        lock.pop_front()
    }

    // -----------------------------------------------------------------------

    pub fn wait_some(&self) -> Vec<T> {
        let mut values = Vec::<T>::new();

        {
            let mut lock = self.triggered_events.lock().unwrap();
            let triggered_count = lock.len() as i32;

            if triggered_count > 0 {
                for _ in 0..triggered_count {
                    values.push(lock.pop_front().unwrap());
                }

                return values;
            }
        }

        if let Some(value) = self.wait_one() {
            values.push(value);
        }

        values
    }

    // -----------------------------------------------------------------------

    pub fn wait_all(&self) -> Vec<T> {
        let mut values = Vec::<T>::new();

        loop {
            let mut v = self.wait_some();

            if v.is_empty() {
                break;
            }

            values.append(&mut v);
        }

        values
    }
}

// ===========================================================================
// ** Event **
// ===========================================================================

pub struct Event<T: Copy> {
    shared: Arc<SharedData<T>>,
    value: T,
}

impl<T: Copy> Event<T> {
    // -----------------------------------------------------------------------

    fn new(shared: Arc<SharedData<T>>, value: T) -> Self {
        shared.event_count.increment();
        Event { shared, value }
    }

    // -----------------------------------------------------------------------

    pub fn trigger(self) {
        self.shared.trigger(self.value);
    }
}

impl<T: Copy> Drop for Event<T> {
    // -----------------------------------------------------------------------

    fn drop(&mut self) {
        self.shared.event_count.decrement();
    }
}

// ===========================================================================
// ** EventListner **
// ===========================================================================

pub struct EventListener<T: Copy> {
    shared: Arc<SharedData<T>>,
}

impl<T: Copy> EventListener<T> {
    // -----------------------------------------------------------------------

    pub fn new() -> Self {
        EventListener {
            shared: Arc::new(SharedData::new()),
        }
    }

    // -----------------------------------------------------------------------

    pub fn create_event(&mut self, value: T) -> Event<T> {
        Event::new(self.shared.clone(), value)
    }

    // -----------------------------------------------------------------------

    pub fn wait_one(&self) -> Option<T> {
        self.shared.wait_one()
    }

    // -----------------------------------------------------------------------

    pub fn wait_some(&self) -> Vec<T> {
        self.shared.wait_some()
    }

    // -----------------------------------------------------------------------

    pub fn wait_all(&self) -> Vec<T> {
        self.shared.wait_all()
    }
}

// ===========================================================================
// ** TESTS **
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------

    #[test]
    fn wait_one() {
        let mut listener = EventListener::<usize>::new();
        let mut triggered_indices = vec![false; 10];
        let mut triggered_count = 0;

        for i in 0..triggered_indices.len() {
            let event = listener.create_event(i);
            event.trigger();
        }

        loop {
            if let Some(index) = listener.wait_one() {
                triggered_indices[index] = true;
                triggered_count += 1;

                if triggered_count == 10 {
                    println!("all triggers fired");
                    break;
                }
            } else {
                break;
            }
        }

        assert!(triggered_count == 10);
    }

    // -----------------------------------------------------------------------

    #[test]
    fn wait_one_no_events() {
        let listener = EventListener::<usize>::new();
        let event_value = listener.wait_one();
        assert!(event_value == None);
    }

    // -----------------------------------------------------------------------

    #[test]
    fn wait_some_no_events() {
        let listener = EventListener::<usize>::new();
        let event_values = listener.wait_some();
        assert!(event_values.is_empty());
    }

    // -----------------------------------------------------------------------

    #[test]
    fn wait_all_no_events() {
        let listener = EventListener::<usize>::new();
        let event_values = listener.wait_all();
        assert!(event_values.is_empty());
    }
}
