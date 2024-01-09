use std::{
    io::Write,
    sync::{Arc, Mutex},
    thread,
    time::Duration, error::Error,
};

use crate::screen::Capturer;
use image::{imageops::resize, imageops::FilterType};
mod screen;
fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args();
    if args.len() <= 1 {
        println!("usage: {} <ip:port>", args.next().unwrap());
        return Ok(());
    }
    let addr = args.nth(1).unwrap();
    let mut stream = std::net::TcpStream::connect(addr)?;
    println!("connect success");

    let mut cap = Capturer::new();
    let (w, h) = cap.size();
    let (w, h) = (w as u32, h as u32);

    let fps = Arc::new(Mutex::new(0));
    let fps_ref = fps.clone();
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(1));
        let mut fps_ref = fps_ref.lock().unwrap();
        println!("fps: {}", *fps_ref);
        *fps_ref = 0;
        drop(fps_ref);
    });
    loop {
        let buf = cap.capture();
        let img = image::ImageBuffer::from_fn(w, h, |x, y| {
            let idx = 4 * (y * w + x) as usize;
            image::Rgba([buf[idx + 2], buf[idx + 1], buf[idx], buf[idx + 3]])
        });
        let img = resize(&img, 240, 240, FilterType::Nearest);
        let buf = img
            .enumerate_pixels()
            .flat_map(|(_, _, p)| {
                let p = p.0;
                [p[0], p[1], p[2]]
            })
            .collect::<Vec<_>>();
        *fps.lock().unwrap() += 1;
        // 连接到tcp投屏服务器
        stream.write_all(&buf)?;
        stream.flush()?;
    }
}
