use std::sync::{Arc, Condvar, Mutex};

// ===========================================================================
// ** Signal **
// ===========================================================================

pub struct Signal {
    cvar: Condvar,
    mutex: Mutex<u32>,
}

impl Signal {
    // -----------------------------------------------------------------------

    pub fn new() -> Self {
        Signal {
            cvar: Condvar::new(),
            mutex: Mutex::new(0),
        }
    }

    // -----------------------------------------------------------------------

    pub fn signal_all(&self) {
        let mut value = self.mutex.lock().unwrap();
        *value += 1;
        self.cvar.notify_all();
    }

    // -----------------------------------------------------------------------

    pub fn signal_one(&self) {
        let mut value = self.mutex.lock().unwrap();
        *value += 1;
        self.cvar.notify_one();
    }

    // -----------------------------------------------------------------------

    pub fn wait(&self) -> u32 {
        let guard = self.mutex.lock().unwrap();
        let value = self.cvar.wait(guard).unwrap();
        *value
    }
}

// ===========================================================================
// ** Gate **
// ===========================================================================

pub struct Gate {
    condvar: Condvar,
    mutex: Mutex<bool>,
}

impl Gate {
    // -----------------------------------------------------------------------

    pub fn new() -> Self {
        Gate {
            condvar: Condvar::new(),
            mutex: Mutex::new(false),
        }
    }

    // -----------------------------------------------------------------------

    pub fn arc() -> Arc<Self> {
        Arc::new(Gate::new())
    }

    // -----------------------------------------------------------------------

    pub fn open(&self) {
        let mut open = self.mutex.lock().unwrap();
        *open = true;
        self.condvar.notify_all();
    }

    // -----------------------------------------------------------------------

    pub fn close(&self) {
        let mut open = self.mutex.lock().unwrap();
        *open = false;
    }

    // -----------------------------------------------------------------------

    pub fn wait(&self) {
        let mut open = self.mutex.lock().unwrap();

        while !*open {
            open = self.condvar.wait(open).unwrap();
        }
    }
}

// ===========================================================================
// ** TESTS **
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    // -----------------------------------------------------------------------
    // ensure the signal is working

    #[test]
    fn validate_signal_wait() {
        let signal = Arc::new(Signal::new());
        let signal_clone = signal.clone();
        let handle = thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            signal_clone.signal_all();
        });

        signal.wait();
        handle.join().unwrap();
    }

    // -----------------------------------------------------------------------
    // ensure the gate is working

    #[test]
    fn validate_gate_open() {
        let gate = Gate::new();
        gate.open();
        gate.wait(); // shouldn't wait if gate is already open
    }
}
