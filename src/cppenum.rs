// 型チェックによるusizeとの区別が不要な場合はエイリアスが便利
pub type ColorType = usize;
pub const RED: ColorType = 0;
pub const GREEN: ColorType = 1;
pub const BLUE: ColorType = 2;

// 以下の場合はタプル構造体が便利
// 1. 型チェックは欲しいが範囲外の数値は気にしない場合
// 2. 様々な数値型にキャストしたい場合
// 3. 様々な数値型からゼロコストで生成したい場合
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Color(pub usize); // モジュール外からColor(1)のように初期化をするならpubの指定が必要

impl Color {
    pub const RED: Self = Self(0);
    pub const GREEN: Self = Self(1);
    pub const BLUE: Self = Self(2);
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Color::RED => write!(f, "Color::RED"),
            Color::GREEN => write!(f, "Color::GREEN"),
            Color::BLUE => write!(f, "Color::BLUE"),
            _ => write!(f, "{}", self.0),
        }
    }
}

// 型チェックと範囲外の数値のエラーハンドリングが必要な場合はenumが安全で便利
#[derive(FromPrimitive, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(usize)] // これを指定すると各値がusizeに
enum ColorEnum {
    Red,
    Green,
    Blue,
}

fn print_color_type(c: ColorType) {
    println!("{:?}", c); // 1 (cが1のとき)
}

fn print_color(c: Color) {
    println!("{:?}", c); // Color(1) (cが1のとき)
}

fn print_color_enum(c: ColorEnum) {
    println!("{:?}", c); // Green (cが1のとき)
}

pub fn use_color_type() {
    let v = vec![1, 2, 3];
    let ct = RED;
    println!("{}", v[ct]); // Vecの添字に使用可能
    let n: usize = 1;
    print_color_type(n); // usizeの変数をそのまま入れられる
    println!("{:?}", ct < GREEN); // true
    println!("{:?}", ct == RED); // true
    for c in RED..=BLUE { // for文の範囲にも使用可能
        println!("{}", c);
    }
    println!("size_of ColorType: {}", std::mem::size_of::<ColorType>()); // 8 (64 bits)
    pub trait Foo { fn foo(&self) { println!("foo!"); } }
    impl Foo for ColorType {}
    RED.foo(); // ColorTypeにimplしたfoo関数の呼び出し
    1.foo(); // ColorTypeにimplするとusizeにもimplしたことになる
}

pub fn use_color_struct() {
    let v = vec![1, 2, 3];
    let cs = Color::RED;
    // error[E0277]: the type `[{integer}]` cannot be indexed by `Color`
    // println!("{}", v[cs]); // structは直接添字に使用できない
    println!("{}", v[cs.0]); // タプル構造体の1つ目のメンバは0でアクセス可能
    let n: usize = 1;
    print_color(Color(n)); // タプル構造体の初期化
    println!("{:?}", cs < Color::GREEN); // true
    println!("{:?}", cs == Color::RED); // true
    // for文の範囲指定では.0でusizeのメンバを取り出す（もしくはColorにIteratorを実装）
    for c in Color::RED.0..=Color::BLUE.0 {
        println!("{}", c);
    }
    println!("size_of Color: {}", std::mem::size_of::<Color>()); // 8 (64 bits)
    println!("{}", cs); // Color::RED
}

pub fn use_color_enum() {
    let v = vec![1, 2, 3];
    let ce = ColorEnum::Red;
    // error[E0277]: the type `[{integer}]` cannot be indexed by `ColorEnum`
    // println!("{}", v[ce]); // enumは直接添字に使用できない
    println!("{}", v[ce as usize]); // enum -> usizeへはasでキャスト可能
    let n: usize = 1;
    print_color_enum(num::FromPrimitive::from_usize(n).unwrap());
    println!("{:?}", ce < ColorEnum::Green); // true
    println!("{:?}", ce == ColorEnum::Red); // true
    // for文の範囲指定ではas usizeが必要。（もしくはColorEnumにIteratorを実装）
    for c in ColorEnum::Red as usize..=ColorEnum::Blue as usize {
        println!("{}", c as usize);
    }
    println!("size_of ColorEnum: {}", std::mem::size_of::<ColorEnum>()); // 8 (64 bits)
    println!("size_of ColorEnum::Red: {}", std::mem::size_of_val(&ColorEnum::Red)); // 8 (64 bits)
}

/* C++におけるenum
#include <iostream>
#include <vector>

enum Color {
    Red,
    Green,
    Blue
};

void print(Color c) {
    std::cout << c << std::endl;
}

int main() {
    std::vector<int> v = {1, 2, 3};
    Color c = Color::Red;
    std::cout << v[c] << std::endl; // vectorの添字としてそのまま使用可能
    // error: invalid conversion from 'int' to 'Color' [-fpermissive]
    // print(0); // enum型にキャストしないとコンパイルエラー
    print(static_cast<Color>(1)); // キャストすればOK
    print(static_cast<Color>(100)); // しかし範囲外の値もキャストできてしまう
    std::cout << (c < Color::Green) << std::endl; // 1 (true)
    std::cout << (c == Color::Red) << std::endl; // 1 (true)
    // enumは++や+=が使用できないので、static_castでunsignedに変換してからenumに戻す必要あり
    for (Color i = Color::Red; i <= Color::Blue; i = static_cast<Color>(static_cast<unsigned>(i) + 1)) {
        std::cout << i << std::endl;
    }
    std::cout << sizeof(Color) << std::endl; // 4 (32 bits)
    std::cout << (sizeof Color::Red) << std::endl; // 4 (32 bits)
}
 */