use crate::screen::Capturer;

mod screen;
fn main() {
    let mut cap = Capturer::new();
    let buf = cap.capture();
    println!("{}", buf.len());
}
