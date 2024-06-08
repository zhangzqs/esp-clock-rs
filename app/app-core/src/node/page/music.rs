use ipc::StorageClient;
use slint::{ComponentHandle, Model, ModelExt, ModelRc, VecModel};

use crate::{get_app_window, proto::*, storage::MusicStorage, ui};
use std::rc::Rc;

pub struct MusicPage {}

impl MusicPage {
    pub fn new() -> Self {
        Self {}
    }

    fn on_show(ctx: Rc<dyn Context>) {
        let ms = MusicStorage(StorageClient(ctx.clone()));
        if let Some(ui) = get_app_window().upgrade() {
            let vm = ui.global::<ui::MusicPageViewModel>();
            let mrc = ModelRc::new(VecModel::from(ms.get_list()).map(Into::into));
            if mrc.row_count() != 0 {
                vm.set_music_list(mrc);
                let mpc = ipc::MidiPlayerClient(ctx.clone());
                let elem = vm.get_music_list().row_data(0).unwrap();
                let bs = ms.get_data(elem.into());
                mpc.play(bs, Box::new(|_| {}));
            }
        }
    }

    fn on_click(ctx: Rc<dyn Context>) {
        let ms = MusicStorage(StorageClient(ctx.clone()));
        if let Some(ui) = get_app_window().upgrade() {
            let vm = ui.global::<ui::MusicPageViewModel>();
            let len = vm.get_music_list().row_count();
            if len == 0 {
                ctx.clone().async_call(
                    NodeName::AlertDialog,
                    Message::AlertDialog(AlertDialogMessage::ShowRequest {
                        duration: Some(3000),
                        content: AlertDialogContent {
                            text: Some("No music, exit...".into()),
                            image: None,
                        },
                    }),
                    Box::new(move |_| {
                        ctx.sync_call(
                            NodeName::Router,
                            Message::Router(RouterMessage::GotoPage(RoutePage::Menu)),
                        );
                    }),
                )
            } else {
                let idx = (vm.get_select_id() + 1) % len as i32;
                vm.set_select_id(idx);

                let mpc = ipc::MidiPlayerClient(ctx.clone());
                // 切换音乐时候先关闭先前的音乐释放内存
                mpc.off();
                let elem = vm.get_music_list().row_data(idx as usize).unwrap();
                let bs = ms.get_data(elem.into());
                mpc.play(bs, Box::new(|_| {}));
            }
        }
    }
}

impl Node for MusicPage {
    fn node_name(&self) -> NodeName {
        NodeName::MusicPage
    }

    fn handle_message(&self, ctx: Rc<dyn Context>, msg: MessageWithHeader) -> HandleResult {
        match msg.body {
            Message::Lifecycle(msg) => match msg {
                LifecycleMessage::Init => {}
                LifecycleMessage::Show => {
                    ctx.subscribe_topic(TopicName::OneButton);
                    Self::on_show(ctx.clone());
                }
                LifecycleMessage::Hide => {
                    ctx.unsubscribe_topic(TopicName::OneButton);
                }
                _ => {}
            },
            Message::OneButton(msg) => match msg {
                OneButtonMessage::Click => Self::on_click(ctx),
                OneButtonMessage::LongPressHolding(dur) => {
                    if dur > 3000 {
                        ctx.sync_call(
                            NodeName::Router,
                            Message::Router(RouterMessage::GotoPage(RoutePage::Home)),
                        );
                        return HandleResult::Block;
                    }
                }
                _ => {}
            },
            _ => {}
        }
        HandleResult::Discard
    }
}
