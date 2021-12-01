extern crate num;
#[macro_use]
extern crate num_derive;

mod cppenum;

fn main() {
    cppenum::use_color_type();
    cppenum::use_color_struct();
    cppenum::use_color_enum();
}
