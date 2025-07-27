use std::collections::HashMap;
use winit::{event::ElementState, keyboard::KeyCode};

pub struct KeysDetector {
    virtual_key_code_states: HashMap<KeyCode, ElementState>,
}

impl KeysDetector {
    pub fn new() -> KeysDetector {
        KeysDetector {
            virtual_key_code_states: HashMap::new(),
        }
    }

    pub fn on_key(&mut self, key_code: KeyCode, element_state: ElementState) {
        self.virtual_key_code_states.insert(key_code, element_state);
    }

    pub fn is_keys_pressed(&mut self, keys: &[KeyCode], is_consume: bool) -> bool {
        let mut states: HashMap<KeyCode, ElementState> = HashMap::new();
        for key in keys {
            if let Some(state) = self.virtual_key_code_states.get(key) {
                states.insert(*key, *state);
            }
        }
        if states.keys().len() == keys.len() {
            for state in states.values() {
                if *state == ElementState::Released {
                    return false;
                }
            }
            if is_consume {
                for key in states.keys() {
                    self.virtual_key_code_states.remove(key);
                }
            }
            true
        } else {
            false
        }
    }

    pub fn consume_key(&mut self, key: &KeyCode) {
        self.virtual_key_code_states.remove(key);
    }

    pub fn consume_keys(&mut self, keys: &[KeyCode]) {
        for key in keys {
            self.virtual_key_code_states.remove(key);
        }
    }

    pub fn virtual_key_code_states(&self) -> &HashMap<KeyCode, ElementState> {
        &self.virtual_key_code_states
    }

    pub fn virtual_key_code_states_mut(&mut self) -> &mut HashMap<KeyCode, ElementState> {
        &mut self.virtual_key_code_states
    }
}
