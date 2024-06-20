use std::rc::Rc;

use clap::Parser;
use log::info;
use proto::*;
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
        let msg = json!({
            "to": to,
            "body": body,
        });
        info!("send msg: {}", msg);
        let req = self.client.post(&self.url).json(&msg).build().unwrap();
        let resp = self.client.execute(req).unwrap();
        resp.json().unwrap()
    }
}

impl Context for ContextImpl {
    fn broadcast_global(&self, msg: Message) {
        self.send_message(MessageTo::Broadcast, msg);
    }

    fn broadcast_topic(&self, topic: TopicName, msg: Message) {
        self.send_message(MessageTo::Topic(topic), msg);
    }

    fn async_call(&self, node: NodeName, msg: Message, callback: MessageCallbackOnce) {
        callback(self.send_message(MessageTo::Point(node), msg))
    }

    fn sync_call(&self, node: NodeName, msg: Message) -> HandleResult {
        self.send_message(MessageTo::Point(node), msg)
    }

    fn async_ready(&self, _seq: usize, _result: Message) {
        unimplemented!()
    }

    fn subscribe_topic(&self, _topic: TopicName) {
        unimplemented!()
    }

    fn unsubscribe_topic(&self, _topic: TopicName) {
        unimplemented!()
    }

    fn create_wait_group(&self) -> Rc<dyn WaitGroup> {
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
