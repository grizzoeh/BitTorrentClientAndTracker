use crate::errors::threadpool_error::ThreadPoolError;
use std::{
    sync::{
        mpsc::{self, channel, Receiver, Sender},
        Arc,
    },
    thread::{self, JoinHandle},
    time::Duration,
};
type WorkerId = usize;
pub(crate) type Job = Box<dyn FnOnce() + Send + 'static>;

// How much time should wait before checking if a thread has panicked
// if too long, it can lose a thread without noticing
// if too short, spend too much time checking if a thread has panicked
const THREAD_WAIT_TIMEOUT: Duration = Duration::from_micros(1000); // 0.001s

#[derive(Clone)]
pub struct ThreadPool {
    job_sender: Sender<Job>, // to send tasks to the ThreadManager
    _thread_manager_handler: Arc<ManagerHandle>, // ThreadManager handler
}

impl ThreadPool {
    /// The threadpool uses an extra thread for internal processing (ThreadManager).
    pub fn new(amount: usize) -> ThreadPool {
        let (sender, receiver): (Sender<Job>, Receiver<Job>) = mpsc::channel();
        let handler = thread::spawn(move || {
            ThreadManager::new(amount, receiver).run();
        });

        ThreadPool {
            job_sender: sender,
            _thread_manager_handler: Arc::new(ManagerHandle(Some(handler))),
        }
    }

    /// Submits a job to the thread pool.
    pub fn execute<F>(&self, job: F) -> Result<(), ThreadPoolError>
    where
        F: FnOnce() + Send + 'static,
    {
        self.job_sender.send(Box::new(job))?;
        Ok(())
    }
}

// intermidiate between ThreadPool interface and worker threads
struct ThreadManager {
    threads: Vec<ThreadInfo>,
    ready_receiver: Receiver<WorkerId>, // to receive idle worker id
    job_receiver: Receiver<Job>,        // to receive tasks
    ready_sender: Sender<WorkerId>, // used in case of restarting a worker that has panicked, he will use it to send his id
}

// store ThreadManager handler to be able to join it when dropped
struct ManagerHandle(Option<JoinHandle<()>>);

impl Drop for ManagerHandle {
    fn drop(&mut self) {
        if let Some(handle) = self.0.take() {
            let _res = handle.join();
        }
    }
}

impl ThreadManager {
    fn new(amount: usize, job_receiver: Receiver<Job>) -> Self {
        let (ready_sender, ready_receiver) = channel();

        let ready_sender_clone = ready_sender.clone();
        let threads = Self::initialize_threads(amount, ready_sender_clone);
        ThreadManager {
            threads,
            ready_receiver,
            job_receiver,
            ready_sender,
        }
    }

    // waits for a job until job_sender is closed
    fn run(&mut self) {
        while let Ok(job) = self.job_receiver.recv() {
            let i = self.get_free_thread();
            let _res = self.threads[i].job_sender.send(job);
        }
    }

    // get idle thread index
    // in case of no idle thread, try to revive threads that may have panicked
    fn get_free_thread(&mut self) -> WorkerId {
        let mut resultado = self.ready_receiver.recv_timeout(THREAD_WAIT_TIMEOUT);
        while resultado.is_err() {
            // check if any thread has panicked
            self.recover_threads();
            resultado = self.ready_receiver.recv_timeout(THREAD_WAIT_TIMEOUT);
        }
        resultado.unwrap()
    }

    // revive threads that may have panicked
    fn recover_threads(&mut self) {
        for (id, thread) in self.threads.iter_mut().enumerate() {
            if thread
                .handler
                .as_ref()
                .map(|h| -> bool { h.is_finished() })
                .unwrap_or(true)
            {
                Self::reset_thread(thread, id, self.ready_sender.clone());
            }
        }
    }

    // revive thread joining it and restarting it. Updates channels
    fn reset_thread(thread: &mut ThreadInfo, id: WorkerId, ready_sender: Sender<WorkerId>) {
        if let Some(handle) = thread.handler.take() {
            let _res = handle.join();
        }

        let (job_sender, job_receiver) = channel();

        thread.handler = Some(thread::spawn(move || {
            worker(job_receiver, ready_sender, id)
        }));

        thread.job_sender = job_sender;
    }

    // init threads with their communication channels
    fn initialize_threads(amount: usize, ready_sender: Sender<WorkerId>) -> Vec<ThreadInfo> {
        let mut threads = Vec::new();
        for i in 0..amount {
            let (job_sender, job_receiver) = channel();
            let rs = ready_sender.clone();

            let handler = thread::spawn(move || worker(job_receiver, rs, i));

            threads.push(ThreadInfo {
                handler: Some(handler),
                job_sender,
            });
        }
        threads
    }
}

impl Drop for ThreadManager {
    fn drop(&mut self) {
        while let Some(mut thread) = self.threads.pop() {
            if let Some(handle) = thread.handler.take() {
                drop(thread);
                // when dropped, run() loop will be exited
                let _ = handle.join();
            }
        }
    }
}

// ThreadManager stores information about each worker thread
struct ThreadInfo {
    handler: Option<JoinHandle<()>>, // El handler para hacer join al thread
    job_sender: Sender<Job>,         // El canal para envíar tareas al thread
}

// each thread waits for a job
fn worker(job_receiver: Receiver<Job>, ready_sender: Sender<WorkerId>, id: WorkerId) {
    if let Err(_err) = ready_sender.send(id) {
        return;
    }

    for job in job_receiver {
        job();
        if let Err(_err) = ready_sender.send(id) {
            // if fails it can be restarted later
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ThreadPool;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_simple_sum() {
        let n1 = Arc::new(Mutex::new(0));
        let mut n2 = 0;

        let threadpool = ThreadPool::new(10);
        for i in 0..100 {
            n2 += i;
            let n1_copy = n1.clone();
            let _exe_ret = threadpool.execute(move || {
                *n1_copy.lock().unwrap() += i;
            });
        }
        drop(threadpool);

        assert_eq!(*n1.lock().unwrap(), n2);
    }

    #[test]
    fn test_panic_and_sum() {
        let threadpool = ThreadPool::new(10);
        for _ in 0..30 {
            let _exe_ret = threadpool.execute(move || {
                panic!("test");
            });
        }
        // should recover threads to execute the following jobs
        let n1 = Arc::new(Mutex::new(0));
        let mut n2 = 0;
        for i in 0..100 {
            n2 += i;
            let n1_copy = n1.clone();
            let _exe_ret = threadpool.execute(move || {
                *n1_copy.lock().unwrap() += i;
            });
        }
        drop(threadpool);

        assert_eq!(*n1.lock().unwrap(), n2);
    }

    #[test]
    fn test_clone_threadpool() {
        let threadpool_1 = ThreadPool::new(10);
        let threadpool_2 = threadpool_1.clone();
        let n1 = Arc::new(Mutex::new(0));
        let mut n2 = 0;
        let n3 = Arc::new(Mutex::new(100));

        for i in 0..100 {
            // first threadpool will sum numbers
            n2 += i;
            let n1_copy = n1.clone();
            let _exe_ret = threadpool_1.execute(move || {
                *n1_copy.lock().unwrap() += i;
            });
            // second threadpool will subtract numbers
            let n3_copy = n3.clone();
            let _exe_ret = threadpool_2.execute(move || {
                *n3_copy.lock().unwrap() -= 1;
            });
        }
        drop(threadpool_1);
        drop(threadpool_2);

        assert_eq!(*n1.lock().unwrap(), n2);
        assert_eq!(*n3.lock().unwrap(), 0);
    }
}
