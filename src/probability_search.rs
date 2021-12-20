use std::time::Instant;
use std::sync::{Mutex, Arc, RwLock};
// use std::sync::atomic;
// use std::sync::atomic::AtomicUsize;
use std::cell::RefCell;
use std::marker::{Sync, Send};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Color {
    Red,
    Green,
    Blue,
}

impl Color {
    fn all() -> Vec<Self> {
        vec![Self::Red, Self::Green, Self::Blue]
    }
    fn len() -> usize {
        Self::all().len()
    }
    // Color全種類とNoneを表現できるビット数（1色なら1ビット、2～3色なら2ビット、4～7色なら3ビット）
    fn bits() -> usize {
        let mut n = Self::len();
        let mut count = 1;
        while n > 1 {
            count += 1;
            n >>= 1;
        }
        count
    }
    // 色のビット表現（Noneを0としたいのでColor::Redを1とする）
    fn to_bit(&self) -> usize {
        *self as usize + 1
    }
}

#[derive(Clone, Debug)]
pub struct Board {
    board: Vec<Vec<Option<Color>>>
}

// width個の筒にカラーボールをheight個積むことができる筒のセットの状態を表現するもの
impl Board {
    pub fn with_size(width: usize, height: usize) -> Self {
        let mut board = vec![];
        for _ in 0..width {
            let mut line = vec![];
            for _ in 0..height {
                line.push(None);
            }
            board.push(line);
        }
        Self { board }
    }
    pub fn width(&self) -> usize {
        self.board.len()
    }
    pub fn height(&self) -> usize {
        self.board[0].len()
    }
    // カラーボールを積めるな最も高い位置のインデックスを返す
    pub fn top(&self, x: usize) -> usize {
        let mut y = 0;
        while y < self.height() && self.board[x][y] != None {
            y += 1;
        }
        y
    }
    // カラーボールを落として設置する
    pub fn drop(&mut self, x: usize, color: Color) {
        let y = self.top(x);
        debug_assert!(y >= self.height());
        self.board[x][y] = Some(color);
    }
    // pub fn serialize(&self) -> usize {
    //     let mut data = 0;
    //     for v in &self.board {
    //         for color in v {
    //             data <<= Color::bits();
    //             data += *color as usize;
    //         }
    //     }
    //     if data >= CACHE_SIZE {
    //         panic!();
    //     }
    //     data
    // }

    // Boardの状態のビット表現。筒の並びは問わないため要素数順にソートする
    pub fn serialize(&self) -> usize {
        let mut v = vec![];
        for line in &self.board {
            let mut data = 0;
            for color in line {
                data <<= Color::bits();
                if let Some(c) = color {
                    data += c.to_bit();
                }
            }
            v.push(data);
        }
        v.sort();
        let mut data = 0;
        for line in v {
            data <<= Color::bits() * self.height();
            data += line;
        }
        data
    }
    // いずれかの筒において同色のカラーボールがconnection_size個以上連続して積まれていたらtrueを返す
    pub fn is_connected(&self, connection_size: usize) -> bool {
        for line in &self.board {
            let mut count = 0;
            let mut current_color = Color::Red;
            for color in line {
                if let Some(c) = color {
                    if *c == current_color {
                        count += 1;
                        if count >= connection_size {
                            return true;
                        }
                    } else {
                        current_color = *c;
                        count = 1;
                    }
                } else {
                    break;
                }
            }
        }
        false
    }
}

// 確率計算のメモ化に使用するキャッシュのインターフェース
pub trait Cache {
    fn with_len(len: usize) -> Self;
    fn len(&self) -> usize;
    fn get(&self, board: &Board) -> Option<f64>;
    fn set(&self, board: &Board, data: f64);
}

pub struct RefCellCache {
    cache: Vec<RefCell<Option<f64>>>,
}

pub struct MutexCache {
    cache: Vec<Mutex<Option<f64>>>,
}

pub struct RwLockCache {
    cache: Vec<RwLock<Option<f64>>>,
}

// キャッシュを使用しないことを示す空の構造体
pub struct NoCache { }

impl Cache for RefCellCache {
    fn with_len(len: usize) -> Self {
        let mut cache = Vec::<RefCell<Option<f64>>>::with_capacity(len);
        for _ in 0..len {
            cache.push(RefCell::new(None));
        }
        Self { cache }
    }
    fn len(&self) -> usize { self.cache.len() }
    fn get(&self, board: &Board) -> Option<f64> {
        let i = board.serialize();
        if i < self.cache.len() {
            *self.cache[i].borrow()
        } else {
            None
        }
    }
    fn set(&self, board: &Board, data: f64) {
        let i = board.serialize();
        if i < self.cache.len() {
            self.cache[i].replace(Some(data));
        }
    }
}

impl Cache for RwLockCache {
    fn with_len(len: usize) -> Self {
        let mut cache = Vec::<RwLock<Option<f64>>>::with_capacity(len);
        for _ in 0..len {
            cache.push(RwLock::new(None));
        }
        Self { cache }
    }
    fn len(&self) -> usize { self.cache.len() }
    fn get(&self, board: &Board) -> Option<f64> {
        let i = board.serialize();
        if i < self.cache.len() {
            *self.cache[i].read().unwrap()
        } else {
            None
        }
    }
    fn set(&self, board: &Board, data: f64) {
        let i = board.serialize();
        if i < self.cache.len() {
            *self.cache[i].write().unwrap() = Some(data);
        }
    }
}

impl Cache for MutexCache {
    fn with_len(len: usize) -> Self {
        let mut cache = Vec::<Mutex<Option<f64>>>::with_capacity(len);
        for _ in 0..len {
            cache.push(Mutex::new(None));
        }
        Self { cache }
    }
    fn len(&self) -> usize { self.cache.len() }
    fn get(&self, board: &Board) -> Option<f64> {
        let i = board.serialize();
        if i < self.cache.len() {
            *self.cache[i].lock().unwrap()
        } else {
            None
        }
    }
    fn set(&self, board: &Board, data: f64) {
        let i = board.serialize();
        if i < self.cache.len() {
            *self.cache[i].lock().unwrap() = Some(data);
        }
    }
}

impl Cache for NoCache {
    fn with_len(_: usize) -> Self { Self { } }
    fn len(&self) -> usize { 0 }
    fn get(&self, _: &Board) -> Option<f64> { None }
    fn set(&self, _: &Board, _: f64) { }
}

fn probability(n: usize, board: Board, connection_size: usize, cache: &impl Cache) -> f64 {
    if n == 0 {
        if board.is_connected(connection_size) {
            1.0
        } else {
            0.0
        }
    } else {
        if let Some(c) = cache.get(&board) {
            return c;
        }
        let mut sum = 0.0;
        for color in Color::all() {
            let mut max = 0.0;
            for x in 0..board.width() {
                let mut board = board.clone();
                if board.top(x) >= board.height() {
                    continue;
                }
                board.drop(x, color);
                let p = probability(n - 1, board, connection_size, cache);
                if p > max {
                    max = p;
                }
            }
            sum += max;
        }
        let p = sum / Color::len() as f64;
        cache.set(&board, p);
        p
    }
}

fn probability_parallel<T>(n: usize, threaded_n: usize, board: Board,
                           connection_size: usize, cache: &Arc<T>) -> f64
    where T: Cache + Sync + Send + 'static
{
    if n <= 1 || threaded_n == 0 {
        probability(n, board, connection_size, cache.as_ref())
    } else {
        let mut handles_map = vec![];
        for color in Color::all() {
            handles_map.push(vec![]);
            for x in 0..board.width() {
                let mut board = board.clone();
                if board.top(x) >= board.height() {
                    continue;
                }
                let cache = Arc::clone(&cache);
                let handle = std::thread::spawn(move || {
                    //println!("thread spawned: n = {}, x = {}, color = {}", n, x, color as usize);
                    board.drop(x, color);
                    probability_parallel(n - 1, threaded_n - 1, board, connection_size, &cache)
                });
                handles_map[color as usize].push(handle);
            }
        }
        let mut sum = 0.0;
        for handles in handles_map {
            let mut max = 0.0;
            for handle in handles {
                let p = handle.join().unwrap();
                if p > max {
                    max = p;
                }
            }
            sum += max;
        }
        sum / Color::len() as f64
    }
}


pub fn calc_probabilities() {
    let width = 2;
    let height = 6;
    let n = 12;
    let connection_size = 4;
    let threaded_n = 2;
    let cache_size = 2usize.pow((width * height * Color::bits()) as u32);
    println!("cache_size = {}", cache_size);
    // {
    //     let board = Board::with_size(width, height);
    //     let cache = Arc::new(NoCache::with_len(0));
    //     let start = Instant::now();
    //     let p = search_parallel(n, threaded_n, board, connection_size, &cache);
    //     let end = start.elapsed();
    //     println!("p = {}", p);
    //     println!("{}.{:03} [s]", end.as_secs(), end.subsec_nanos() / 1_000_000);
    // }
    // {
    //     let board = Board::with_size(width, height);
    //     let cache = Arc::new(NoCache::with_len(0));
    //     let start = Instant::now();
    //     let p = search_parallel(n, threaded_n, board, connection_size, &cache);
    //     let end = start.elapsed();
    //     println!("p = {}", p);
    //     println!("{}.{:03} [s]", end.as_secs(), end.subsec_nanos() / 1_000_000);
    // }
    // {
    //     let board = Board::with_size(width, height);
    //     let start = Instant::now();
    //     let p = search(n, board, connection_size, &NoCache::with_len(0));
    //     let end = start.elapsed();
    //     println!("p = {}", p);
    //     println!("{}.{:03} [s]", end.as_secs(), end.subsec_nanos() / 1_000_000);
    // }
    for _ in 0..3 {
        {
            let board = Board::with_size(width, height);
            let cache = RefCellCache::with_len(cache_size);
            //let cache = MutexCache::with_len(CACHE_SIZE);
            let start = Instant::now();
            let p = probability(n, board, connection_size, &cache);
            let end = start.elapsed();
            println!("p = {}", p);
            println!("{}.{:03} [s]", end.as_secs(), end.subsec_nanos() / 1_000_000);
        }
        {
            let board = Board::with_size(width, height);
            let cache = Arc::new(MutexCache::with_len(cache_size));
            //let cache = Arc::new(RefCellCache::with_len(CACHE_SIZE));
            let start = Instant::now();
            let p = probability_parallel(n, threaded_n, board, connection_size, &cache);
            let end = start.elapsed();
            println!("p = {}", p);
            println!("{}.{:03} [s]", end.as_secs(), end.subsec_nanos() / 1_000_000);
        }
        {
            let board = Board::with_size(width, height);
            let cache = Arc::new(RwLockCache::with_len(cache_size));
            let start = Instant::now();
            let p = probability_parallel(n, threaded_n, board, connection_size, &cache);
            let end = start.elapsed();
            println!("p = {}", p);
            println!("{}.{:03} [s]", end.as_secs(), end.subsec_nanos() / 1_000_000);
        }
    }

}