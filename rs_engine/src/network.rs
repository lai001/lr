use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
pub enum ENetMode {
    #[default]
    Server,
    Client,
}

pub trait NetworkReplicated {
    fn get_network_id(&self) -> &uuid::Uuid;

    fn set_network_id(&mut self, network_id: uuid::Uuid);

    fn is_replicated(&self) -> bool;

    fn set_replicated(&mut self, is_replicated: bool);

    fn on_replicated(&mut self) -> Vec<u8> {
        vec![]
    }

    fn on_sync(&mut self, data: &Vec<u8>) {
        let _ = data;
    }

    fn call(&mut self) -> Vec<u8> {
        vec![]
    }

    fn on_call(&mut self, data: &Vec<u8>) {
        let _ = data;
    }

    fn debug_name(&self) -> Option<String> {
        None
    }

    fn sync_with_server(&mut self, is_sync: bool);

    fn is_sync_with_server(&self) -> bool;

    fn on_net_mode_changed(&mut self, net_mode: ENetMode) {
        let _ = net_mode;
    }
}

pub trait NetworkModule {
    fn on_new_connections(&mut self, connections: &[rs_network::server::Connection]) {
        let _ = connections;
    }
}

pub(crate) fn default_uuid() -> uuid::Uuid {
    uuid::Uuid::new_v4()
}
