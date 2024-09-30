use std::{
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
};

/// Represents a type alias for a job, which is a boxed closure which can be called once and takes
/// no arguments.
///
/// Reference:
/// `FnOnce`: Can be called once. Takes ownership of captured variables.
/// `FnMut`: Can be called multiple times. Captures variables by mutable reference.
/// `Fn`: Can be called multiple times. Captures variables by immutable reference.
type Job = Box<dyn FnOnce() + Send + 'static>;

/// Represents the different signals the `ThreadPool` accepts.
enum Message {
    NewJob(Job),
    Terminate,
}

/// Represents a `ThreadPool` with an arbitrary number of workers which uses a `Sender` to
/// communicate to its `Worker` threads.
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Sender<Message>,
}

impl ThreadPool {
    /// Create a new `ThreadPool` of a given size.
    pub fn new(size: usize) -> Self {
        assert!(size > 0, "Thread pool size must be greater than 0.");

        let (sender, receiver) = mpsc::channel();

        // Wrap the receiver in an `Arc<Mutext<_>>` to allow shared ownership and safe concurrent
        // access.
        let receiver = Arc::new(Mutex::new(receiver));

        // Initialize the `Worker` objects for the given `size` of the thread pool.
        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, receiver.clone()));
        }

        Self { workers, sender }
    }

    /// Execute a job by sending it to the channel.
    ///
    /// We require the closure to implement `Send` because jobs will be moved from the main thread
    /// to a worker thread. Because we do not need concurrent access to a job, it does not need to
    /// be `Sync` (in other words, each job is owned by the thread that executes it).
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        // The `Job` type is only used privately because the interface for users is simpler and
        // more ergonomic for them to only provide the closure. In other words, they shouldn't need
        // to know about the `Box`.
        //
        // The `Box` is needed because on its own, the trait object `dyn FnOnce() + Send + 'static`
        // is unsized, meaning its size is not known at compile time. `Box` allocates the data on
        // the heap and also gives us a sized type. Its size is now the size of a pointer, which
        // allows the `Sender` to accept the boxed closure.
        let job: Job = Box::new(f);
        self.sender
            .send(Message::NewJob(job))
            .expect("Failed to send.");
    }
}

impl Drop for ThreadPool {
    /// Ensure all the worker threads are joined before the `ThreadPool` is dropped, which is when
    /// it goes out of scope.
    fn drop(&mut self) {
        for _ in &self.workers {
            self.sender
                .send(Message::Terminate)
                .expect("Failed to send termination message.");
        }

        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                println!("Stopping worker {}", worker.id);
                thread.join().expect("Failed to join.");
            }
        }
    }
}

/// Represents a [`Worker`] which can be used by the [`ThreadPool`].
struct Worker {
    /// The worker's ID
    id: usize,

    /// The `Worker` must hold a reference to the thread used to perform its work.
    /// Use an `Option` so that we can set it to `None` using `take()` during shutdown.
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    /// Create a new `Worker` with a given ID and a receiver for `Job`s.
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Message>>>) -> Self {
        let thread = thread::spawn(move || loop {
            // While another worker holds the `Mutex` lock, we will block on `receiver.lock()`.
            // While we hold the `Mutex` lock, we will block on `recv()` while we wait for a
            // message.
            let job = receiver.lock().expect("Failed to lock.").recv();

            match job {
                Ok(Message::NewJob(job)) => {
                    // Execute the job.
                    job();
                }
                Ok(Message::Terminate) => {
                    println!("Worker {} received terminate message.", id);
                    break;
                }
                Err(_) => {
                    // If there is an error, it means the channel is disconnected,
                    // so we break the loop.
                    break;
                }
            }
        });

        Self {
            id,
            thread: Some(thread),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn test_thread_pool_creation() {
        let pool = ThreadPool::new(4);
        assert_eq!(pool.workers.len(), 4);
        drop(pool);
    }

    #[test]
    #[should_panic(expected = "Thread pool size must be greater than 0.")]
    fn test_thread_pool_creation_invalid_size() {
        ThreadPool::new(0);
    }

    #[test]
    fn test_execute_job() {
        let pool = ThreadPool::new(4);
        let (sender, receiver) = mpsc::channel();

        pool.execute(move || {
            sender.send(42).expect("Failed to send message.");
        });

        let result = receiver.recv_timeout(Duration::from_secs(1));
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_multiple_jobs() {
        let pool = ThreadPool::new(4);
        let (sender, receiver) = mpsc::channel();

        for i in 0..10 {
            let sender = sender.clone();
            pool.execute(move || {
                sender.send(i).expect("Failed to send message.");
            });
        }

        let mut results: Vec<i32> = Vec::new();
        for _ in 0..10 {
            results.push(receiver.recv_timeout(Duration::from_secs(1)).unwrap());
        }

        results.sort();
        assert_eq!(results, (0..10).collect::<Vec<i32>>());
    }

    #[test]
    fn test_thread_pool_drop() {
        let pool = ThreadPool::new(4);
        let (sender, receiver) = mpsc::channel();

        pool.execute(move || {
            thread::sleep(Duration::from_millis(100));
            sender.send(42).expect("Failed to send message.");
        });

        drop(pool);

        let result = receiver.recv_timeout(Duration::from_millis(200));
        assert_eq!(result.unwrap(), 42);
    }
}
