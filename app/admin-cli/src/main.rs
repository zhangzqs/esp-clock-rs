use std::rc::Rc;

use app_core::proto::*;
use clap::Parser;
use reqwest::blocking::Client;
use serde_json::json;

mod subcmds;

#[derive(Parser)]
struct AdminCli {
    /// 是否使用debug模式（默认是info级别）
    #[clap(long, short = 'D')]
    debug: bool,

    /// url
    #[clap(long)]
    url: Option<String>,

    /// 子命令
    #[clap(subcommand)]
    subcmd: subcmds::SubCommands,
}

struct ContextImpl {
    client: Client,
    url: String,
}

impl ContextImpl {
    fn new(url: String) -> Self {
        let client = Client::new();
        Self { client, url }
    }

    fn send_message(&self, to: MessageTo, body: Message) -> HandleResult {
        let req = self
            .client
            .post(&self.url)
            .json(&json!({
                "to": to,
                "body": body,
            }))
            .build()
            .unwrap();
        let resp = self.client.execute(req).unwrap();
        resp.json().unwrap()
    }
}

impl Context for ContextImpl {
    fn boardcast(&self, msg: Message) {
        self.send_message(MessageTo::Broadcast, msg);
    }

    fn async_call(&self, node: NodeName, msg: Message, callback: MessageCallbackOnce) {
        callback(self.send_message(MessageTo::Point(node), msg))
    }

    fn sync_call(&self, node: NodeName, msg: Message) -> HandleResult {
        self.send_message(MessageTo::Point(node), msg)
    }

    fn async_ready(&self, seq: usize, result: Message) {
        unimplemented!()
    }
}

fn main() -> anyhow::Result<()> {
    let cli = AdminCli::parse();
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    } else if cli.debug {
        std::env::set_var("RUST_LOG", "debug");
    }

    let ctx = Rc::new(ContextImpl::new(
        cli.url
            .unwrap_or_else(|| std::env::var("CLOCK_URL").expect("no url")),
    ));
    env_logger::init();
    cli.subcmd.run(ctx)?;
    anyhow::Ok(())
}