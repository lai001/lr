pub trait Replicable {
    fn set_id(&mut self, id: uuid::Uuid);
    fn get_id(&self) -> &uuid::Uuid;
    fn on_replicated(&mut self, data: &(impl serde::Serialize + Clone));
    fn replicated(&mut self) -> impl serde::Serialize + Clone;
}
