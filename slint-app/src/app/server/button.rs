use std::fmt::Debug;

use crate::AppWindow;
use embedded_svc::http::server::{Connection, HandlerResult, Request};
use log::{debug, warn};
use serde::Deserialize;
use slint::Weak;

#[derive(Deserialize, Debug)]
struct ButtonControl {
    pub event: String,
    pub clicks: u32,
    pub duration: u32,
}

pub fn read_from_req_body<const S: usize, T, C>(req: &mut Request<C>) -> anyhow::Result<T>
where
    C: Connection,
    T: serde::de::DeserializeOwned,
{
    let mut buf = [0u8; S];
    let read_size: usize = req.read(&mut buf).map_err(|x| anyhow::anyhow!("{:?}", x))?;
    let buf = &buf[..read_size];
    debug!("buf: {}", String::from_utf8_lossy(buf));
    Ok(serde_json::from_slice::<T>(buf)?)
}

pub fn button_handler<C>(mut req: Request<C>, app: Weak<AppWindow>) -> HandlerResult
where
    C: Connection,
{
    let btn = read_from_req_body::<512, ButtonControl, _>(&mut req)?;
    if let Some(ui) = app.upgrade() {
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
}
