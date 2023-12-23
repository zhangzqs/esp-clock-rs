use std::{
    marker::PhantomData,
    thread,
    time::Duration,
};

use crate::{
    interface::{Server, ServerBuilder},
    AppWindow,
};
use include_dir::{include_dir, Dir};
use log::{debug, info, warn};

use embedded_svc::http::{
    server::{Connection, Handler, HandlerResult},
    Method,
};
use serde::Deserialize;
use slint::Weak;

static VUE_DIST: Dir = include_dir!("console-dist");

struct VueConsoleHandler;

impl<C> Handler<C> for VueConsoleHandler
where
    C: Connection,
{
    fn handle(&self, c: &mut C) -> HandlerResult {
        let u = c.uri();
        info!("receive http request uri: {}", u);
        // 提取出url的path部分
        let path = if let Some(idx) = u.find('?') {
            &u[1..idx]
        } else {
            &u[1..]
        };
        let file_path = if path.is_empty() { "index.html" } else { path };
        if let Some(f) = VUE_DIST.get_file(file_path) {
            let content_type = match file_path.split('.').last() {
                Some("html") => "text/html",
                Some("js") => "application/javascript",
                Some("css") => "text/css",
                Some("png") => "image/png",
                Some("ico") => "image/x-icon",
                Some("svg") => "image/svg+xml",
                _ => "",
            };

            if f.contents().starts_with(&[0x1f, 0x8b]) {
                c.initiate_response(
                    200,
                    Some("OK"),
                    &[("Content-Type", content_type), ("Content-Encoding", "gzip")],
                )?;
            } else {
                c.initiate_response(200, Some("OK"), &[("Content-Type", content_type)])?;
            };
            c.write_all(f.contents())?;
        } else {
            c.initiate_response(404, Some("Not Found"), &[("Content-Type", "")])?;
            c.write_all(b"Page Not Found")?;
        }
        Ok(())
    }
}

pub struct HttpServerApp<SB>
where
    SB: ServerBuilder<'static>,
{
    app: Weak<AppWindow>,
    _phantom: PhantomData<SB>,
}

#[derive(Deserialize, Debug)]
struct ButtonControl {
    pub event: String,
    pub clicks: u32,
    pub duration: u32,
}

// struct Embedded2StdReaderWrapper<'a, R>(&'a mut R)
// where
//     R: embedded_io::Read;

// impl<'a, R> std::io::Read for Embedded2StdReaderWrapper<'a, R>
// where
//     R: embedded_io::Read,
// {
//     fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
//         self.0.read(buf).map_err(|e| {
//             e.into()
//         })
//     }
// }

impl<SB> HttpServerApp<SB>
where
    SB: ServerBuilder<'static>,
{
    pub fn new(app: Weak<AppWindow>) -> Self {
        let app_ref = app.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_secs(10));
            let mut server = SB::new().build().unwrap();
            server
                .handler("/*", Method::Get, VueConsoleHandler)
                .unwrap()
                .fn_handler("/control/button", Method::Post, move |mut req| {
                    let mut buf = [0u8; 512];
                    let read_size = req.read(&mut buf)?;
                    let buf = &buf[..read_size];
                    debug!("buf: {}", String::from_utf8_lossy(buf));
                    let btn = serde_json::from_slice::<ButtonControl>(buf)?;
                    if let Some(ui) = app_ref.upgrade() {
                        match btn.event.as_str() {
                            "click" => {
                                ui.invoke_on_one_button_clicks(btn.clicks as _);
                            }
                            "longPressedHeld" => {
                                ui.invoke_on_one_button_long_pressed_held(btn.duration as _);
                            }
                            "longPressedHolding" => {
                                ui.invoke_on_one_button_long_pressed_holding(btn.duration as _);
                            }
                            _ => {
                                warn!("invalid event type");
                            }
                        }
                    }
                    debug!("reveive a button event: {:?}", btn);
                    req.into_ok_response()?;
                    Ok(())
                })
                .unwrap();
            loop {
                thread::sleep(Duration::from_secs(1));
            }
        });

        Self {
            app,
            _phantom: PhantomData,
        }
    }
}
