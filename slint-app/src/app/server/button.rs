use std::fmt::Debug;

use crate::{
    util::{read_json_from_req_body, write_json_to_resp_body},
    AppWindow,
};
use embedded_svc::http::server::{Connection, HandlerResult, Request};
use log::{debug, warn};
use serde::{Deserialize, Serialize};
use slint::Weak;

#[derive(Serialize, Deserialize, Debug)]
struct ButtonControl {
    pub event: String,
    pub clicks: u32,
    pub duration: u32,
}

pub fn button_handler<C>(mut req: Request<C>, app: Weak<AppWindow>) -> HandlerResult
where
    C: Connection,
{
    let btn: ButtonControl = read_json_from_req_body(&mut req)?;
    debug!("reveive a button event: {:?}", btn);

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
    let mut resp = req.into_ok_response()?;
    write_json_to_resp_body(&mut resp, &btn)?;
    Ok(())
}
