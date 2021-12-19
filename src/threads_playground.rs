use std::thread;
use std::sync::{Arc, Mutex, RwLock};
use std::sync::atomic;

const THREAD_NUM: usize = 3;
// fn borrow_ref() {
//     let v = vec![1, 2, 3];
//     // error[E0597]: `v` does not live long enough
//     let v_ref = &v;
//     let handle = thread::spawn(|| {
//         println!("v = {:?}", v_ref); // スレッド内に'staticより短い寿命の参照を持ち込むことはできない
//     });
//     handle.join().unwrap();
// } // `v` dropped here while still borrowed

// fn borrow() {
//     let v = vec![1, 2, 3];
//     // error[E0373]: closure may outlive the current function, but it borrows `v`, which is owned by the current function
//     let handle = thread::spawn(|| {
//         println!("v = {:?}", v); // `v` is borrowed here
//     });
//     handle.join().unwrap();
// }

fn move_once() {
    let v = vec![1, 2, 3];
    let handle = thread::spawn(move || {
        println!("v = {:?}", v); // `v` is borrowed here
    });
    handle.join().unwrap();
}

// fn move_twice() {
//     let mut handles = vec![];
//     let v = vec![1, 2, 3];
//     for _ in 0..THREAD_NUM {
//         handles.push(thread::spawn(move || {
//             // error[E0382]: use of moved value: `v`
//             println!("[{}, {}, {}]", v[0], v[1], v[2]);
//         }));
//     }
//     handles.into_iter().for_each(|h| h.join().unwrap());
// }

fn move_clone() {
    let mut handles = vec![];
    let v = vec![1, 2, 3];
    for _ in 0..THREAD_NUM {
        let v = v.clone();
        handles.push(thread::spawn(move || {
            println!("[{}, {}, {}]", v[0], v[1], v[2]);
        }));
    }
    handles.into_iter().for_each(|h| h.join().unwrap());
}

// use std::rc::Rc;
// fn use_rc() {
//     let mut handles = vec![];
//     let v = Rc::new(vec![1, 2, 3]);
//     for _ in 0..THREAD_NUM {
//         let v = Rc::clone(&v);
//         handles.push(thread::spawn(move || {
//             // error[E0277]: `Rc<Vec<i32>>` cannot be sent between threads safely
//             println!("{:?}", v);
//         }));
//     }
//     handles.into_iter().for_each(|h| h.join().unwrap());
// }

fn arc_read() {
    let mut handles = vec![];
    let v = Arc::new(vec![1, 2, 3]);
    for _ in 0..THREAD_NUM {
        let v = Arc::clone(&v);
        handles.push(thread::spawn(move || {
            println!("[{}, {}, {}]", v[0], v[1], v[2]);
        }));
    }
    handles.into_iter().for_each(|h| h.join().unwrap());
}

// fn arc_write() {
//     let mut handles = vec![];
//     let v = Arc::new(vec![1, 2, 3]);
//     for _ in 0..THREAD_NUM {
//         let v = Arc::clone(&v);
//         handles.push(thread::spawn(move || {
//             // error[E0596]: cannot borrow data in an `Arc` as mutable
//             v[0] = 10;
//             println!("{:?}", v);
//         }));
//     }
//     handles.into_iter().for_each(|h| h.join().unwrap());
// }

fn arc_mutex_write() {
    let mut handles = vec![];
    let v = Arc::new(vec![Mutex::new(1), Mutex::new(2), Mutex::new(3)]);
    for _ in 0..THREAD_NUM {
        let v = Arc::clone(&v);
        handles.push(thread::spawn(move || {
            {
                let mut n = v[0].lock().unwrap();
                *n = 10;
                println!("v[0] = {}", *n);
            } // ここでnが破棄され、取得したロックが解放される
            // ここで何か別の処理が挟まれる想定
            // ...
            println!("{}", v[0].lock().unwrap()); // ここでロックを取得して値を読み込み、ロックを解放
        }));
    }
    handles.into_iter().for_each(|h| h.join().unwrap());
}

fn arc_rwlock_write() {
    let mut handles = vec![];
    let v = Arc::new(vec![RwLock::new(1), RwLock::new(2), RwLock::new(3)]);
    for _ in 0..THREAD_NUM {
        let v = Arc::clone(&v);
        handles.push(thread::spawn(move || {
            {
                let mut n = v[0].write().unwrap();
                *n = 10;
                println!("v[0] = {}", *n);
            } // ここでnが破棄され、取得したロックが解放される
            // ここで何か別の処理が挟まれる想定
            // ...
            println!("{}", v[0].read().unwrap()); // ここでロックを取得して値を読み込み、ロックを解放
        }));
    }
    handles.into_iter().for_each(|h| h.join().unwrap());
}

// fn arc_push() {
//     let mut handles = vec![];
//     let v = Arc::new(vec![Mutex::new(1), Mutex::new(2), Mutex::new(3)]);
//     for _ in 0..THREAD_NUM {
//         let mut v = Arc::clone(&v);
//         handles.push(thread::spawn(move || {
//             // error[E0596]: cannot borrow data in an `Arc` as mutable
//             v.push(Mutex::new(4));
//         }));
//     }
//     handles.into_iter().for_each(|h| h.join().unwrap());
// }

fn arc_mutex_push() {
    let mut handles = vec![];
    let v = Arc::new(Mutex::new(vec![1, 2, 3]));
    for _ in 0..THREAD_NUM {
        let v = Arc::clone(&v);
        handles.push(thread::spawn(move || {
            let mut v = v.lock().unwrap();
            v[0] = 10; // 書き込み可能
            v.push(4); // v自体の操作が可能
            let elem = v.pop().unwrap();
            println!("popped = {}", elem);
        }));
    }
    handles.into_iter().for_each(|h| h.join().unwrap());
}

fn atomic() {
    let mut handles = vec![];
    let current_num = Arc::new(atomic::AtomicUsize::new(0));
    for _ in 0..THREAD_NUM {
        let current_num = Arc::clone(&current_num);
        handles.push(thread::spawn(move || {
            loop {
                let n = current_num.fetch_add(1, atomic::Ordering::Relaxed);
                if n >= 10 {
                    break;
                }
                println!("got {}", n);
            }
        }));
    }
    handles.into_iter().for_each(|h| h.join().unwrap());
}

pub fn threads_playground() {
    move_once();
    move_clone();
    arc_read();
    arc_mutex_write();
    arc_rwlock_write();
    arc_mutex_push();
    atomic();
}