use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::sync::Mutex;

pub struct SyncWait {
    owned_thread_sender: Mutex<Sender<()>>,
    parallelism_thread_receiver: Mutex<Receiver<()>>,
    parallelism_thread_sender: Mutex<Sender<()>>,
    owned_thread_receiver: Mutex<Receiver<()>>,
}

impl SyncWait {
    pub fn shared() -> Arc<SyncWait> {
        let (owned_thread_sender, parallelism_thread_receiver) = channel::<()>();
        let (parallelism_thread_sender, owned_thread_receiver) = channel::<()>();
        Arc::new(SyncWait {
            owned_thread_sender: Mutex::new(owned_thread_sender),
            parallelism_thread_receiver: Mutex::new(parallelism_thread_receiver),
            parallelism_thread_sender: Mutex::new(parallelism_thread_sender),
            owned_thread_receiver: Mutex::new(owned_thread_receiver),
        })
    }

    pub fn send_stop_signal_and_wait(&self) {
        match self.owned_thread_sender.lock().unwrap().send(()) {
            Ok(_) => match self.owned_thread_receiver.lock().unwrap().recv() {
                Ok(_) => {}
                Err(error) => panic!("{}", error),
            },
            Err(error) => panic!("{}", error),
        }
    }

    pub fn is_stop(&self) -> bool {
        match self.parallelism_thread_receiver.lock().unwrap().try_recv() {
            Ok(_) => true,
            Err(error) => match error {
                std::sync::mpsc::TryRecvError::Empty => false,
                std::sync::mpsc::TryRecvError::Disconnected => true,
            },
        }
    }

    pub fn finish(&self) {
        match self.parallelism_thread_sender.lock().unwrap().send(()) {
            Ok(_) => {}
            Err(error) => panic!("{}", error),
        }
    }
}

pub struct MultipleSyncWait {
    inner: Arc<SyncWait>,
    count: Arc<Mutex<u32>>,
}

impl MultipleSyncWait {
    pub fn new() -> MultipleSyncWait {
        MultipleSyncWait {
            inner: SyncWait::shared(),
            count: Arc::new(Mutex::new(0)),
        }
    }

    pub fn clone(&self) -> MultipleSyncWait {
        let count = self.count.clone();
        *count.lock().unwrap() += 1;
        MultipleSyncWait {
            inner: self.inner.clone(),
            count,
        }
    }

    pub fn send_stop_signal_and_wait(&self) {
        let mut count = *self.count.lock().unwrap();
        while count > 0 {
            self.inner.send_stop_signal_and_wait();
            count -= 1
        }
    }

    pub fn is_stop(&self) -> bool {
        self.inner.is_stop()
    }

    pub fn finish(&self) {
        self.inner.finish()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::thread_pool::ThreadPool;

    #[test]
    fn test_case_multiple_sync_wait0() {
        let wait = MultipleSyncWait::new();
        for i in 1..=50 {
            ThreadPool::global().spawn({
                let wait = wait.clone();
                move || {
                    let is_stop = wait.is_stop();
                    if is_stop {
                        println!("Stop work. {}", i);
                    } else {
                        std::thread::sleep(std::time::Duration::from_secs_f32(1.0f32));
                        println!("Finish work. {}", i);
                    }
                    wait.finish();
                }
            });
        }
        wait.send_stop_signal_and_wait();
        println!("Exit.");
    }

    #[test]
    fn test_case_multiple_sync_wait1() {
        let wait = MultipleSyncWait::new();
        for i in 1..=50 {
            ThreadPool::global().spawn({
                let wait = wait.clone();
                move || {
                    std::thread::sleep(std::time::Duration::from_secs_f32(1.0f32));
                    println!("Finish work. {}", i);
                    wait.finish();
                }
            });
        }
        wait.send_stop_signal_and_wait();
        println!("Exit.");
    }

    #[test]
    fn test_case_multiple_sync_wait2() {
        let wait = MultipleSyncWait::new();
        for i in 1..=10 {
            ThreadPool::global().spawn({
                let wait = wait.clone();
                move || {
                    std::thread::sleep(std::time::Duration::from_secs_f32(1.0f32));
                    println!("Finish work. {}", i);
                    wait.finish();
                }
            });
        }
        std::thread::sleep(std::time::Duration::from_secs_f32(10.0f32));
        wait.send_stop_signal_and_wait();
        println!("Exit.");
    }

    #[test]
    fn test_case_multiple_sync_wait3() {
        let wait = MultipleSyncWait::new();
        for i in 1..=10 {
            ThreadPool::global().spawn({
                let wait = wait.clone();
                move || {
                    std::thread::sleep(std::time::Duration::from_secs_f32(1.0f32));
                    println!("Finish work. {}", i);
                    wait.finish();
                }
            });
        }
        std::thread::sleep(std::time::Duration::from_secs_f32(10.0f32));
        wait.send_stop_signal_and_wait();
        println!("Exit.");
    }

    #[test]
    fn test_case_sync_wait0() {
        let sync_wait = SyncWait::shared();
        ThreadPool::render().spawn({
            let sync_wait = sync_wait.clone();
            move || {
                println!("Finish work. {}", sync_wait.is_stop());
                std::thread::sleep(std::time::Duration::from_secs_f32(1.0f32));
                println!("Finish work. {}", sync_wait.is_stop());
                sync_wait.finish();
            }
        });
        std::thread::sleep(std::time::Duration::from_secs_f32(3.0f32));
        sync_wait.send_stop_signal_and_wait();
        println!("Exit.");
    }

    #[test]
    fn test_case_sync_wait1() {
        let sync_wait = SyncWait::shared();
        ThreadPool::render().spawn({
            let sync_wait = sync_wait.clone();
            move || {
                std::thread::sleep(std::time::Duration::from_secs_f32(1.0f32));
                println!("Finish work.");
                sync_wait.finish();
            }
        });
        sync_wait.send_stop_signal_and_wait();
        println!("Exit.");
    }

    #[test]
    fn test_case_sync_wait2() {
        let sync_wait = SyncWait::shared();
        ThreadPool::render().spawn({
            let sync_wait = sync_wait.clone();
            move || {
                while sync_wait.is_stop() == false {
                    println!("Working.");
                    std::thread::sleep(std::time::Duration::from_secs_f32(1.0f32));
                }
                println!("Finish work.");
                sync_wait.finish();
            }
        });
        std::thread::sleep(std::time::Duration::from_secs_f32(3.0f32));
        sync_wait.send_stop_signal_and_wait();
        println!("Exit.");
    }
}
