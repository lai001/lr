pub trait NetworkReplicated {
    fn get_network_id(&self) -> &uuid::Uuid;
    fn set_network_id(&mut self, network_id: uuid::Uuid);
    fn is_replicated(&self) -> bool;
    fn set_replicated(&mut self, is_replicated: bool);
    fn on_replicated(&mut self) -> Vec<u8>;
    fn on_sync(&mut self, data: &Vec<u8>);
    fn debug_name(&self) -> Option<String>;
}

pub(crate) fn default_uuid() -> uuid::Uuid {
    uuid::Uuid::new_v4()
}
