use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};

use crate::thread::AtomicInteger;

// ===========================================================================

struct ChannelData<T> {
    mutex: Mutex<VecDeque<T>>,
    put_event: Condvar,
    end_count: AtomicInteger,
    open_count: AtomicInteger,
    wait_count: AtomicInteger,
    instance_counter: AtomicInteger,
    _name: String,
}

impl<T> ChannelData<T> {
    fn new(name: &str) -> Self {
        ChannelData {
            mutex: Mutex::new(VecDeque::new()),
            put_event: Condvar::new(),
            end_count: AtomicInteger::new(0),
            open_count: AtomicInteger::new(0),
            wait_count: AtomicInteger::new(0),
            instance_counter: AtomicInteger::new(0),
            _name: name.to_string(),
        }
    }
}

// ===========================================================================

pub struct Channel<T> {
    data: Arc<ChannelData<T>>,
    instance_id: i32,
}

impl<T> Channel<T> {
    // -----------------------------------------------------------------------

    pub fn new() -> Self {
        Channel::named("")
    }

    // -----------------------------------------------------------------------

    pub fn named(name: &str) -> Self {
        let mut channel = Channel {
            data: Arc::new(ChannelData::new(name)),
            instance_id: 0,
        };

        channel.instance_id = channel.data.instance_counter.increment();
        channel.data.open_count.increment();
        channel
    }

    // -----------------------------------------------------------------------

    pub fn end(&self) {
        let deque = self.data.mutex.lock().unwrap();
        self.data.end_count.increment();

        if deque.is_empty() {
            self.data.put_event.notify_all();
        }
    }

    // -----------------------------------------------------------------------

    pub fn get(&self) -> Option<T> {
        let mut deque = self.data.mutex.lock().unwrap();

        if deque.len() > 0 {
            return deque.pop_front();
        }

        let end = self.data.end_count.get() > 0;

        if end {
            self.data.put_event.notify_all();
            return None;
        }

        let wait_count = self.data.wait_count.get();
        let open_count = self.data.open_count.get();

        if wait_count + 1 == open_count {
            self.data.put_event.notify_all();
            return None;
        }

        self.data.wait_count.increment();
        let mut deque = self.data.put_event.wait(deque).unwrap();
        self.data.wait_count.decrement();
        deque.pop_front()
    }

    // -----------------------------------------------------------------------

    pub fn put(&self, item: T) {
        let mut deque = self.data.mutex.lock().unwrap();
        deque.push_back(item);
        self.data.put_event.notify_one();
    }
}

impl<T> Clone for Channel<T> {
    // -----------------------------------------------------------------------

    fn clone(&self) -> Self {
        let channel = Channel {
            data: self.data.clone(),
            instance_id: self.data.instance_counter.increment(),
        };

        channel.data.open_count.increment();
        channel
    }
}

impl<T> Drop for Channel<T> {
    // -----------------------------------------------------------------------

    fn drop(&mut self) {
        let open = self.data.open_count.decrement() - 1;
        let waiting = self.data.wait_count.get();

        if waiting == open {
            self.data.put_event.notify_all();
        }
    }
}
