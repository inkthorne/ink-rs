use crate::latent::Latent;
use crate::pool::ThreadPool;

// ===========================================================================
// ** WaitGroup **
// ===========================================================================

pub struct WaitGroup<T: Clone> {
    latents: Vec<Latent<T>>,
}

impl<T: Clone> WaitGroup<T> {
    // -----------------------------------------------------------------------

    pub fn new() -> Self {
        WaitGroup {
            latents: Vec::new(),
        }
    }

    // -----------------------------------------------------------------------

    pub fn wait(&mut self) -> Vec<T> {
        let mut results = Vec::<T>::new();

        for latent in self.latents.drain(..) {
            results.push(latent.wait());
        }

        results
    }
}

// ===========================================================================
// ** WorkGroup **
// ===========================================================================

pub struct WorkGroup<I: Send + 'static, O: Clone + Send + 'static> {
    pool: ThreadPool,
    func: fn(I) -> O,
}

impl<I: Send + 'static, O: Clone + Send + 'static> WorkGroup<I, O> {
    // -----------------------------------------------------------------------

    pub fn new(pool: ThreadPool, func: fn(I) -> O) -> Self {
        WorkGroup { pool, func }
    }

    // -----------------------------------------------------------------------

    pub fn put(&self, item: I) -> Latent<O> {
        let func = self.func;
        let closure = move || (func)(item);

        self.pool.put(closure)
    }

    // -----------------------------------------------------------------------

    pub fn is_running(&self) -> bool {
        self.pool.is_empty()
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
    fn validate_workgroup() {
        let work = |path: String| -> Vec<String> {
            let mut files = Vec::<String>::new();

            if let Ok(metadata) = std::fs::metadata(&path) {
                if metadata.is_dir() {
                    if let Ok(entries) = std::fs::read_dir(path) {
                        for entry in entries {
                            if let Ok(entry) = entry {
                                if entry.path().is_dir() {
                                    files.push(entry.path().to_str().unwrap().to_string());
                                }
                            }
                        }
                    }
                }
            }

            files
        };

        let pool = ThreadPool::new(2);
        let workgroup = WorkGroup::new(pool, work);
        workgroup.put("\\".to_string());

        /*
        loop {
            let dirs = workgroup.wait_one();

            for dir in dirs {
                println!("dir: {}", dir);
                workgroup.put(dir);
            }

            if workgroup.is_done() {
                break;
            }
        }
        */

        /*
        // while workgroup.is_running() {
        for _ in 0..2 {
            let files = output.wait();

            if files.len() > 0 {
                for file in files {
                    println!("file: {}", file);
                    output = workgroup.put(file);
                }

                // output = workgroup.putv(files);
            }
        }
        */

        println!("done");
    }
}
