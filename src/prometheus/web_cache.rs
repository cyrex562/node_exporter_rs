// Copyright 2021 The Prometheus Authors
// This code is partly borrowed from Caddy:
//    Copyright 2015 Matthew Holt and The Caddy Authors
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use rand::Rng;
use std::collections::HashMap;
use std::sync::{Mutex, Arc};

const CACHE_SIZE: usize = 100;

pub struct Cache {
    cache: Mutex<HashMap<String, bool>>,
}

impl Cache {
    pub fn new() -> Self {
        Cache {
            cache: Mutex::new(HashMap::new()),
        }
    }

    pub fn get(&self, key: &str) -> Option<bool> {
        let cache = self.cache.lock().unwrap();
        cache.get(key).copied()
    }

    pub fn set(&self, key: String, value: bool) {
        let mut cache = self.cache.lock().unwrap();
        self.make_room(&mut cache);
        cache.insert(key, value);
    }

    fn make_room(&self, cache: &mut HashMap<String, bool>) {
        if cache.len() < CACHE_SIZE {
            return;
        }

        let num_to_delete = (cache.len() / 10).max(1);
        let mut rng = rand::thread_rng();

        for _ in 0..num_to_delete {
            if let Some(key) = cache.keys().nth(rng.gen_range(0..cache.len())).cloned() {
                cache.remove(&key);
            }
        }
    }
}