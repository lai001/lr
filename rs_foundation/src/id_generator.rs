use std::collections::VecDeque;

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

pub struct ReusableIdGenerator {
    next_id: u64,
    free_list: VecDeque<u64>,
}

impl ReusableIdGenerator {
    pub fn new(start_id: u64) -> Self {
        Self {
            next_id: start_id,
            free_list: VecDeque::new(),
        }
    }

    pub fn next_id(&mut self) -> u64 {
        if let Some(id) = self.free_list.pop_front() {
            id
        } else {
            let id = self.next_id;
            #[cfg(debug_assertions)]
            {
                self.next_id = self.next_id.strict_add(1);
            }
            #[cfg(not(debug_assertions))]
            {
                self.next_id += 1;
            }
            id
        }
    }

    pub fn free(&mut self, id: u64) -> bool {
        #[cfg(not(test))]
        {
            debug_assert!(!self.free_list.contains(&id) && id < self.next_id);
        }
        if !self.free_list.contains(&id) && id < self.next_id {
            self.free_list.push_back(id);
            return true;
        }
        false
    }
}

#[cfg(test)]
mod test {
    use crate::id_generator::ReusableIdGenerator;
    #[test]
    fn reusable_id_generator_test() {
        let mut id_generator = ReusableIdGenerator::new(1);
        let id1 = id_generator.next_id();
        let id2 = id_generator.next_id();
        assert_eq!(1, id1);
        assert_eq!(2, id2);
        assert!(id_generator.free(id1));
        assert_eq!(false, id_generator.free(5));
        let id3 = id_generator.next_id();
        assert_eq!(id1, id3);
    }
}
