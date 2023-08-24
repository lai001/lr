use crate::video_frame_extractor::{EVideoDecoderType, VideoFrame, VideoFrameExtractor};
use std::{
    collections::VecDeque,
    sync::mpsc::{Receiver, Sender},
};
struct Protocol {
    frame: Option<VideoFrame>,
    request_more_frames: Option<usize>,
    seek_time: Option<f32>,
    eof: Option<bool>,
}

pub struct VideoPlayerItem {
    filepath: String,
    video_receiver: Option<Receiver<Protocol>>,
    user_sender: Option<Sender<Protocol>>,
}

impl VideoPlayerItem {
    pub fn new(filepath: &str) -> VideoPlayerItem {
        let mut player = VideoPlayerItem {
            filepath: filepath.to_string(),
            video_receiver: None,
            user_sender: None,
        };
        player.init();
        player
    }

    fn init(&mut self) {
        let (video_sender, video_receiver) = std::sync::mpsc::channel();
        let (user_sender, user_receiver): (Sender<Protocol>, Receiver<Protocol>) =
            std::sync::mpsc::channel();

        let video_sender_clone = video_sender.clone();

        let filepath = self.filepath.to_string();
        std::thread::spawn(move || {
            let mut video_frame_extractor =
                VideoFrameExtractor::new(&filepath, Some(EVideoDecoderType::Hardware));
            let mut resp_protocols: VecDeque<Protocol> = VecDeque::new();

            loop {
                let mut req_protocols: Vec<Protocol> = vec![];

                match user_receiver.recv() {
                    Ok(protocol) => req_protocols.push(protocol),
                    Err(_) => break,
                }
                req_protocols
                    .append(&mut user_receiver.try_iter().map(|element| element).collect());

                let seek_protocols: Vec<&Protocol> = req_protocols
                    .iter()
                    .filter(|element| {
                        if element.seek_time.is_none() == false {
                            true
                        } else {
                            false
                        }
                    })
                    .collect();

                let request_more_frames_protocols: Vec<&Protocol> = req_protocols
                    .iter()
                    .filter(|element| {
                        if element.request_more_frames.is_none() == false {
                            true
                        } else {
                            false
                        }
                    })
                    .collect();

                if let Some(seek_protocol) = seek_protocols.last() {
                    resp_protocols.clear();
                    video_frame_extractor.seek(seek_protocol.seek_time.unwrap());

                    while resp_protocols.is_empty() {
                        match video_frame_extractor.next_frames() {
                            Some(frames) => {
                                for frame in frames {
                                    if frame.get_time_range_second().end
                                        >= seek_protocol.seek_time.unwrap()
                                    {
                                        resp_protocols.push_back(Protocol {
                                            frame: Some(frame),
                                            request_more_frames: None,
                                            seek_time: None,
                                            eof: None,
                                        });
                                    }
                                }
                            }
                            None => {
                                resp_protocols.push_back(Protocol {
                                    frame: None,
                                    request_more_frames: None,
                                    seek_time: None,
                                    eof: Some(true),
                                });
                                break;
                            }
                        }
                    }
                    if let Some(resp_protocol) = resp_protocols.pop_front() {
                        let _ = video_sender_clone.send(resp_protocol);
                    }
                } else if request_more_frames_protocols.is_empty() == false {
                    while resp_protocols.len() < 6 {
                        match video_frame_extractor.next_frames() {
                            Some(frames) => {
                                for frame in frames {
                                    resp_protocols.push_back(Protocol {
                                        frame: Some(frame),
                                        request_more_frames: None,
                                        seek_time: None,
                                        eof: Some(true),
                                    });
                                }
                            }
                            None => {
                                resp_protocols.push_back(Protocol {
                                    frame: None,
                                    request_more_frames: None,
                                    seek_time: None,
                                    eof: Some(true),
                                });
                                break;
                            }
                        }
                    }

                    if let Some(resp_protocol) = resp_protocols.pop_front() {
                        let _ = video_sender_clone.send(resp_protocol);
                    }
                }
            }
        });
        self.video_receiver = Some(video_receiver);
        self.user_sender = Some(user_sender);
    }

    pub fn try_recv(&mut self) -> Result<VideoFrame, crate::error::Error> {
        let protocal_result = self.video_receiver.as_ref().unwrap().try_recv();
        match protocal_result {
            Ok(protocal) => {
                if let Some(frame) = protocal.frame {
                    let _ = self.user_sender.as_ref().unwrap().send(Protocol {
                        frame: None,
                        request_more_frames: Some(1),
                        seek_time: None,
                        eof: None,
                    });
                    Ok(frame)
                } else if let Some(_) = protocal.eof {
                    Err(crate::error::Error::EndOfFile)
                } else {
                    panic!()
                }
            }
            Err(error) => match error {
                std::sync::mpsc::TryRecvError::Empty => {
                    let _ = self.user_sender.as_ref().unwrap().send(Protocol {
                        frame: None,
                        request_more_frames: Some(1),
                        seek_time: None,
                        eof: None,
                    });
                    Err(crate::error::Error::TryAgain)
                }
                std::sync::mpsc::TryRecvError::Disconnected => {
                    Err(crate::error::Error::Disconnected)
                }
            },
        }
    }

    pub fn seek(&mut self, time: f32) {
        self.video_receiver.as_ref().unwrap().try_iter();
        let _ = self.user_sender.as_ref().unwrap().send(Protocol {
            frame: None,
            request_more_frames: None,
            seek_time: Some(time),
            eof: None,
        });
    }
}
