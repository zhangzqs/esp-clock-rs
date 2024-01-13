use std::{
    marker::PhantomData,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crate::{
    interface::{Server, ServerBuilder},
    resources,
    util::{read_json_from_req_body, write_json_to_resp_body},
    AppWindow, LEDController,
};
use embedded_io::Write;


use embedded_svc::http::Method;

use slint::Weak;

mod static_file;
use static_file::StaticFileHandler;
mod button;
mod tone;

pub struct HttpServerApp<SB, SCBC>
where
    SB: ServerBuilder<'static>,
    SCBC: LEDController + 'static + Send,
{
    app: Weak<AppWindow>,
    screen_brightness_controller: Arc<Mutex<SCBC>>,
    _phantom: PhantomData<SB>,
}

impl<SB, SCBC> HttpServerApp<SB, SCBC>
where
    SB: ServerBuilder<'static>,
    SCBC: LEDController + 'static + Send,
{
    pub fn new(app: Weak<AppWindow>, screen_brightness_controller: Arc<Mutex<SCBC>>) -> Self {
        let app_ref = app.clone();
        let sbc = screen_brightness_controller.clone();
        thread::spawn(move || -> anyhow::Result<()> {
            thread::sleep(Duration::from_secs(20));
            let mut server = SB::new().uri_match_wildcard(true).build().unwrap();
            let sbc1 = sbc.clone();
            let sbc2 = sbc.clone();
            server
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
                .fn_handler("/storage", Method::Put, |_| todo!())?
                .fn_handler("/screen/brightness", Method::Get, move |req| {
                    let data = sbc1.lock().unwrap().get_brightness_percent();
                    let mut resp = req.into_ok_response()?;
                    write_json_to_resp_body(&mut resp, &data)?;
                    Ok(())
                })?
                .fn_handler("/screen/brightness", Method::Put, move |mut req| {
                    let data = read_json_from_req_body(&mut req)?;
                    sbc2.lock().unwrap().set_brightness_percent(data);
                    let _ = req.into_ok_response()?;
                    Ok(())
                })?
                .handler("/*", Method::Get, StaticFileHandler(&resources::VUE_DIST))?;

            loop {
                thread::sleep(Duration::from_secs(1));
            }
        });

        Self {
            app,
            screen_brightness_controller,
            _phantom: PhantomData,
        }
    }
}
