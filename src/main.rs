mod virt_pages;
use virt_pages::*;

fn main() {
    let p = Page::new(0xdeadbeef);

    println!("{:?}", p);
}
