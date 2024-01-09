use embedded_io_adapters::std::ToStd;
use embedded_svc::http::server::{Connection, Request, Response};

pub fn read_json_from_req_body<T, C>(req: &mut Request<C>) -> anyhow::Result<T>
where
    C: Connection,
    T: serde::de::DeserializeOwned,
{
    let r = ToStd::new(req);
    Ok(serde_json::from_reader::<_, T>(r)?)
}

pub fn write_json_to_resp_body<T, C>(resp: &mut Response<C>, data: &T) -> anyhow::Result<()>
where
    C: Connection,
    T: serde::Serialize,
{
    let w = ToStd::new(resp);
    serde_json::to_writer(w, data)?;
    Ok(())
}
