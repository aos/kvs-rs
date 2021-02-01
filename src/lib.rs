use std::collections::HashMap;
pub struct KvStore {
    map: HashMap<String, String>,
}

impl KvStore {
    pub fn new() -> Self {
        KvStore {
            map: HashMap::new()
        }
    }

    pub fn get(&self, key: String) -> Option<String> {
        match self.map.get(&key) {
            Some(v) => Some(v.clone()),
            None => None
        }
    }

    pub fn set(&mut self, key: String, value: String) {
        self.map.insert(key, value);
    }

    pub fn remove(&mut self, key: String) {
        self.map.remove(&key);
    }
}
