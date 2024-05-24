use std::sync::{
    mpsc::{sync_channel, Receiver, SyncSender, TryRecvError},
    Arc, Mutex,
};

pub enum SingleConsumeChnnelAPayload<T> {
    Message(T),
    Stop,
}

pub enum SingleConsumeChnnelBPayload<T> {
    Message(T),
    DidStop,
}

pub struct SingleConsumeChnnel<T, U> {
    a_thread_sender: Mutex<SyncSender<SingleConsumeChnnelAPayload<T>>>,
    b_thread_receiver: Mutex<Receiver<SingleConsumeChnnelAPayload<T>>>,
    b_thread_sender: Mutex<SyncSender<SingleConsumeChnnelBPayload<U>>>,
    a_thread_receiver: Mutex<Receiver<SingleConsumeChnnelBPayload<U>>>,
}

impl<T, U> SingleConsumeChnnel<T, U> {
    pub fn shared(
        a_bound: Option<usize>,
        b_bound: Option<usize>,
    ) -> Arc<SingleConsumeChnnel<T, U>> {
        let (a_thread_sender, b_thread_receiver) =
            sync_channel(a_bound.unwrap_or(u8::MAX as usize));
        let (b_thread_sender, a_receiver) = sync_channel(b_bound.unwrap_or(u8::MAX as usize));
        Arc::new(SingleConsumeChnnel::<T, U> {
            a_thread_sender: Mutex::new(a_thread_sender),
            b_thread_receiver: Mutex::new(b_thread_receiver),
            b_thread_sender: Mutex::new(b_thread_sender),
            a_thread_receiver: Mutex::new(a_receiver),
        })
    }

    pub fn send_stop_signal_and_wait(&self) {
        match self
            .a_thread_sender
            .lock()
            .unwrap()
            .send(SingleConsumeChnnelAPayload::Stop)
        {
            Ok(_) => loop {
                match self.a_thread_receiver.lock().unwrap().recv() {
                    Ok(payload) => match payload {
                        SingleConsumeChnnelBPayload::Message(_) => {}
                        SingleConsumeChnnelBPayload::DidStop => {
                            break;
                        }
                    },
                    Err(error) => panic!("{}", error),
                }
            },
            Err(error) => panic!("{}", error),
        }
    }

    pub fn from_b_try_recv(&self) -> Result<U, TryRecvError> {
        match self.a_thread_receiver.lock().unwrap().try_recv() {
            Ok(payload) => match payload {
                SingleConsumeChnnelBPayload::Message(message) => Ok(message),
                SingleConsumeChnnelBPayload::DidStop => panic!(),
            },
            Err(error) => Err(error),
        }
    }

    pub fn to_b(&self, message: T) {
        let _ = self
            .a_thread_sender
            .lock()
            .unwrap()
            .send(SingleConsumeChnnelAPayload::Message(message));
    }

    pub fn to_a(&self, message: U) {
        let _ = self
            .b_thread_sender
            .lock()
            .unwrap()
            .send(SingleConsumeChnnelBPayload::Message(message));
    }

    pub fn try_to_a(
        &self,
        message: U,
    ) -> Result<(), std::sync::mpsc::TrySendError<SingleConsumeChnnelBPayload<U>>> {
        self.b_thread_sender
            .lock()
            .unwrap()
            .try_send(SingleConsumeChnnelBPayload::Message(message))
    }

    pub fn from_a_block_current_thread<F>(&self, mut closure: F)
    where
        F: FnMut(T) -> (),
    {
        loop {
            match self.b_thread_receiver.lock().unwrap().recv() {
                Ok(payload) => match payload {
                    SingleConsumeChnnelAPayload::Message(message) => {
                        closure(message);
                    }
                    SingleConsumeChnnelAPayload::Stop => {
                        break;
                    }
                },
                Err(_) => {
                    break;
                }
            }
        }
        let _ = self
            .b_thread_sender
            .lock()
            .unwrap()
            .send(SingleConsumeChnnelBPayload::DidStop);
    }
}
