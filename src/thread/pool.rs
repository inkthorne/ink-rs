use crate::thread::AtomicInteger;
use crate::thread::Channel;
use crate::thread::Latent;
use crate::thread::Signal;
use std::sync::Arc;
use std::thread;

// ===========================================================================
struct Task {
    func: Box<dyn FnOnce() + Send + 'static>,
}

impl Task {
    fn new(func: impl FnOnce() + Send + 'static) -> Self {
        Task {
            func: Box::new(func),
        }
    }
}

// ===========================================================================

pub struct ThreadPool {
    thread_count: usize,
    threads: Vec<thread::JoinHandle<()>>,
    task_channel: Channel<Task>,
    running_count: Arc<AtomicInteger>,
    empty_signal: Arc<Signal>,
}

impl ThreadPool {
    // -----------------------------------------------------------------------

    fn create_thread(&mut self) {
        // clone struct values to be captured by the thread

        let task_channel = self.task_channel.clone();
        let running_count = self.running_count.clone();
        let empty_signal = self.empty_signal.clone();
        let _id = self.threads.len();

        // create the worker thread

        let handle = thread::spawn(move || {
            // println!("pool thread {} started", _id);

            // wait for a task from the channel

            while let Some(task) = task_channel.get() {
                running_count.increment();
                // let _result = task();
                let _result = (task.func)();
                running_count.decrement();

                if running_count.get() == 0 {
                    empty_signal.signal_all();
                }
            }

            // println!("pool thread {} exited", _id);
        });

        // save the thread handle

        self.threads.push(handle);
    }

    // -----------------------------------------------------------------------

    pub fn new(thread_count: usize) -> Self {
        let mut pool = ThreadPool {
            thread_count,
            threads: Vec::with_capacity(thread_count),
            task_channel: Channel::named("ThreadPool"),
            running_count: Arc::new(AtomicInteger::new(0)),
            empty_signal: Arc::new(Signal::new()),
        };

        for _ in 0..thread_count {
            pool.create_thread();
        }

        pool
    }

    // -----------------------------------------------------------------------

    pub fn thread_count(&self) -> usize {
        self.thread_count
    }

    // -----------------------------------------------------------------------
    // returns 'true' if no threads are currently running

    pub fn is_empty(&self) -> bool {
        self.running_count.get() == 0
    }

    // -----------------------------------------------------------------------
    // returns 'true' if all threads are currently running

    pub fn is_full(&self) -> bool {
        self.running_count.get() == self.thread_count as i32
    }

    // -----------------------------------------------------------------------
    // fill the pool with the same 'task' on all threads

    pub fn fill<T: Clone + Send + 'static>(
        &self,
        task: impl FnOnce() -> T + Send + Clone + 'static,
    ) -> Vec<Latent<T>> {
        let mut results = Vec::<Latent<T>>::with_capacity(self.thread_count);

        for _ in 1..self.thread_count {
            let latent = self.put(task.clone());
            results.push(latent);
        }

        let latent = self.put(task);
        results.push(latent);
        results
    }

    // -----------------------------------------------------------------------
    // 'put' a task into the pool's queue

    pub fn put<T: Clone + Send + 'static>(
        &self,
        task: impl FnOnce() -> T + Send + 'static,
    ) -> Latent<T> {
        let latent = Latent::<T>::new();
        let l = latent.clone();
        let t = move || {
            let r = task();
            l.set(r);
        };

        let task_info = Task::new(t);
        self.task_channel.put(task_info);
        latent
    }

    // -----------------------------------------------------------------------
    // wait for all tasks to complete

    pub fn wait(&self) {
        if self.running_count.get() > 0 {
            self.empty_signal.wait();
        }
    }
}

// ===========================================================================
// TESTS

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    // -----------------------------------------------------------------------

    #[test]
    fn validate_threadpool_fill() {
        let threads = 12;
        println!("threads in pool: {}", threads);
        let pool = ThreadPool::new(threads);
        let results_rx = Channel::<i32>::new();
        let results_tx = results_rx.clone();
        let outgoing = Channel::<i32>::new();
        let incoming = outgoing.clone();
        let worker = move || {
            while let Some(item) = incoming.get() {
                thread::sleep(Duration::from_millis(1000));
                let new_item = item + 1;
                results_tx.put(new_item);
                // println!("tid: {:?}, item: {}", thread::current().id(), item);
            }
        };

        pool.fill(worker);

        for _ in 0..threads * 2 {
            outgoing.put(0);
        }

        drop(outgoing);
        let mut sum = 0;

        while let Some(item) = results_rx.get() {
            println!("*** result: {}", item);
            sum += item;
        }

        assert!(sum as usize == threads * 2);
        pool.wait();
    }

    // -----------------------------------------------------------------------
    // test that 'Gate' is working properly

    #[test]
    fn validate_threadpool_gate() {
        let pool = ThreadPool::new(2);
        let work_1_second = move || {
            thread::sleep(Duration::from_millis(1000));
        };

        let work_3_seconds = move || {
            thread::sleep(Duration::from_millis(3000));
        };

        let done1 = pool.put(work_1_second);
        let done2 = pool.put(work_3_seconds);
        let done3 = pool.put(work_1_second);
        let done4 = pool.put(work_1_second);
        done1.wait();
        done2.wait();
        done3.wait();
        done4.wait();

        assert!(pool.is_empty());
    }

    // -----------------------------------------------------------------------

    #[test]
    fn validate_threadpool_return_value() {
        let pool = ThreadPool::new(2);
        let task_42 = move || {
            thread::sleep(Duration::from_millis(1500));
            42
        };

        let task_39 = move || {
            thread::sleep(Duration::from_millis(1000));
            39
        };

        let latent_39 = pool.put(task_39);
        assert!(latent_39.is_ready() == false);

        let value_42 = pool.put(task_42).wait();
        assert_eq!(value_42, 42);

        assert!(latent_39.is_ready() == true);
        let value_39 = latent_39.wait();
        assert_eq!(value_39, 39);

        let task_none = move || {
            thread::sleep(Duration::from_millis(1000));
        };

        let none = pool.put(task_none);
        assert!(none.is_ready() == false);

        let none_value = none.wait();
        assert!(none_value == ());

        pool.put(task_42);
        pool.wait();
        assert!(pool.is_empty());
    }
}
