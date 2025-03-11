use std::{cell::RefCell, collections::HashMap};

use noise::{NoiseFn, Seedable};

pub struct FullCache<T> {
    source: T,
    cache_map: RefCell<HashMap<[i64; 2], f64>>,
}

impl<T> FullCache<T> {
    pub fn new(source: T) -> Self {
        Self {
            source,
            cache_map: RefCell::new(HashMap::new()),
        }
    }
}

impl<T> Default for FullCache<T>
where
    T: Default + Seedable,
{
    fn default() -> Self {
        Self {
            source: Default::default(),
            cache_map: RefCell::new(HashMap::new()),
        }
    }
}

impl<T> Seedable for FullCache<T>
where
    T: Default + Seedable,
{
    fn set_seed(mut self, seed: u32) -> Self {
        self.source = T::default().set_seed(seed);
        self
    }

    fn seed(&self) -> u32 {
        self.source.seed()
    }
}

impl<T> NoiseFn<f64, 2usize> for FullCache<T>
where
    T: NoiseFn<f64, 2usize>,
{
    fn get(&self, point: [f64; 2usize]) -> f64 {
        let cache_key = [point[0] as i64, point[1] as i64];
        let mut map = self.cache_map.borrow_mut();
        let item = map.get(&cache_key);
        if let Some(cache_value) = item {
            return *cache_value;
        }
        let value = self.source.get(point);
        map.insert(cache_key, value);
        return value;
    }
}
