use embedded_svc::http::server::{Connection, Handler, HandlerResult};
use include_dir::Dir;
use log::info;

pub struct StaticFileHandler<'a>(pub &'a Dir<'a>);

impl<'a, C> Handler<C> for StaticFileHandler<'a>
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
        if let Some(f) = self.0.get_file(file_path) {
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
