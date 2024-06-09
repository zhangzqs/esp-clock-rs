use std::{
    io::{stdin, Read, Stdin},
    process::Stdio,
    rc::Rc,
    sync::mpsc,
    thread,
    time::Duration,
};

use app_core::proto::*;
use clap::Subcommand;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ipc::{StorageClient, SystemClient, WeatherClient};
use log::{debug, info};
use storage::{MusicStorage, WeatherStorage, WiFiStorage};
use tui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders},
    Terminal,
};

mod onebutton;

#[derive(Subcommand)]
pub enum SubCommands {
    StorageList {
        #[arg()]
        prefix: Option<String>,
    },
    MusicList,
    MusicUpload {
        mid_file: String,
    },
    WifiConfig {
        #[arg()]
        ssid: String,
        #[arg()]
        password: Option<String>,
    },
    WeatherSetKey {
        key: String,
    },
    WeatherSearch {
        query: String,
    },
    WeatherSetLocation {
        location_id: u32,
        location: String,
    },
    MonitorEnable {
        #[arg()]
        enable: i8,
    },
    AlertDialog {
        text: String,
    },

    #[clap(name = "onebutton")]
    OneButton,
}

impl SubCommands {
    pub fn run(self, ctx: Rc<dyn Context>) -> anyhow::Result<()> {
        match self {
            SubCommands::StorageList { prefix } => {
                let stg = ipc::StorageClient(ctx);
                let keys = stg.list(prefix.unwrap_or_default()).unwrap();
                for ref k in keys.into_iter() {
                    let v = stg.get(k.into()).unwrap();
                    println!("{k}\t{v:?}");
                }
            }
            SubCommands::MusicUpload { mid_file } => {
                let mut f = std::fs::File::open(&mid_file).unwrap();
                let mut buffer = Vec::new();
                f.read_to_end(&mut buffer).unwrap();
                MusicStorage(StorageClient(ctx)).upload(mid_file, buffer);
                info!("upload done");
            }
            SubCommands::MusicList => {
                let list = MusicStorage(StorageClient(ctx)).get_list();
                for e in list.into_iter() {
                    println!("{e}");
                }
            }
            SubCommands::OneButton => loop {
                let stdin = std::io::stdin();
                let mut buffer = [0u8; 1]; // 用于存储单个字节的缓冲区

                loop {
                    // 读取单个字节的输入
                    match stdin.lock().read_exact(&mut buffer) {
                        Ok(_) => {
                            let c = buffer[0] as char;
                            println!("输入的字符: {}", c);

                            match c {
                                'q' => break,
                                '1' => {
                                    ctx.broadcast_topic(
                                        TopicName::OneButton,
                                        Message::OneButton(OneButtonMessage::Click),
                                    );
                                }
                                '2' => {
                                    ctx.broadcast_topic(
                                        TopicName::OneButton,
                                        Message::OneButton(OneButtonMessage::Clicks(2)),
                                    );
                                }
                                _ => {}
                            }
                        }
                        Err(error) => {
                            eprintln!("读取输入时出错: {}", error);
                            break;
                        }
                    }
                }
            },
            SubCommands::WifiConfig { ssid, password } => {
                let ws = WiFiStorage(StorageClient(ctx.clone()));
                ws.set_ssid(Some(ssid));
                ws.set_password(password);
                SystemClient(ctx.clone()).restart();
            }
            SubCommands::MonitorEnable { enable } => {
                ctx.sync_call(
                    NodeName::BootPage,
                    Message::BootPage(BootPageMessage::EnableSystemMonitor(match enable {
                        0 => false,
                        1 => true,
                        n => {
                            panic!("invalid number {n}");
                        }
                    })),
                );
            }
            SubCommands::WeatherSetKey { key } => {
                WeatherStorage(StorageClient(ctx)).set_key(key).unwrap();
            }
            SubCommands::WeatherSearch { query } => {
                WeatherClient(ctx).city_lookup(
                    query,
                    Box::new(|r| {
                        for i in r.unwrap().into_iter() {
                            println!("{:?}", i);
                        }
                    }),
                );
            }
            SubCommands::WeatherSetLocation {
                location_id,
                location,
            } => {
                WeatherStorage(StorageClient(ctx))
                    .set_location(location_id, location)
                    .unwrap();
            }
            SubCommands::AlertDialog { text } => ctx.async_call(
                NodeName::AlertDialog,
                Message::AlertDialog(AlertDialogMessage::ShowRequest {
                    duration: Some(3000),
                    content: AlertDialogContent {
                        text: Some(text),
                        image: None,
                    },
                }),
                Box::new(|r| {}),
            ),
        }
        anyhow::Ok(())
    }
}
