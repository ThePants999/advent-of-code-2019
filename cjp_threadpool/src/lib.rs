//! A crate containing my practice implementation of a variable size thread pool
//! for executing functions in parallel.  Spawns a specified number of worker
//! threads, but will panic if any of them panic.
//! 
//! After creating the [`ThreadPool`], give it work by passing boxed closures to 
//! [`schedule()`]. If your closures return a value, you can access the returned 
//! values through the [`results`] channel on the [`ThreadPool`].
//! 
//! # Examples
//! 
//! ```
//! use cjp_threadpool::ThreadPool;
//! 
//! let pool = ThreadPool::new_with_default_size();
//! let job = Box::new(move || 1 + 1);
//! for _ in 0..24 { pool.schedule(job.clone()); }
//! 
//! for _ in 0..24 { assert_eq!(Ok(2), pool.results.recv()); }
//! ```
//! 
//! [`ThreadPool`]: ./struct.ThreadPool.html
//! [`schedule()`]: ./struct.ThreadPool.html#method.schedule
//! [`results`]: ./struct.ThreadPool.html#structfield.results

#![crate_name = "cjp_threadpool"]
#![crate_type = "lib"]

extern crate num_cpus;

use std::thread::{self, JoinHandle};
use std::sync::{Arc, Mutex, mpsc::{self, Sender, Receiver}};

type Job<T> = Box<dyn FnOnce() -> T + Send + 'static>;

/// A thread pool that owns a number of worker threads and can schedule work across
/// them.
pub struct ThreadPool<T> {
    threads: Vec<Worker>,
    inbound_work_sender: Sender<Job<T>>,
    inbound_work_receiver: Arc<Mutex<Receiver<Job<T>>>>,
    /// The receive half of a channel on which the return values of scheduled jobs
    /// will be sent.
    pub results: Receiver<T>,
}

impl<T: Send + 'static> ThreadPool<T> {
    /// Creates a new thread pool capable of executing `num_threads` number of jobs
    /// concurrently.
    /// 
    /// # Panics
    /// 
    /// This function will panic if `num_threads` is 0.
    /// 
    /// # Examples
    /// 
    /// Create a new thread pool capable of executing four jobs concurrently, where
    /// the jobs return `()`:
    /// 
    /// ```
    /// use cjp_threadpool::ThreadPool;
    /// 
    /// let pool = ThreadPool::<()>::new(4);
    /// ```
    pub fn new(num_threads: usize) -> Self {
        let (inbound_work_sender, inbound_work_receiver) = mpsc::channel();
        let (result_sender, results) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(inbound_work_receiver));
        let mut pool = Self {
            threads: Vec::with_capacity(num_threads),
            inbound_work_sender,
            inbound_work_receiver: Arc::clone(&receiver),
            results,
        };
        for _ in 0..num_threads {
            pool.threads.push(Worker::new(Arc::clone(&receiver), result_sender.clone()));
        }
        pool
    }

    /// Creates a new thread pool with capacity equal to the number of logical
    /// CPUs on the machine it's running on.
    /// 
    /// # Examples
    /// 
    /// Create a new thread pool capable of executing one concurrent job per logical
    /// CPU:
    /// 
    /// ```
    /// use cjp_threadpool::ThreadPool;
    /// 
    /// let pool = ThreadPool::<()>::new_with_default_size();
    /// ```
    pub fn new_with_default_size() -> Self {
        Self::new(num_cpus::get())
    }

    /// Queues the function `job` for execution on a thread in the pool.
    /// 
    /// # Examples
    /// 
    /// Execute four jobs on a thread pool that can run two jobs concurrently:
    /// 
    /// ```
    /// use cjp_threadpool::ThreadPool;
    /// 
    /// let pool = ThreadPool::new(2);
    /// let job = Box::new(move || 1 + 1);
    /// for _ in 0..4 { pool.schedule(job.clone()); }
    /// pool.join();
    /// ```
    pub fn schedule(&self, job: Job<T>) {
        self.inbound_work_sender.send(job).unwrap();
    }

    /// Block the current thread until all jobs in the pool have been executed.
    pub fn join(self) {
        drop(self.inbound_work_sender);

        // Join all worker threads.
        for worker in self.threads {
            worker.thread.join().unwrap();
        }
    }

    /// Like [`join()`], but drops any pending jobs that aren't already mid-execution.
    /// 
    /// [`join()`]: #method.join
    pub fn terminate(self) {
        drop(self.inbound_work_sender);
        {
            // Consume all the remaining scheduled work.
            let receiver = self.inbound_work_receiver.lock().unwrap();
            while let Ok(job) = receiver.recv() { drop(job); }
        }

        // Join all worker threads.
        for worker in self.threads {
            worker.thread.join().unwrap();
        }
    }
}

struct Worker {
    thread: JoinHandle<()>,
}

impl Worker {
    fn new<T: Send + 'static>(receiver: Arc<Mutex<Receiver<Job<T>>>>, result_sender: Sender<T>) -> Self {
        Self {
            thread: thread::spawn(move || {
                loop {
                    let job = match receiver.lock().unwrap().recv() {
                        Ok(job) => job,
                        Err(_) => break,
                    };
                    let result = job();
                    result_sender.send(result).unwrap();
                }
                ()
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};
    use std::sync::{Arc, Mutex};

    #[test]
    fn two_sequential_jobs() {
        let pool = ThreadPool::new(1);
        let job = Box::new(move || 1 + 1);
        pool.schedule(job.clone());
        pool.schedule(job);
        assert_eq!(Ok(2), pool.results.recv());
        assert_eq!(Ok(2), pool.results.recv());
        assert!(pool.results.try_recv().is_err());
        pool.join();
    }

    #[test]
    fn highly_parallel() {
        let pool = ThreadPool::new(8);
        let job = Box::new(move || {
            thread::sleep(Duration::from_millis(100));
            1 + 1
        });
        let now = Instant::now();
        for _ in 0..24 {
            pool.schedule(job.clone());
        }
        for _ in 0..24 {
            assert_eq!(Ok(2), pool.results.recv());
        }
        assert!(now.elapsed().as_millis() < 350);
        pool.join();
    }

    #[test]
    fn terminate_early() {
        let pool = ThreadPool::new(8);
        let value = Arc::new(Mutex::new(0));
        for _ in 0..24 {
            let value_clone = Arc::clone(&value);
            let job = Box::new(move || {
                thread::sleep(Duration::from_millis(100));
                let mut data = value_clone.lock().unwrap();
                *data += 1;
                *data
            });
            pool.schedule(job);
        }
        thread::sleep(Duration::from_millis(50));
        pool.terminate();
        thread::sleep(Duration::from_secs(1));
        let data = value.lock().unwrap();
        assert_eq!(*data, 8);
    }
}