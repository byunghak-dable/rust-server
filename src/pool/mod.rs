mod worker;
use self::worker::Worker;
use std::sync::{
    mpsc::{self, SendError, Sender},
    Arc, Mutex,
};
pub enum Message<T>
where
    T: FnOnce() + Send + 'static,
{
    Job(T),
    Terminate,
}

pub struct Pool<T>
where
    T: FnOnce() + Send + 'static,
{
    workers: Vec<Worker>,
    sender: Sender<Message<T>>,
}

impl<T> Pool<T>
where
    T: FnOnce() + Send + 'static,
{
    pub fn new(size: usize) -> Result<Self, &'static str> {
        if size == 0 {
            return Err("need to have more than 1");
        }
        let mut workers: Vec<Worker> = Vec::with_capacity(size);
        let (sender, receiver) = mpsc::channel();
        let m_receiver = Arc::new(Mutex::new(receiver));
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&m_receiver)));
        }
        Ok(Self { workers, sender })
    }

    pub fn execute(&self, f: T) -> Result<(), SendError<Message<T>>> {
        self.sender.send(Message::Job(f))?;
        Ok(())
    }
}

impl<T> Drop for Pool<T>
where
    T: FnOnce() + Send + 'static,
{
    fn drop(&mut self) {
        println!("Sending terminate message to all workers.");
        self.workers.iter().for_each(|_| {
            if let Err(err) = self.sender.send(Message::Terminate) {
                eprintln!("sending terminate msg failed: {:?}", err);
            }
        });

        self.workers.iter_mut().for_each(|worker| {
            eprintln!("shutting down worker: id-{:?}", worker.id);
            if let Some(thread) = worker.thread.take() {
                if let Err(err) = thread.join() {
                    eprintln!("worker thread join failed: {:?}", err);
                }
            }
        });
    }
}
