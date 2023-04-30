use std::{arch::x86_64::*, mem::align_of_val};

#[repr(align(16))]
struct AlignedArr16<T: Default + Copy, const N: usize>([T; N]);

impl<T: Default + Copy, const N: usize> AlignedArr16<T, N> {
    fn new() -> Self {
        AlignedArr16::<T, N>([T::default(); N])
    }
}

// #[repr(align(16))]
// struct AlignedArr16u8([u8; 16]);

#[repr(align(16))]
struct Alignedu128(u128);


fn print_bytes(bits: &[u8; 16]) {
    for (i, b) in bits.iter().rev().enumerate() {
        print!("{:08b} ", b);
        if i % 8 == 7 {
            println!();
        }
    }
}

fn print_hex(bits: &[u16; 8]) {
    for b in bits.iter().rev() {
        print!("{:04x} ", b);
    }
    println!();
}

fn print_m128i(m: &__m128i, unit: usize) {
    unsafe {
        if unit == 128 {
            let mut bit = Alignedu128(0);
            _mm_storeu_si128(&mut bit as *mut Alignedu128 as *mut __m128i, *m);
            println!("{}", bit.0);
        } else if unit == 64 {
            let mut bits = AlignedArr16::<i64, 2>::new();
            _mm_storeu_si128(&mut bits as *mut AlignedArr16<i64, 2> as *mut __m128i, *m);
            println!("({}, {})", bits.0[1], bits.0[0]);
        } else if unit == 32 {
            let mut bits = AlignedArr16::<i32, 4>::new();
            _mm_storeu_si128(&mut bits as *mut AlignedArr16<i32, 4> as *mut __m128i, *m);
            println!("({}, {}, {}, {})", bits.0[3], bits.0[2], bits.0[1], bits.0[0]);
        } else if unit == 16 {
            let mut bits = AlignedArr16::<i16, 8>::new();
            _mm_storeu_si128(&mut bits as *mut AlignedArr16<i16, 8> as *mut __m128i, *m);
            let bits_str = bits.0.iter().rev().map(|x| x.to_string()).collect::<Vec<_>>().join(", ");
            println!("({})", bits_str);
        } else if unit == 8 {
            let mut bits = AlignedArr16::<i8, 16>::new();
            _mm_storeu_si128(&mut bits as *mut AlignedArr16<i8, 16> as *mut __m128i, *m);
            let bits_str = bits.0.iter().rev().map(|x| x.to_string()).collect::<Vec<_>>().join(", ");
            println!("({})", bits_str);
        } else if unit == 4 {
            let mut bits = AlignedArr16::<u16, 8>::new();
            _mm_storeu_si128(&mut bits as *mut AlignedArr16<u16, 8> as *mut __m128i, *m);
            print_hex(&bits.0);
        } else {
            let mut bits = AlignedArr16::<u8, 16>::new();
            _mm_storeu_si128(&mut bits as *mut AlignedArr16<u8, 16> as *mut __m128i, *m);
            print_bytes(&bits.0);
        }
    }
}

fn print_alignment() {
    let arr = [0u8; 16];
    println!("align_of_val(&arr) = {}", align_of_val(&arr));
    println!("Addresses of arr: ");
    for a in &arr {
        print!("{:p} ", a);
    }
    println!();
    let num: i128 = 0;
    println!("align_of_val(&num) = {}", align_of_val(&num));
    println!("Address of num: ");
    println!("{:p}", &num);
    let buf = AlignedArr16::<u8, 16>::new();
    println!("align_of_val(&buf) = {}", align_of_val(&buf));
    println!("Addresses of buf: ");
    for a in &buf.0 {
        print!("{:p} ", a);
    }
    println!();
}

fn load_store() {
    println!("-----load_store-----");
    // !! 16バイトアラインメントされていない低速な処理の記述 !!
    unsafe {
        // u16整数を8つSIMD演算したい場合は以下の配列に下位ビットから順にセット
        let a: [u16; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
        // 一度生ポインタに変換してから__m128iの生ポインタにキャストしてxmmレジスタにロード
        let m = _mm_loadu_si128(&a as *const [u16; 8] as *const __m128i);
        // 以下の変数にSIMDから結果（今回は何もSIMDで処理していないが……）をストアする
        let mut bits = [0u16; 8];
        // loadとは逆に、xmmレジスタから__m128i変数にstore
        _mm_storeu_si128(&mut bits as *mut [u16; 8] as *mut __m128i, m);
        // 変数のビット列を16進数で出力
        print_hex(&bits); // 0008 0007 0006 0005 0004 0003 0002 0001
        // i128型変数に格納することもできるがメモリ配置によってはパフォーマンス悪化のリスクあり
        let mut n: i128 = 0;
        _mm_storeu_si128(&mut n as *mut i128 as *mut __m128i, m);
        println!("{:032x}", n); // 00080007000600050004000300020001
    }
    // 16バイトアラインメントされた状態でのロードとストア
    unsafe {
        // 16バイトアラインメントの配列に下位ビットから順にi16整数をセット
        let a = AlignedArr16::<i16, 8>([1, 2, 3, 4, 5, 6, 7, 8]);
        // 一度生ポインタに変換してから__m128iの生ポインタにキャストしてxmmレジスタにロード
        let m = _mm_loadu_si128(&a as *const AlignedArr16<i16, 8> as *const __m128i);
        let mut bits = AlignedArr16::<u16, 8>::new();
        // loadとは逆に、xmmレジスタから__m128i変数にstore
        _mm_storeu_si128(&mut bits as *mut AlignedArr16<u16, 8> as *mut __m128i, m);
        // 変数のビット列を16進数で出力
        print_hex(&bits.0); // 0008 0007 0006 0005 0004 0003 0002 0001
    }
    unsafe {
        // 64ビット整数2つを128ビットにセット（こちらも下位桁から順に）
        let a = AlignedArr16::<u64, 2>([0xffff, 0xff]);
        let m = _mm_loadu_si128(&a as *const AlignedArr16<u64, 2> as *const __m128i);
        print_m128i(&m, 4);
    }
    unsafe {
        // 配列でないu128変数からもload可能
        let a = Alignedu128(0x000f_00ff_0fff_ffff_fff0_ff00_f000_0000);
        let m = _mm_loadu_si128(&a as *const Alignedu128 as *const __m128i);
        print_m128i(&m, 4);
    }
    unsafe {
        // setもextractも複数のCPU命令が発行されるため一般には遅いとされるが
        // コンパイラの最適化によって高速化されるケースも多いため使用価値は十分にあり（特に即値を与える場合）
        // epi64の末尾にxをつけるかつけないかはただの別名で、使用しているコンパイラで存在する方を使えばOK
        let a = _mm_set_epi64x(3, 1); // 上位64bitに3、下位64bitに1をセット
        let b = _mm_set1_epi64x(3); // 上位64bit、下位64bitそれぞれに3をセット
        // 他に32bit、16bit、8bit用も存在
        let c = _mm_set_epi16(1, 2, 3, 4, 5, 6, 7, 8);
        print_m128i(&a, 4); // 0000 0000 0000 0003 0000 0000 0000 0001
        print_m128i(&b, 4); // 0000 0000 0000 0003 0000 0000 0000 0003
        print_m128i(&c, 4); // 0001 0002 0003 0004 0005 0006 0007 0008
        // 以下のextract系関数では、64bit単位や16bit単位で値を取り出せる
        // 取り出す場所は即値（最下位を0とする）で指定
        let a1 = _mm_extract_epi64::<1>(a);
        let a0 = _mm_extract_epi64::<0>(a);
        println!("({a1}, {a0})"); // (3, 1)
        let c2 = _mm_extract_epi16::<2>(c);
        let c1 = _mm_extract_epi16::<1>(c);
        let c0 = _mm_extract_epi16::<0>(c);
        println!("({c2}, {c1}, {c0})"); // (6, 7, 8)
    }
    unsafe {
        // m128i型の下位32bitに値をセット（ここでは-1を与えているので下位32bitが全て1に）
        let m = _mm_cvtsi32_si128(-1);
        print_m128i(&m, 4); // 0000 0000 0000 0000 0000 0000 ffff ffff
        // 下位32ビットのみを抽出。_mm_soteruよりも高速
        let a = _mm_cvtsi128_si32(m);
        println!("{a}"); // -1
        // 上記の64bit版もあり
        let m64 = _mm_cvtsi64_si128(-1);
        print_m128i(&m64, 4); // 0000 0000 0000 0000 ffff ffff ffff ffff
        let a64 = _mm_cvtsi128_si64(m64);
        println!("{a64}"); // -1
    }
    unsafe {
        // 全てのビットが0の値を生成する関数は専用のものがある
        let zeros = _mm_setzero_si128();
        // 全てのビットが1の値を生成する専用関数は無いが、
        // 既にSIMDレジスタに存在する値がある場合はcmpeq関数で全てのビットが1の値を生成可能
        let ones = _mm_cmpeq_epi64(zeros, zeros);
        print_m128i(&zeros, 4); // 0000 0000 0000 0000 0000 0000 0000 0000
        print_m128i(&ones, 4); // ffff ffff ffff ffff ffff ffff ffff ffff
        // 速度の観点ではコンパイラの最適化に頼ることになるが、_mm_set1_epi64関数でも生成可能
        let ones = _mm_set1_epi64x(-1);
        print_m128i(&ones, 4); // ffff ffff ffff ffff ffff ffff ffff ffff
    }
}

fn shift() {
    println!("-----shift-----");
    unsafe {
        let m = _mm_set_epi16(127, 63, 31, 15, 7, 3, 1, 0);
        {
            // 16ビット毎に1ビット右シフト（srliでのシフト数は定数で指定する必要あり）
            let a = _mm_srli_epi16::<1>(m);
            // 32ビット毎に1ビット右シフト(16ビット中の最下位ビットが次の16ビットの最上位に入り込んで負数が生じる)
            let b = _mm_srli_epi32::<1>(m);
            print_m128i(&m, 16); // (127, 63, 31, 15, 7, 3, 1, 0)
            print_m128i(&a, 16); // (63, 31, 15, 7, 3, 1, 0, 0)
            print_m128i(&b, 16); // (63, -32737, 15, -32761, 3, -32767, 0, -32768)
        }
        // シフト数を指定する変数に2を格納
        let two = _mm_cvtsi32_si128(2);
        {
            // srlではシフト数を変数で指定できる（16bit毎に2bitだけ右シフト）
            let a = _mm_srl_epi16(m, two);
            print_m128i(&a, 16); // (31, 15, 7, 3, 1, 0, 0, 0)
            // 16ビット毎に2bitだけ左シフト
            let b = _mm_sll_epi16(m, two);
            print_m128i(&b, 16); // (508, 252, 124, 60, 28, 12, 4, 0)
        }
        {
            let a = _mm_set1_epi16(-1);
            // 右シフトには符号を維持する（シフトで生じる新たな上位ビットを1で埋める）算術シフトも存在
            let b = _mm_sra_epi16(a, two);
            print_m128i(&b, 16); // (-1, -1, -1, -1, -1, -1, -1, -1)
            let c = _mm_srl_epi16(a, two);
            print_m128i(&c, 16); // (16383, 16383, 16383, 16383, 16383, 16383, 16383, 16383)
            // もちろん正の数であれば算術シフトも論理シフトも変わらない
            let d = _mm_sra_epi16(m, two);
            print_m128i(&d, 16); // (31, 15, 7, 3, 1, 0, 0, 0)
        }
        {
            // バイト単位かつ定数指定であれば__m128iのビット列全体をシフトさせることも可能
            let a = _mm_srli_si128::<2>(m);
            let b = _mm_slli_si128::<2>(m);
            print_m128i(&a, 16); // (0, 127, 63, 31, 15, 7, 3, 1)
            print_m128i(&b, 16); // (63, 31, 15, 7, 3, 1, 0, 0)
        }
    }
}

fn logical() {
    println!("-----logical-----");
    unsafe {
        let a = _mm_set_epi32(0, 0, 1, 1);
        let b = _mm_set_epi32(1, 0, 1, 0);
        // ビット単位のAND
        let and = _mm_and_si128(a, b);
        print_m128i(&and, 32); // (0, 0, 1, 0)
        // ビット単位のOR
        let or = _mm_or_si128(a, b);
        print_m128i(&or, 32); // (1, 0, 1, 1)
        // ビット単位のXOR
        let xor = _mm_xor_si128(a, b);
        print_m128i(&xor, 32); // (1, 0, 0, 1)
        // andnotではaに立っているビットをbから消す操作が可能（!a and b）
        let andnot = _mm_andnot_si128(a, b);
        print_m128i(&andnot, 32); // (1, 0, 0, 0)
        // aの否定（ビット反転）: i64などの整数値における!a相当
        // 全てのビットが1の値とのxorを取るとビット反転される
        let ones = _mm_set1_epi64x(-1);
        let not = _mm_xor_si128(a, ones);
        // ただし以下の方がコンパイラの最適化に左右されない高速な書き方
        // let not = _mm_xor_si128(a, _mm_cmpeq_epi32(a, a));
        print_m128i(&not, 32); // (-1, -1, -2, -2)
        // 全てのビットが1なら1、1つでも0が含まれていれば0を返す
        println!("{}", _mm_test_all_ones(ones)); // 1
        println!("{}", _mm_test_all_ones(a)); // 0
        let zeros = _mm_setzero_si128();
        // maskとのANDで全てのビットが0になるなら1を返し、そうでないなら0を返す
        // maskを使わずに全てのビットが0かを判定するにはmaskに自分自身を指定すればよい
        println!("{}", _mm_test_all_zeros(zeros, zeros)); // 1
        println!("{}", _mm_test_all_zeros(a, a)); // 0
    }
}

fn arithmetic() {
    println!("-----arithmetic-----");
    unsafe {
        let a = _mm_set_epi32(1, 2, 3, 4);
        let b = _mm_set_epi32(0, 1, 2, 3);
        // 4つの32bit整数の並列足し算
        let add = _mm_add_epi32(a, b);
        print_m128i(&add, 32); // (1, 3, 5, 7)
        // 4つの32bit整数の並列引き算
        let sub = _mm_sub_epi32(a, b);
        print_m128i(&sub, 32); // (1, 1, 1, 1)
        // mulはaとbの下位32bitの掛け算の次は95bit～64bit同士の掛け算になるので 1 * 0と3 * 2の結果は無い
        // これは32bit同士の掛け算を64bitに格納するためで、全要素の掛け算をするにはmulloまたはmulhiを使う
        let mul = _mm_mul_epi32(a, b);
        print_m128i(&mul, 32); // (0, 2, 0, 12)
        // mulloでは掛け算の結果64bitの下位32bitを抽出
        let mullo = _mm_mullo_epi32(a, b);
        print_m128i(&mullo, 32); // (0, 2, 6, 12)
        // mullhiはepi32が存在せずepi16のみで、掛け算の結果32bitの上位16bitを抽出
        let mulhi = _mm_mulhi_epi16(a, b);
        // 今回与えた値は小さいため上位桁は全て0に
        print_m128i(&mulhi, 32); // (0, 0, 0, 0)
    }
    unsafe {
        let a = _mm_set_epi16(0, 1, 2, 3, 4, 5, 6, 7);
        // i16の最大値は32767
        let b = _mm_set1_epi16(32765);
        // 足し算の結果、32767を超えた場合は32767に飽和
        let addsi = _mm_adds_epi16(a, b);
        print_m128i(&addsi, 16); // (32765, 32766, 32767, 32767, 32767, 32767, 32767, 32767)
        // 足し算の結果、32767を超えた場合はオーバーフローして（最上位ビットに1が立ち）負数に
        let add = _mm_add_epi16(a, b);
        print_m128i(&add, 16); // (32765, 32766, 32767, -32768, -32767, -32766, -32765, -32764)
        let c = _mm_set1_epi16(-3);
        // epu16の場合は65535（= -1）で飽和
        let addsu = _mm_adds_epu16(a, c);
        print_m128i(&addsu, 16); // (-3, -2, -1, -1, -1, -1, -1, -1)
        // i16の引き算では負の最小値に飽和
        let d = _mm_set1_epi16(-32766);
        let subsi = _mm_subs_epi16(d, a);
        print_m128i(&subsi, 16); // (-32766, -32767, -32768, -32768, -32768, -32768, -32768, -32768)
        let e = _mm_set1_epi16(2);
        // u16の引き算では0に飽和
        let subsu = _mm_subs_epu16(a, e);
        print_m128i(&subsu, 16); // (0, 0, 0, 1, 2, 3, 4, 5)
        let d = _mm_set_epi32(1, 0, -1, 0);
        // eが0なら0、-1ならマイナスに変換、1ならそのまま
        let sign = _mm_sign_epi32(a, d);
        print_m128i(&sign, 32); // (1, 0, -3, 0)
    }
}

fn compare() {
    println!("-----compare-----");
    unsafe {
        let a = _mm_set_epi32(1, 2, 3, 4);
        let b = _mm_set_epi32(0, 2, 4, 4);
        let c = _mm_cmpeq_epi32(a, b);
        print_m128i(&c, 4); // 0000 0000 ffff ffff 0000 0000 ffff ffff
        let d = _mm_cmpgt_epi32(a, b);
        print_m128i(&d, 4); // ffff ffff 0000 0000 0000 0000 0000 0000
        let e = _mm_cmplt_epi32(a, b);
        print_m128i(&e, 4); // 0000 0000 0000 0000 ffff ffff 0000 0000
    }
}

fn blend() {
    println!("-----blend-----");
    unsafe {
        let a = _mm_set_epi16(128, 64, 32, 16, 8, 4, 2, 1);
        // 16ビット毎に1ビット左シフト（2倍）
        let b = _mm_slli_epi16::<1>(a);
        // 最上位ビットが1のビット列として-1を、最上位ビットが0のビット列として0を与える
        // （-1は全てのビットが1で、0は全てのビットが0であるため）
        let mask = _mm_set_epi16(-1, 0, -1, 0, -1, 0, -1, 0);
        // 16ビット毎に、マスクで最上位ビットが1のときにb、そうでないときにaのビットを採用する
        let c = _mm_blendv_epi8(a, b, mask);
        print_m128i(&a, 16); // (128, 64, 32, 16, 8, 4, 2, 1)
        print_m128i(&b, 16); // (256, 128, 64, 32, 16, 8, 4, 2)
        print_m128i(&mask, 16); // (-1, 0, -1, 0, -1, 0, -1, 0)
        print_m128i(&c, 16); // (256, 64, 64, 16, 16, 4, 4, 1)
    }
}

fn miscellaneous() {
    println!("-----minpos-----");
    unsafe {
        let a = _mm_set_epi16(5, 4, 3, -5, -4, -3, -2, -1);
        // 符号無し整数での最小値とその位置を取得（下位16bitに値が、その次の16bitに位置が格納される）
        // 位置は、0: 0～15, 1: 16～31, 2: 32～63、のように下位桁から順に0から番号付けられている
        let p = _mm_minpos_epu16(a);
        let min = _mm_extract_epi16(p, 0);
        let pos = _mm_extract_epi16(p, 1);
        println!("min = {min}, pos = {pos}"); // min = 3, pos = 5
        // aの4番目の16bitsを0に書き換える
        let b = _mm_insert_epi16::<4>(a, 0);
        let q = _mm_minpos_epu16(b);
        let min = _mm_extract_epi16(q, 0);
        let pos = _mm_extract_epi16(q, 1);
        println!("min = {min}, pos = {pos}"); // min = 0, pos = 4
    }
    println!("-----mask-----");
    unsafe {
        // 16bit整数として(0, -1, ...)を渡すと8bit毎の最上位ビットは(0, 0, 1, 1, ...)となる
        let a = _mm_set_epi16(0, -1, 0, 0, -1, -1, 0, 0);
        // 8bit毎の最上位ビットがbの下位16bitに並ぶ
        let b = _mm_movemask_epi8(a);
        println!("{:016b}", b); // 0011000011110000
    }
    println!("-----horizontal sum-----");
    unsafe {
        // 2を16個8bit毎にセット
        let a = _mm_set1_epi8(2);
        let sum = sum_u8_horizontal(a);
        println!("sum = {sum}"); // sum = 32
    }
}

fn sum_u8_horizontal(m: __m128i) -> i32 {
    unsafe {
        let sum_a = _mm_sad_epu8(m, _mm_setzero_si128());
        _mm_cvtsi128_si32(sum_a) + _mm_extract_epi16(sum_a, 4)
    }
} // cf. https://stackoverflow.com/questions/36998538/fastest-way-to-horizontally-sum-sse-unsigned-byte-vector


fn main() {
    print_alignment();
    load_store();
    shift();
    logical();
    arithmetic();
    compare();
    blend();
    miscellaneous();
}