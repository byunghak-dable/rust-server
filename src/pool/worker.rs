use super::Message;
use std::{
    error::Error,
    sync::{mpsc::Receiver, Arc, Mutex},
    thread::{self, JoinHandle},
};
enum WorkMsg {
    Done,
    Terminate,
}
pub(super) struct Worker {
    pub(super) id: usize,
    pub(super) thread: Option<JoinHandle<()>>,
}

impl Worker {
    pub(super) fn new<T>(id: usize, receiver: Arc<Mutex<Receiver<Message<T>>>>) -> Self
    where
        T: FnOnce() + Send + 'static,
    {
        let thread = thread::spawn(move || loop {
            let work_msg = match Self::work(&receiver) {
                Ok(process) => process,
                Err(err) => {
                    eprintln!("work failed: {:?}", err);
                    continue;
                }
            };
            if let WorkMsg::Terminate = work_msg {
                eprintln!("terminate msg received");
                break;
            }
        });
        Self {
            id,
            thread: Some(thread),
        }
    }

    fn work<T>(receiver: &Arc<Mutex<Receiver<Message<T>>>>) -> Result<WorkMsg, Box<dyn Error + '_>>
    where
        T: FnOnce() + Send + 'static,
    {
        let msg = receiver.lock()?.recv()?;
        match msg {
            Message::Job(job) => job(),
            Message::Terminate => return Ok(WorkMsg::Terminate),
        }
        Ok(WorkMsg::Done)
    }
}
