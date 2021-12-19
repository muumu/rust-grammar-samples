use std::thread;
use std::cmp;
use std::sync::{Mutex, Arc, RwLock};
use std::sync::atomic;
use std::sync::atomic::AtomicUsize;
use std::marker::{Sync, Send};

use crate::indexed_value::IndexedValue;

pub struct MutexCache {
    cache: Vec<Mutex<(usize, u64)>>,
    len: usize
}

pub struct RwLockCache {
    cache: Vec<RwLock<(usize, u64)>>,
    len: usize
}

pub struct CounterCache {
    cache: Vec<RwLock<(usize, u64)>>,
    len: usize,
    counter: AtomicUsize,
    hit: AtomicUsize
}

pub struct NoCache { }

pub trait Cache {
    fn with_len(len: usize) -> Self;
    fn len(&self) -> usize;
    fn get(&self, i: usize) -> (usize, u64);
    fn set(&self, i: usize, data: (usize, u64));
}

impl Cache for RwLockCache {
    fn with_len(len: usize) -> Self {
        let mut cache = Vec::<RwLock<(usize, u64)>>::with_capacity(len);
        for _ in 0..len {
            cache.push(RwLock::new((0, 0)));
        }
        Self { cache, len }
    }
    fn len(&self) -> usize { self.len }
    fn get(&self, i: usize) -> (usize, u64) {
        if i < self.len {
            *self.cache[i].read().unwrap()
        } else {
            (0, 0)
        }
    }
    fn set(&self, i: usize, data: (usize, u64)) {
        if i < self.len {
            *self.cache[i].write().unwrap() = data;
        }
    }
}

impl Cache for MutexCache {
    fn with_len(len: usize) -> Self {
        let mut cache = Vec::<Mutex<(usize, u64)>>::with_capacity(len);
        for _ in 0..len {
            cache.push(Mutex::new((0, 0)));
        }
        Self { cache, len }
    }
    fn len(&self) -> usize { self.len }
    fn get(&self, i: usize) -> (usize, u64) {
        if i < self.len {
            *self.cache[i].lock().unwrap()
        } else {
            (0, 0)
        }
    }
    fn set(&self, i: usize, data: (usize, u64)) {
        if i < self.len {
            *self.cache[i].lock().unwrap() = data;
        }
    }
}

impl Cache for CounterCache {
    fn with_len(len: usize) -> Self {
        let mut cache = Vec::<RwLock<(usize, u64)>>::with_capacity(len);
        for _ in 0..len {
            cache.push(RwLock::new((0, 0)));
        }
        Self { cache, len, counter: AtomicUsize::new(0), hit: AtomicUsize::new(0) }
    }
    fn len(&self) -> usize { self.len }
    fn get(&self, i: usize) -> (usize, u64) {
        self.counter.fetch_add(1, atomic::Ordering::Relaxed);
        if i < self.len {
            let r = *self.cache[i].read().unwrap();
            if r.0 > 0 {
                self.hit.fetch_add(1, atomic::Ordering::Relaxed);
            }
            r
        } else {
            (0, 0)
        }
    }
    fn set(&self, i: usize, data: (usize, u64)) {
        if i < self.len {
            *self.cache[i].write().unwrap() = data;
        }
    }
}

impl CounterCache {
    fn stats(&self) {
        println!("cache try = {}", self.counter.load(atomic::Ordering::Relaxed));
        println!("cache hit = {}", self.hit.load(atomic::Ordering::Relaxed));
    }
}

impl Cache for NoCache {
    fn with_len(_: usize) -> Self { Self { } }
    fn len(&self) -> usize { 0 }
    fn get(&self, _: usize) -> (usize, u64) { (0, 0) }
    fn set(&self, _: usize, _: (usize, u64)) { }
}


pub fn collatz(n: u64, mut v: Vec<u64>) -> Vec<u64> {
    if n == 1 {
        v.push(n);
        v
    }
    else if n % 2 == 0 {
        v.push(n / 2);
        collatz(n / 2, v)
    }
    else {
        v.push(3 * n + 1);
        collatz(3 * n + 1, v)
    }
}

fn collatz_len_max_with_cache(n: u64, cache: &Arc<impl Cache>) -> (usize, u64) {
    if n == 1 {
        (1, 1)
    } else {
        let r = cache.get(n as usize);
        if r.0 > 0 {
            return r;
        }
        let next_n = if n % 2 == 0 { n / 2 } else { 3 * n + 1 };
        let r = collatz_len_max_with_cache(next_n, cache);
        let result = (r.0 + 1, std::cmp::max(n, r.1));
        cache.set(n as usize, result);
        result
    }
}

fn collatz_len_max(n: u64) -> (usize, u64) {
    if n == 1 {
        (1, 1)
    } else {
        let next_n = if n % 2 == 0 { n / 2 } else { 3 * n + 1 };
        let r = collatz_len_max(next_n);
        (r.0 + 1, std::cmp::max(n, r.1))
    }
}

pub fn collatz_len_max_parallel<T>(start: usize, end: usize, thread_num: usize, cache: T)
    where T: Cache + Sync + Send + 'static
{
    let current_num = Arc::new(AtomicUsize::new(start));
    let cache = Arc::new(cache);
    let mut handles = vec![];
    for _ in 0..thread_num {
        let current_num = Arc::clone(&current_num);
        let cache = Arc::clone(&cache);
        let handle = thread::spawn(move || -> (IndexedValue::<usize>, IndexedValue::<u64>) {
            let mut max_len = IndexedValue::<usize> { n: 0, value: 0 };
            let mut max_max = IndexedValue::<u64> { n: 0, value: 0 };
            loop {
                let n = current_num.fetch_add(1, atomic::Ordering::Relaxed);
                if n > end {
                    break
                }
                let (len, max) = if cache.len() > 0 {
                    collatz_len_max_with_cache(n as u64, &cache)
                } else {
                    collatz_len_max(n as u64)
                };
                max_len = cmp::max(max_len, IndexedValue::<usize> { n, value: len });
                max_max = cmp::max(max_max, IndexedValue::<u64> { n, value: max });
            };
            (max_len, max_max)
        });
        handles.push(handle);
    }
    let mut max_len = IndexedValue::<usize> { n: 0, value: 0 };
    let mut max_max = IndexedValue::<u64> { n: 0, value: 0 };
    for handle in handles {
        let (max_len_thread, max_max_thread) = handle.join().unwrap();
        max_len = cmp::max(max_len, max_len_thread);
        max_max = cmp::max(max_max, max_max_thread);
    }
    println!("max_len = {}, max_value = {}", max_len, max_max);
}