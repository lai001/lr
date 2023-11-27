use super::{
    block_image::BlockImage, tile_index::TileIndex,
    virtual_texture_configuration::VirtualTextureConfiguration,
};
use crate::{thread_pool::ThreadPool, util::texture2d_from_rgba_image};
use std::{
    collections::HashMap,
    sync::{
        mpsc::{Receiver, Sender},
        Arc,
    },
};

#[derive(Debug)]
pub enum Error {
    ImageNotFound,
}

struct Message {
    path: Option<String>,
    key: Option<String>,
    tile_index: Option<TileIndex>,
    image: Option<Result<Arc<image::RgbaImage>, Error>>,
}

pub struct VirtualTextureAsyncLoader {
    load_receiver: Receiver<Message>,
    user_sender: Sender<Message>,
    textures: HashMap<String, HashMap<TileIndex, Arc<wgpu::Texture>>>,
}

impl VirtualTextureAsyncLoader {
    pub fn new(
        virtual_texture_configuration: VirtualTextureConfiguration,
    ) -> VirtualTextureAsyncLoader {
        let (video_sender, video_receiver) = std::sync::mpsc::channel();
        let (user_sender, user_receiver) = std::sync::mpsc::channel();

        let video_sender_clone = video_sender.clone();

        let cache = VirtualTextureAsyncLoader {
            load_receiver: video_receiver,
            user_sender,
            textures: HashMap::new(),
        };

        ThreadPool::virtual_texture_cache().spawn(move || {
            let sender = video_sender_clone;
            let receiver = user_receiver;
            let mut block_images: HashMap<String, BlockImage> = HashMap::new();
            let mut id: u32 = 0;
            loop {
                match receiver.recv() {
                    Ok(ref message) => {
                        if let (Some(key), Some(path)) =
                            (message.key.as_ref(), message.path.as_ref())
                        {
                            if block_images.contains_key(key) == false {
                                let block_image =
                                    BlockImage::new(&path, virtual_texture_configuration, id);
                                block_images.insert(key.to_string(), block_image);
                                id += 1;
                            }
                        } else if let (Some(key), Some(tile_index)) =
                            (message.key.as_ref(), message.tile_index.as_ref())
                        {
                            match block_images.get_mut(key) {
                                Some(block_image) => match block_image.get_image(*tile_index) {
                                    Some(page_image) => {
                                        let _ = sender.send(Message {
                                            path: None,
                                            key: Some(key.to_string()),
                                            tile_index: Some(tile_index.clone()),
                                            image: Some(Ok(page_image)),
                                        });
                                    }
                                    None => {
                                        let _ = sender.send(Message {
                                            path: None,
                                            key: Some(key.to_string()),
                                            tile_index: None,
                                            image: Some(Err(Error::ImageNotFound)),
                                        });
                                    }
                                },
                                None => {
                                    let _ = sender.send(Message {
                                        path: None,
                                        key: Some(key.to_string()),
                                        tile_index: None,
                                        image: Some(Err(Error::ImageNotFound)),
                                    });
                                }
                            }
                        }
                    }
                    Err(error) => {}
                }
            }
        });

        cache
    }

    pub fn push(&mut self, file_path: &str, key: &str) {
        match self.user_sender.send(Message {
            path: Some(file_path.to_string()),
            key: Some(key.to_string()),
            image: None,
            tile_index: None,
        }) {
            Ok(_) => {}
            Err(_) => {}
        }
    }

    pub fn get_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        key: &str,
        tile_index: &TileIndex,
    ) -> Option<Arc<wgpu::Texture>> {
        for message in self.load_receiver.try_iter() {
            if let (Some(key), Some(tile_index), Some(image)) = (
                message.key.as_ref(),
                message.tile_index.as_ref(),
                message.image.as_ref(),
            ) {
                if let Ok(image) = image.as_deref() {
                    let texture = texture2d_from_rgba_image(device, queue, image);
                    if self.textures.contains_key(key) {
                        self.textures
                            .get_mut(key)
                            .unwrap()
                            .insert(tile_index.clone(), Arc::new(texture));
                    } else {
                        let mut map: HashMap<TileIndex, Arc<wgpu::Texture>> = HashMap::new();
                        map.insert(tile_index.clone(), Arc::new(texture));
                        self.textures.insert(key.to_string(), map);
                    }
                }
            }
        }

        match self.textures.get_mut(key) {
            Some(map) => match map.get_mut(tile_index) {
                Some(texture) => Some(texture.clone()),
                None => {
                    let _ = self.user_sender.send(Message {
                        path: None,
                        key: Some(key.to_string()),
                        image: None,
                        tile_index: Some(tile_index.clone()),
                    });
                    None
                }
            },
            None => {
                let _ = self.user_sender.send(Message {
                    path: None,
                    key: Some(key.to_string()),
                    image: None,
                    tile_index: Some(tile_index.clone()),
                });
                None
            }
        }
    }
}
