use std::{marker::PhantomData, thread, time::Duration};

use crate::{
    interface::{Server, ServerBuilder},
    AppWindow,
};
use embedded_io::Write;
use include_dir::{include_dir, Dir};


use embedded_svc::http::{
    Method,
};

use slint::Weak;

mod static_file;
use static_file::StaticFileHandler;
mod button;

static VUE_DIST: Dir = include_dir!("console-dist");

pub struct HttpServerApp<SB>
where
    SB: ServerBuilder<'static>,
{
    app: Weak<AppWindow>,
    _phantom: PhantomData<SB>,
}

impl<SB> HttpServerApp<SB>
where
    SB: ServerBuilder<'static>,
{
    pub fn new(app: Weak<AppWindow>) -> Self {
        let app_ref = app.clone();
        thread::spawn(move || -> anyhow::Result<()> {
            thread::sleep(Duration::from_secs(1));
            let mut server = SB::new().uri_match_wildcard(true).build().unwrap();
            server
                .handler("/*", Method::Get, StaticFileHandler(&VUE_DIST))?
                .fn_handler("/control/button", Method::Post, move |req| {
                    button::button_handler(req, app_ref.clone())
                })?
                .fn_handler("/tone/music/start", Method::Post, |_| todo!())?
                .fn_handler("/tone/music/stop", Method::Post, |_| todo!())?
                .fn_handler("/tone/realtime", Method::Post, |_| todo!())?
                .fn_handler("/ping", Method::Get, |req| {
                    req.into_ok_response()?.write_all(b"pong")?;
                    Ok(())
                })?
                .fn_handler("/wifi/scan", Method::Get, |_| todo!())?
                .fn_handler("/wifi/connect", Method::Post, |_| todo!())?
                .fn_handler("/weather/city_lookup", Method::Get, |_| todo!())?
                .fn_handler("/weather/now", Method::Get, |_| todo!())?
                .fn_handler("/storage", Method::Get, |_| todo!())?
                .fn_handler("/storage", Method::Put, |_| todo!())?;
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
