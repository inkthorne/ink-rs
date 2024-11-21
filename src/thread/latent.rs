use crate::thread::AtomicInteger;
use crate::thread::{Event, EventListener};
use std::collections::HashMap;
use std::sync::{Arc, Condvar, Mutex};

// ===========================================================================
// ** LatentWait **
// ===========================================================================

pub trait LatentWait {
    fn add_event(&self, event: Event<usize>, listener_id: usize);
    fn remove_event(&self, listener_id: usize);
    fn is_ready(&self) -> bool;
}

// ===========================================================================
// ** Latent **
// ===========================================================================

// struct LatentData<T: Clone> {
struct LatentData<T> {
    value: Mutex<Option<T>>,
    condvar: Condvar,
    events: Mutex<HashMap<usize, Event<usize>>>,
}

// impl<T: Clone> LatentData<T> {
impl<T> LatentData<T> {
    fn new() -> Self {
        LatentData {
            value: Mutex::new(None),
            condvar: Condvar::new(),
            events: Mutex::new(HashMap::new()),
        }
    }
}

#[derive(Clone)]
pub struct Latent<T: Clone> {
    shared: Arc<LatentData<T>>,
}

impl<T: Clone> Latent<T> {
    // -----------------------------------------------------------------------

    pub fn new() -> Self {
        Latent {
            shared: Arc::new(LatentData::<T>::new()),
        }
    }

    // -----------------------------------------------------------------------

    pub fn is_ready(&self) -> bool {
        let value = self.shared.value.lock().unwrap();
        value.is_some()
    }

    // -----------------------------------------------------------------------

    pub fn set(self, value: T) {
        let mut future_value = self.shared.value.lock().unwrap();

        // latent values can only be set once and the setter's copy is consumed
        assert!(future_value.is_none(), "value already set");

        if future_value.is_none() {
            *future_value = Some(value);
            self.shared.condvar.notify_all();
        }

        let mut events = self.shared.events.lock().unwrap();
        let entries = events.drain();

        for (_, event) in entries {
            event.trigger();
        }
    }

    // -----------------------------------------------------------------------

    pub fn wait(self) -> T {
        let mut value = self.shared.value.lock().unwrap();

        while value.is_none() {
            value = self.shared.condvar.wait(value).unwrap();
        }

        value.clone().unwrap()
    }
}

impl<T: Clone> LatentWait for Latent<T> {
    // -----------------------------------------------------------------------

    fn add_event(&self, event: Event<usize>, listener_id: usize) {
        let value = self.shared.value.lock().unwrap();

        if value.is_none() {
            let mut events = self.shared.events.lock().unwrap();
            events.insert(listener_id, event);
        } else {
            event.trigger();
        }
    }

    // -----------------------------------------------------------------------

    fn remove_event(&self, listener_id: usize) {
        let mut events = self.shared.events.lock().unwrap();
        events.remove(&listener_id);
    }

    // -----------------------------------------------------------------------

    fn is_ready(&self) -> bool {
        self.is_ready()
    }
}

// ===========================================================================
// ** LatentWaiter **
// ===========================================================================

pub struct LatentWaiter;

static WAITER_COUNT: AtomicInteger = AtomicInteger::new(0);

impl LatentWaiter {
    // -----------------------------------------------------------------------

    pub fn wait_one(latents: &Vec<&dyn LatentWait>) -> Option<usize> {
        let listener_id = WAITER_COUNT.increment();
        let mut listener = EventListener::<usize>::new();

        for (i, latent) in latents.iter().enumerate() {
            latent.add_event(listener.create_event(i), listener_id as usize);
        }

        let index = listener.wait_one();

        // TODO: need to remove the events added to the latents that didn't fire

        for (_, latent) in latents.iter().enumerate() {
            latent.remove_event(listener_id as usize);
        }

        index
    }

    // -----------------------------------------------------------------------

    pub fn wait_one_v<T: LatentWait>(latents: &Vec<T>) -> Option<usize> {
        let listener_id = WAITER_COUNT.increment();
        let mut listener = EventListener::<usize>::new();

        for (i, latent) in latents.iter().enumerate() {
            latent.add_event(listener.create_event(i), listener_id as usize);
        }

        let index = listener.wait_one();

        for (_, latent) in latents.iter().enumerate() {
            latent.remove_event(listener_id as usize);
        }

        index
    }

    // -----------------------------------------------------------------------

    pub fn wait_all(latents: &Vec<&dyn LatentWait>) -> Vec<usize> {
        let listener_id = WAITER_COUNT.increment();
        let mut listener = EventListener::<usize>::new();

        for (i, latent) in latents.iter().enumerate() {
            latent.add_event(listener.create_event(i), listener_id as usize);
        }

        listener.wait_all()
    }
}

// ===========================================================================
// ** LatentGroup **
// ===========================================================================

pub struct LatentGroup<T: Clone> {
    latents: HashMap<usize, Latent<T>>,
    listener: EventListener<usize>,
    counter: usize,
}

impl<T: Clone> LatentGroup<T> {
    // -----------------------------------------------------------------------

    pub fn new() -> Self {
        LatentGroup {
            latents: HashMap::new(),
            listener: EventListener::new(),
            counter: 0,
        }
    }

    // -----------------------------------------------------------------------

    pub fn add(&mut self, latent: Latent<T>) {
        self.counter += 1;

        let latent_id = self.counter;
        let latent_event = self.listener.create_event(latent_id);
        latent.add_event(latent_event, 0);
        self.latents.insert(latent_id, latent);
    }

    // -----------------------------------------------------------------------

    pub fn wait_one(&mut self) -> Option<Latent<T>> {
        let latent_id_opt = self.listener.wait_one();

        if let Some(latent_id) = latent_id_opt {
            return self.latents.remove(&latent_id);
        }

        None
    }

    // -----------------------------------------------------------------------

    pub fn wait_some(&mut self) -> Vec<Latent<T>> {
        let latent_ids = self.listener.wait_some();
        let mut latents = Vec::<Latent<T>>::with_capacity(latent_ids.len());

        for latent_id in latent_ids {
            let latent_opt = self.latents.remove(&latent_id);

            if let Some(latent) = latent_opt {
                latents.push(latent);
            }
        }

        latents
    }
}

// ===========================================================================
// ** TESTS **
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    // -----------------------------------------------------------------------
    // test Latent.wait() & Latent.is_ready()

    #[test]
    fn latent_wait() {
        let latent = Latent::<i32>::new();
        let latent_clone = latent.clone();
        let handle = thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            latent_clone.set(42);
        });

        assert_eq!(latent.is_ready(), false);

        let value = latent.wait();
        handle.join().unwrap();
        assert_eq!(value, 42);
    }

    // -----------------------------------------------------------------------
    // test LatentGroup.wait_one()

    #[test]
    fn latent_group_wait_one() {
        let latent1 = Latent::<i32>::new();
        let latent2 = Latent::<i32>::new();
        let latent3 = Latent::<i32>::new();

        let l1 = latent1.clone();
        let l2 = latent2.clone();
        let l3 = latent3.clone();

        // set 'l1' before being added to the wait group

        l1.set(42);

        let mut latent_group = LatentGroup::<i32>::new();
        assert!(latent_group.wait_one().is_none()); // nothing to wait on should return 'None'

        latent_group.add(latent1);
        latent_group.add(latent2);
        latent_group.add(latent3);

        // wait for 'l1'

        let opt1 = latent_group.wait_one();
        assert!(opt1.is_some());

        let lat1 = opt1.unwrap();
        assert!(lat1.is_ready());

        let val1 = lat1.wait();
        assert!(val1 == 42);

        // set 'l2' and wait

        l2.set(39);
        let opt2 = latent_group.wait_one();
        assert!(opt2.is_some());

        let lat2 = opt2.unwrap();
        assert!(lat2.is_ready());

        let val2 = lat2.wait();
        assert!(val2 == 39);

        // set 'l3' and wait

        l3.set(27);
        let opt3 = latent_group.wait_one();
        assert!(opt3.is_some());

        let lat3 = opt3.unwrap();
        assert!(lat3.is_ready());

        let val3 = lat3.wait();
        assert!(val3 == 27);

        assert!(latent_group.wait_one().is_none()); // nothing to wait on should return 'None'
    }

    // -----------------------------------------------------------------------
    // test LatentGroup.wait_some()

    #[test]
    fn latent_group_wait_some() {
        let latent1 = Latent::<i32>::new();
        let latent2 = Latent::<i32>::new();
        let latent3 = Latent::<i32>::new();

        let l1 = latent1.clone();
        let l2 = latent2.clone();
        let l3 = latent3.clone();

        // set 'l1' & 'l2' before being added to the wait group

        l1.set(42);
        l2.set(39);

        let mut latent_group = LatentGroup::<i32>::new();
        assert!(latent_group.wait_one().is_none()); // nothing to wait on should return 'None'

        latent_group.add(latent1);
        latent_group.add(latent2);
        latent_group.add(latent3);

        // wait for 'l1' & 'l2'

        let mut latents = latent_group.wait_some();
        assert!(latents.len() == 2);

        let ll1 = latents.pop().unwrap();
        assert!(ll1.is_ready());

        let ll1_value = ll1.wait();
        assert!(ll1_value == 42 || ll1_value == 39);

        let ll2 = latents.pop().unwrap();
        assert!(ll2.is_ready());

        let ll2_value = ll2.wait();
        assert!(ll2_value == 42 || ll2_value == 39);

        // set 'l3' and wait

        l3.set(27);
        let mut latents = latent_group.wait_some();
        assert!(latents.len() == 1);

        let ll3 = latents.pop().unwrap();
        assert!(ll3.is_ready());
        assert!(ll3.wait() == 27);

        let latents = latent_group.wait_some();
        assert!(latents.len() == 0); // nothing to wait on
    }

    // -----------------------------------------------------------------------
    // test LatentWaiter.wait_one()

    #[test]
    fn latent_waiter_wait_one() {
        let latent1 = Latent::<i32>::new();
        let l1 = latent1.clone();
        let latent2 = Latent::<String>::new();
        // let l2 = latent2.clone();
        let handle = thread::spawn(move || {
            thread::sleep(Duration::from_millis(1000));
            l1.set(42);
            // l2.set("hello".to_string());
        });

        let latents = vec![&latent1 as &dyn LatentWait, &latent2];
        let index = LatentWaiter::wait_one(&latents);
        assert!(index.unwrap() == 0);
        assert!(latents.len() == 2);
        assert!(latent1.is_ready() == true);
        assert!(latent1.shared.events.lock().unwrap().len() == 0);
        assert!(latent1.wait() == 42);
        handle.join().unwrap();

        assert!(latent2.is_ready() == false);
        assert!(latent2.shared.events.lock().unwrap().len() == 0);

        /*
        println!("waiting on latent1");
        latent1.set(6);
        Latent::wait_one(latent1, latent2);

        println!("waiting on latent1 & latent2");
        latent2.set("hello".to_string());
        Latent::wait_all(latent1, latent2);

        let v = vec![&latent1, &latent2];
        Latent::wait_one_v(v);
        */
    }

    // -----------------------------------------------------------------------
    // test LatentWaiter.wait_all()

    #[test]
    fn latent_waiter_wait_all() {
        let latent1 = Latent::<i32>::new();
        let l1 = latent1.clone();
        let latent2 = Latent::<String>::new();
        let l2 = latent2.clone();
        let handle = thread::spawn(move || {
            thread::sleep(Duration::from_millis(1000));
            l1.set(42);
            l2.set("hello".to_string());
        });

        let latents = vec![&latent1 as &dyn LatentWait, &latent2];
        let index_list = LatentWaiter::wait_all(&latents);
        assert!(index_list.len() == 2);
        assert!(latents.len() == 2);
        assert!(latent1.is_ready());
        assert!(latent1.wait() == 42);
        assert!(latent2.is_ready());
        assert!(latent2.wait() == "hello");
        handle.join().unwrap();
    }
}
