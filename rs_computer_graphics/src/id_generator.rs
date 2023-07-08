pub struct IDGenerator {
    id: u64,
}

impl IDGenerator {
    pub fn new() -> IDGenerator {
        IDGenerator { id: 0 }
    }

    pub fn get_next_id(&mut self) -> u64 {
        let id = self.id;
        self.id += 1;
        id
    }
}
