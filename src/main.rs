extern crate num;
#[macro_use]
extern crate num_derive;

mod cppenum;
mod threadsafemap;
mod collatz;
mod threads_playground;
mod indexed_value;

use std::time::Instant;
use crate::collatz::{Cache, RwLockCache, MutexCache, NoCache};

fn main() {
    cppenum::use_color_type();
    cppenum::use_color_struct();
    cppenum::use_color_enum();
    threadsafemap::threadsafe_map();
    let add = |x, y| x + y;
    println!("{}", add(2, 3));
    let mut n = 1;
    let mut f = || { n += 1; n };
    println!("{}", f()); // 2
    println!("{}", f()); // 3
    println!("{}", n); // 3
    let n = 100;
    let thread_num = 36;
    collatz::collatz_len_max_parallel(1, n, thread_num, NoCache::with_len(0));
    collatz::collatz_len_max_parallel(1, n, thread_num, MutexCache::with_len(10 * n));
    collatz::collatz_len_max_parallel(1, n, thread_num, RwLockCache::with_len(10 * n));
    //let n = 100_000_000;
    {
        let start = Instant::now();
        collatz::collatz_len_max_parallel(1, n, thread_num, NoCache::with_len(0));
        let end = start.elapsed();
        println!("{}.{:03} [s]", end.as_secs(), end.subsec_nanos() / 1_000_000);
    }
    {
        let start = Instant::now();
        collatz::collatz_len_max_parallel(1, n, thread_num, MutexCache::with_len(n));
        let end = start.elapsed();
        println!("{}.{:03} [s]", end.as_secs(), end.subsec_nanos() / 1_000_000);
    }
    {
        let start = Instant::now();
        collatz::collatz_len_max_parallel(1, n, thread_num, RwLockCache::with_len(n));
        let end = start.elapsed();
        println!("{}.{:03} [s]", end.as_secs(), end.subsec_nanos() / 1_000_000);
    }
    let v = collatz::collatz(80049391, Vec::<u64>::new());
    println!("len = {}, max_value = {}", v.len(), v.iter().max().unwrap());
    threads_playground::threads_playground();
}