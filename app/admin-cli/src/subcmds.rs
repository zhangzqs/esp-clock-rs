use std::{rc::Rc};

use app_core::proto::*;
use clap::Subcommand;

mod onebutton;

#[derive(Subcommand)]
pub enum SubCommands {
    StorageList {
        #[arg()]
        prefix: Option<String>,
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
            SubCommands::OneButton => {
                onebutton::run()?;
            }
        }
        anyhow::Ok(())
    }
}
