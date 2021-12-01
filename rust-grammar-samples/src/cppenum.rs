// 型チェックによるusizeとの区別が不要な場合はエイリアスが便利
pub type ColorType = usize;
pub const RED: ColorType = 0;
pub const GREEN: ColorType = 1;
pub const BLUE: ColorType = 2;

pub trait Foo {
    fn foo(&self) { println!("foo!"); }
}

impl Foo for ColorType {}

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

// 型チェックと範囲外の数値のエラーハンドリングが必要な場合はenumが安全で便利
#[derive(FromPrimitive, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(usize)]
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
    for c in RED..BLUE { // for文の範囲にも使用可能
        println!("{}", c);
    }
    println!("size_of ColorType: {}", std::mem::size_of::<ColorType>());
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
    for c in Color::RED.0..Color::BLUE.0 {
        println!("{}", c);
    }
    println!("size_of Color: {}", std::mem::size_of::<Color>());
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
    for c in ColorEnum::Red as usize..ColorEnum::Blue as usize {
        println!("{}", c as usize);
    }
    println!("size_of ColorEnum: {}", std::mem::size_of::<ColorEnum>());
}