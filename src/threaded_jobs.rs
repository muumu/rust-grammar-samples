use std::thread;
use std::sync::atomic;
use std::sync::{Arc, Mutex};
use threadpool::ThreadPool;
use std::sync::mpsc;

fn spawn_thread_i32_sum() {
    let mut handles: Vec<std::thread::JoinHandle<i32>> = vec![];
    let mut values_vec = vec![vec![], vec![], vec![], vec![]];
    for (i, values) in values_vec.iter_mut().enumerate() {
        for value in (i * 25)..((i +1) * 25) {
            values.push(value as i32);
        }
    }
    for values in values_vec {
        handles.push(std::thread::spawn(move || {
            values.iter().sum()
        }));
    }
    let mut sum = 0;
    for handle in handles {
        sum += handle.join().unwrap();
    }
    println!("sum = {}", sum);
}

pub struct Data { n: i32 }

impl Data {
    fn incr(&mut self) { self.n += 1; }
}

fn spawn_thread() {
    let mut handles = vec![];
    let v = vec![Data { n: 0 }, Data { n: 1 }, Data { n: 2 }];
    for mut data in v {
        handles.push(thread::spawn(move || {
            data.incr();
            data
        }));
    }
    let mut sum = 0;
    for handle in handles {
        let data = handle.join().unwrap();
        sum += data.n;
    }
    println!("sum = {}", sum); // 6
}

fn use_threadpool() {
    let n_workers = 4;
    let pool = ThreadPool::new(n_workers);
    let (tx, rx) = mpsc::channel();
    let v = vec![Data { n: 0 }, Data { n: 1 }, Data { n: 2 }];
    let n_jobs = v.len();
    for mut data in v {
        let tx = tx.clone();
        pool.execute(move || {
            data.incr();
            tx.send(data).expect("channel will be there waiting for the pool");
        });
    }
    let sum: i32 = rx.iter().take(n_jobs).map(|data| data.n).sum();
    println!("sum = {}", sum); // 6
}

fn use_atomic() {
    let n_workers = 4;
    let n_jobs = 10000;
    let pool = ThreadPool::new(n_workers);
    let mut some_big_vec = vec![];
    for i in 0..n_jobs {
        some_big_vec.push(Data { n: i as i32 });
    }
    let some_big_vec = Arc::new(some_big_vec);
    let current_num = Arc::new(atomic::AtomicUsize::new(0));
    for _ in 0..n_workers {
        let some_big_vec = Arc::clone(&some_big_vec);
        let current_num = Arc::clone(&current_num);
        pool.execute(move || {
            loop {
                let n = current_num.fetch_add(1, atomic::Ordering::Relaxed);
                if n >= n_jobs {
                    break;
                }
                println!("got {}", n);
            }
        }));
    }
    handles.into_iter().for_each(|h| h.join().unwrap());
}

pub fn threaded_jobs() {
    spawn_thread_i32_sum();
    spawn_thread();
    use_threadpool();
}