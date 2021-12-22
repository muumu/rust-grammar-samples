use std::time::Instant;
use std::sync::{Mutex, Arc, RwLock};
use std::cell::RefCell;
use std::marker::{Sync, Send};

/*
解きたい問題
毎回ランダムに配られる3色のカラーボールを2つの筒に1個ずつ入れていきます。
1つの筒には最大6個のカラーボールを1列に積むことができます。
12個のカラーボールを2つの筒に入れ終わった時点で、いずれかまたは両方の筒において
同色のカラーボールが4つ以上接しているようにカラーボールを積める確率を求めてください。
なお、手元のカラーボールを積むまで次にどの色が来るかは見ることができないものとし、
カラーボールをどちらの筒に入れるかは求める確率が最大となるように選択する（その時点での最善手を選択する）ものとします。
 */

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

// width個の筒にカラーボールをheight個積むことができる筒のセットの状態を表現する構造体
#[derive(Clone, Debug)]
pub struct Board {
    board: Vec<Vec<Option<Color>>>
}

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

    // Boardの状態のビット表現。筒の並びは問わないため筒のビット表現の値順にソートする
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

// シングルスレッド用のキャッシュ構造体
// MutexCache / RwLockCacheとインターフェースを共通化するためRefCellで包んでいる
pub struct RefCellCache {
    cache: Vec<RefCell<Option<f64>>>,
}

// Mutexを使用したスレッドセーフなキャッシュ構造体
pub struct MutexCache {
    cache: Vec<Mutex<Option<f64>>>,
}

// RwLockを使用したスレッドセーフなキャッシュ構造体
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

// 条件を満たす確率を求める関数
// Cacheを使用する場合と使用しない場合とで共通の実装になっているが
// Cacheトレイト実装型として引数に与えられる型は静的ディスパッチで決定されるため
// NoCacheを与えた場合はコンパイラの最適化によりキャッシュ処理のコードは削除される
pub fn probability(n: usize, board: Board, connection_size: usize, cache: &impl Cache) -> f64 {
    // 最後まで積み終わった状態で、設定された連結数以上に連結があれば条件を満たす
    // 条件を満たしている場合は確率1、満たしていない場合は確率0を返す
    if n == 0 {
        if board.is_connected(connection_size) {
            1.0
        } else {
            0.0
        }
    } else {
        // キャッシュがある場合はキャッシュの値を返す（NoCacheの場合は常に存在しない）
        if let Some(c) = cache.get(&board) {
            return c;
        }
        let mut sum = 0.0;
        // ランダムに来るn色の場合の確率をそれぞれ計算してsumに足していく
        for color in Color::all() {
            let mut max = 0.0;
            // どの筒に入れるかは、入れた場合にもっとも確率が高くなる方に入れるという判断をする
            // もっとも確率が高くなる方に入れた場合の確率がmax変数に入る
            for x in 0..board.width() {
                if board.top(x) >= board.height() {
                    continue;
                }
                let mut board = board.clone();
                board.drop(x, color);
                let p = probability(n - 1, board, connection_size, cache);
                if p > max {
                    max = p;
                }
            }
            sum += max;
        }
        // sumを色数で割って得られる確率の平均値が求める確率
        let p = sum / Color::len() as f64;
        // 得られた確率はキャッシュにも格納しておく（NoCacheの場合は何もしない）
        cache.set(&board, p);
        p
    }
}

// threaded_n回目の呼び出しまでスレッドを立ち上げて並列計算を実施
pub fn probability_parallel<T>(n: usize, threaded_n: usize, board: Board,
                           connection_size: usize, cache: &Arc<T>) -> f64
    where T: Cache + Sync + Send + 'static
{
    if n <= 1 || threaded_n == 0 {
        // 以降の計算は各スレッドにおいて直列処理を呼び出して処理を続行
        probability(n, board, connection_size, cache.as_ref())
    } else {
        let mut handles_map = vec![];
        for color in Color::all() {
            handles_map.push(vec![]);
            for x in 0..board.width() {
                if board.top(x) >= board.height() {
                    continue;
                }
                let mut board = board.clone();
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

pub fn print_elapsed_times(elapsed_nanos: &Vec<u128>, label: &str) {
    let unit = 1_000_000_000.0;
    println!("{} (min): {:.4} [s]", label, *elapsed_nanos.iter().min().unwrap() as f64 / unit);
    println!("{} (max): {:.4} [s]", label, *elapsed_nanos.iter().max().unwrap() as f64 / unit);
    println!("{} (mean): {:.4} [s]", label,
             elapsed_nanos.iter().sum::<u128>() as f64 / (unit * elapsed_nanos.len() as f64));
}

pub fn calc_probabilities() {
    let width = 2; // 筒の個数
    let height = 6; // 1本の筒に積めるカラーボール最大数
    let n = 12; // カラーボールを積む総数
    let connection_size = 4; // 条件を満たすのに必要な同色のカラーボールの連結数
    let threaded_n = 2; // スレッドを立ち上げる再帰の深さ（2なら色数3、筒数2のとき6 + 6 * 6 = 42スレッド）
    let cache_size = 2usize.pow((width * height * Color::bits()) as u32);
    let board = Board::with_size(width, height);
    println!("cache_size = {}", cache_size);
    // 処理時間にばらつきが生じるためそれぞれ3回計測
    let repeat_num = 3;
    // 直列処理キャッシュ無し
    let mut elapsed_nanos = vec![];
    for _ in 0..repeat_num {
        let start = Instant::now();
        let p = probability(n, board.clone(), connection_size, &NoCache::with_len(0));
        let end = start.elapsed();
        elapsed_nanos.push(end.as_nanos());
        println!("p = {} (elapsed: {:.4})", p, end.as_nanos() as f64 / 1_000_000_000.0);
    }
    print_elapsed_times(&elapsed_nanos, "Serial without cache");
    // 並列処理キャッシュ無し
    let mut elapsed_nanos = vec![];
    for _ in 0..repeat_num {
        let cache = Arc::new(NoCache::with_len(0));
        let start = Instant::now();
        let p = probability_parallel(n, threaded_n, board.clone(), connection_size, &cache);
        let end = start.elapsed();
        elapsed_nanos.push(end.as_nanos());
        println!("p = {} (elapsed: {:.4})", p, end.as_nanos() as f64 / 1_000_000_000.0);
    }
    print_elapsed_times(&elapsed_nanos, "Parallel without cache");
    // 直列処理RefCellキャッシュ使用
    let mut elapsed_nanos = vec![];
    for _ in 0..repeat_num {
        let cache = RefCellCache::with_len(cache_size);
        let start = Instant::now();
        let p = probability(n, board.clone(), connection_size, &cache);
        let end = start.elapsed();
        elapsed_nanos.push(end.as_nanos());
        println!("p = {} (elapsed: {:.4})", p, end.as_nanos() as f64 / 1_000_000_000.0);
    }
    print_elapsed_times(&elapsed_nanos, "Serial with RefCellCache");
    // 並列処理Mutexキャッシュ使用
    let mut elapsed_nanos = vec![];
    for _ in 0..repeat_num {
        let cache = Arc::new(MutexCache::with_len(cache_size));
        let start = Instant::now();
        let p = probability_parallel(n, threaded_n, board.clone(), connection_size, &cache);
        let end = start.elapsed();
        elapsed_nanos.push(end.as_nanos());
        println!("p = {} (elapsed: {:.4})", p, end.as_nanos() as f64 / 1_000_000_000.0);
    }
    print_elapsed_times(&elapsed_nanos, "Parallel with MutexCache");
    // 並列処理RwLockキャッシュ使用
    let mut elapsed_nanos = vec![];
    for _ in 0..repeat_num {
        let cache = Arc::new(RwLockCache::with_len(cache_size));
        let start = Instant::now();
        let p = probability_parallel(n, threaded_n, board.clone(), connection_size, &cache);
        let end = start.elapsed();
        elapsed_nanos.push(end.as_nanos());
        println!("p = {} (elapsed: {:.4})", p, end.as_nanos() as f64 / 1_000_000_000.0);
    }
    print_elapsed_times(&elapsed_nanos, "Parallel with RwLockCache");
}