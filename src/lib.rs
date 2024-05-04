use std::{
    sync::{
        mpsc::{self, Receiver},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
};

// any closure or function pointer which can be called exactly once and
//is  safe to pass between threads and
//does not capture any referrences which have non static lifetime
type Job = Box<dyn FnOnce() + Send + 'static>;

//we will pass messages to the workers to tell them what to do
// either we can pass a new job or terminate the worker via the terminate variant
enum Message {
    NewJob(Job),
    Terminate,
}

//each wroker will be assigned an id and a thread
// the worker can use it's thread to execute the jobs
struct Worker {
    _id: usize,
    //we are using option so that while dropping or terminating we can replace thread with None
    thread: Option<JoinHandle<()>>,
}

///Threadpool will have a group of `workers` who will be listening for `jobs`.
/// Use the `new` associated function to create a new `threadpool` with `size` number of `workers`.
/// Use the `execute` method to `send` a job to the workers.
//The threadpool will have a sender which will be used to send Messages to the workers.
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
    trace: bool,
}

//debugger function to print states of the workers
fn print_state(trace: bool, msg: String) {
    if trace {
        println!("{}", msg);
    }
}

//make the workers ready to listen for messages during their creation
impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Message>>>, trace: bool) -> Worker {
        //span a new thread for the worker and keep it alive infinitely with loop
        //here both the lock and the recv are blocking calls
        let thread = thread::spawn(move || loop {
            let message = receiver
                .lock()
                .expect("Failed to aquire lock")
                .recv()
                .expect("Failed to receive message");

            match message {
                Message::NewJob(job) => {
                    // wroker got a job and is executing it
                    print_state(trace, format!("Worker {} got a job; executing...", id));
                    job();
                }
                Message::Terminate => {
                    //worker got a terminate message and is breaking the loop
                    print_state(
                        trace,
                        format!("Worker {} was told to terminate. terminating...", id),
                    );
                    break;
                }
            }
        });
        Worker {
            _id: id,
            thread: Some(thread),
        }
    }
}

impl ThreadPool {
    /// This function will return a new `ThreadPool` which will have `size` number of `workers(os threads)`.
    /// Turn the `trace` flag on to print the states of the workers.
    ///
    /// # Example
    ///
    /// ```
    /// use rahat3062_pool::ThreadPool; // Replace with your actual crate name
    ///
    /// // Create a ThreadPool with 4 workers and tracing enabled
    /// let pool = ThreadPool::new(4, true);
    /// ```
    ///
    /// This example creates a `ThreadPool` with 4 workers and enables tracing, which prints the states of the workers.
    pub fn new(size: usize, trace: bool) -> ThreadPool {
        assert!(size > 0);
        //create a vector whose capacity is size
        let mut workers = Vec::with_capacity(size);

        //create a channel to send the jobs to the workers
        let (sender, receiver) = mpsc::channel();

        //wrap the receiver in an Arc and Mutex to make it thread safe
        let receiver = Arc::new(Mutex::new(receiver));

        //create the threads and store them in the vector
        for id in 0..size {
            //create a new thread and store it in the vector
            workers.push(Worker::new(id, Arc::clone(&receiver), trace));
        }
        ThreadPool {
            workers,
            sender,
            trace,
        }
    }

    /// # Example
    ///
    /// ```
    /// use rahat3062_pool::ThreadPool;
    /// use std::sync::mpsc::channel;
    /// use std::time::Duration;
    ///
    /// let (sender, receiver) = channel();
    ///
    /// let pool = ThreadPool::new(4, true);
    ///
    /// for i in 0..10 {
    ///     let sender = sender.clone();
    ///     pool.execute(move || {
    ///         std::thread::sleep(Duration::from_millis(500));
    ///         sender.send(i + 1).unwrap();
    ///     });
    /// }
    ///
    /// // Explicitly drop the ThreadPool. This is not necessary, but it demonstrates that the jobs will continue to execute.
    /// drop(pool);
    ///
    /// // Close the channel to indicate that no more messages will be sent to make receiver.iter() unblocking.
    /// drop(sender);
    ///
    /// // Collect all the messages and check that we received the correct sum.
    /// let result: i32 = receiver.iter().sum();
    /// assert_eq!(result, 55); //((10*11)/2 = 55)
    /// ```
    ///
    /// This example creates a `ThreadPool` with 4 workers. It then creates 10 jobs that each sleep for a short time and then send a message on a channel. The `ThreadPool` is dropped immediately after the jobs are submitted, but the jobs continue to execute. Later, the main thread collects all the messages and checks that it received the correct sum. This test exercises the `ThreadPool`'s ability to run multiple jobs in parallel and shows that the jobs will continue to execute even if the `ThreadPool` is dropped.
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static, //the job will be called exactly once and is safe to pass between threads with no non static references
    {
        let job = Box::new(f); //forming the job pointer
        self.sender.send(Message::NewJob(job)).unwrap(); //send the job to the workers
    }
}

//when the threadpool is dropped we want to send a terminate message to all the workers
impl Drop for ThreadPool {
    /// This ensures all workers are terminated gracefully by finishing their current job before shutting down.
    fn drop(&mut self) {
        //get the trace
        let trace = self.trace;
        //clean up the workers
        print_state(trace, format!("Sending terminate message to all workers!"));
        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }
        //wait for all the workers to finish
        print_state(trace, format!("Waiting for all workers to finish..."));
        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc::channel;
    use std::time::Duration;

    #[test]
    fn test_execute() {
        let (sender, receiver) = channel();

        let pool = ThreadPool::new(4, false);

        for _ in 0..10 {
            let sender = sender.clone();
            pool.execute(move || {
                std::thread::sleep(Duration::from_millis(500));
                sender.send(1).unwrap();
            });
        }

        drop(sender); // Close the channel to indicate that no more messages will be sent

        let result: i32 = receiver.iter().sum();
        assert_eq!(result, 10);
    }
}
