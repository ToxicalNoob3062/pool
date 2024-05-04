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

//threadpool will have a group of workers who will be listening for jobs
//the threadpool will have a sender which will be used to send Messages to the workers
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

//make the workers ready to listen for messages during their creation
impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Message>>>) -> Worker {
        //span a new thread for the worker and keep it alive infinitely with loop
        //here both the lock and the recv are blocking calls
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();
            println!("Worker {} got a job; executing.", id);
            match message {
                Message::NewJob(job) => {
                    println!("Worker {} got a job; executing.", id);
                    job();
                }
                Message::Terminate => {
                    println!("Worker {} was told to terminate.", id);
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
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        //create a vector whoose capcity is size
        let mut workers = Vec::with_capacity(size);

        //create a channel to send the jobs to the workers
        let (sender, receiver) = mpsc::channel();

        //wrap the receiver in an Arc and Mutex to make it thread safe
        let receiver = Arc::new(Mutex::new(receiver));

        //create the threads and store them in the vector
        for id in 0..size {
            //create a new thread and store it in the vector
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
        ThreadPool { workers, sender }
    }

    //send a job to the workers
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
    fn drop(&mut self) {
        //clean up the workers
        println!("Sending terminate message to all workers.");
        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }
        println!("Shutting down all workers.");
        //wait for all the workers to finish
        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}
