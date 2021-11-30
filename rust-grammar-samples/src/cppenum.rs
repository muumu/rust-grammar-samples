pub type ColorType = usize;
pub const RED: ColorType = 0;
pub const GREEN: ColorType = 1;
pub const BLUE: ColorType = 2;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Color(usize);

impl Color {
    pub const RED: Self = Self(0);
    pub const GREEN: Self = Self(1);
    pub const BLUE: Self = Self(2);
    pub fn from_usize(n: usize) -> Self { Self(n) }
    pub fn to_usize(&self) -> usize { self.0 }
}

fn print_color_type(c: ColorType) {
    println!("c: ColorType = {}", c);
}

fn print_color(c: Color) {
    println!("c: Color = {}", c.to_usize());
}

pub fn cppenum() {
    let v = vec![1, 2, 3];
    let ct = RED;
    println!("{}", v[ct]); // Vecの添字に使用可能
    print_color_type(0);
    let cs = Color::RED;
    // error[E0277]: the type `[{integer}]` cannot be indexed by `Color`
    // println!("{}", v[cs]); // structは直接添字に使用できない
    println!("{}", v[cs.to_usize()]);
    let n: usize = 1;
    print_color(Color::from_usize(n));

}