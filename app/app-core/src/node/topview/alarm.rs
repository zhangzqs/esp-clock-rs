use std::cell::RefCell;
use std::collections::HashMap;
use std::default;
use std::{rc::Rc, time::Duration};

use ipc::StorageClient;
use proto::TopicName;
use slint::ComponentHandle;
use storage::UserAlarmStorage;
use time::{UtcOffset, Weekday};

use crate::proto::*;
use crate::{get_app_window, ui};

type Result<T> = std::result::Result<T, UserAlarmError>;

struct AlarmElement {
    hour: u8,
    minute: u8,
    mode: UserAlarmRepeatMode,
    last_active_day: RefCell<Option<u8>>,
}
type AlarmList = HashMap<usize, AlarmElement>;

#[derive(Default)]
struct State {
    alarm_list: RefCell<AlarmList>,
}

#[derive(Default)]
pub struct UserAlarmService {
    timer: slint::Timer,
    state: Rc<State>,
}

impl UserAlarmService {
    pub fn new() -> Self {
        Self::default()
    }

    fn on_alarm(ctx: Rc<dyn Context>, id: usize) {
        // play
        let stg = UserAlarmStorage(StorageClient(ctx.clone()));
        let ret = stg.get(id).map_err(UserAlarmError::StorageError).unwrap();

        let t = slint::Timer::default();
        t.start(slint::TimerMode::Repeated, Duration::from_secs(2), {
            let ctx = ctx.clone();
            move || {
                let cli = ipc::BuzzerClient(ctx.clone());
                let freq = 2000;
                let d1 = 100;
                let d2 = 50;
                cli.tone_series(
                    ToneSeries(vec![
                        (freq, ToneDuration(d1)),
                        (0, ToneDuration(d2)),
                        (freq, ToneDuration(d1)),
                        (0, ToneDuration(d2)),
                        (freq, ToneDuration(d1)),
                        (0, ToneDuration(d2)),
                        (freq, ToneDuration(d1)),
                        (0, ToneDuration(500)),
                        (freq, ToneDuration(d1)),
                        (0, ToneDuration(d2)),
                        (freq, ToneDuration(d1)),
                        (0, ToneDuration(d2)),
                        (freq, ToneDuration(d1)),
                        (0, ToneDuration(d2)),
                        (freq, ToneDuration(d1)),
                        (0, ToneDuration(d2)),
                    ]),
                    Box::new(|r| {}),
                );
            }
        });
        ctx.async_call(
            NodeName::AlertDialog,
            Message::AlertDialog(AlertDialogMessage::ShowRequest {
                duration: Some(2 * 60 * 1000), // 持续2分钟
                content: AlertDialogContent {
                    text: Some(format!("闹铃！！！{ret:?}")),
                    image: None,
                },
            }),
            Box::new(move |r| {
                // off play
                t.stop();
            }),
        );
    }

    fn on_timer_check(ctx: Rc<dyn Context>, state: Rc<State>) {
        let stg = UserAlarmStorage(StorageClient(ctx.clone()));

        let now = time::OffsetDateTime::now_utc().to_offset(UtcOffset::from_hms(8, 0, 0).unwrap());
        let (now_day, now_hh, now_mm) = (now.day(), now.hour(), now.minute());
        let weekday = now.weekday();
        let mut need_droped = Vec::new();
        for (id, e) in state.alarm_list.borrow().iter() {
            if now_hh == e.hour && now_mm == e.minute && e.mode.is_active(weekday) {
                // 闹钟今日是否响过了
                let today_actived = match e.last_active_day.borrow().as_ref() {
                    Some(x) => *x == now_day,
                    None => false,
                };
                if !today_actived {
                    // 闹铃应该被响起
                    Self::on_alarm(ctx.clone(), *id);
                    // 标记一下闹铃今日已响过
                    *e.last_active_day.borrow_mut() = Some(now_day);

                    if let UserAlarmRepeatMode::Once = e.mode {
                        stg.delete(*id).unwrap();
                        need_droped.push(*id);
                    }
                }
            }
        }
        // 移除一些一次性闹铃
        for id in need_droped.into_iter() {
            state.alarm_list.borrow_mut().remove(&id);
        }
    }

    fn init(&self, ctx: Rc<dyn Context>) {
        let stg = UserAlarmStorage(StorageClient(ctx));
        let id_list = stg
            .get_id_list()
            .map_err(UserAlarmError::StorageError)
            .unwrap_or_default();
        for id in id_list {
            let body = stg.get(id).unwrap();
            let ele = AlarmElement {
                hour: body.time.0,
                minute: body.time.1,
                mode: body.repeat_mode.clone(),
                last_active_day: Default::default(),
            };
            self.state.alarm_list.borrow_mut().insert(id, ele);
        }
    }

    fn add(&self, ctx: Rc<dyn Context>, body: UserAlarmBody) -> Result<HandleResult> {
        let stg = UserAlarmStorage(StorageClient(ctx));
        let ele = AlarmElement {
            hour: body.time.0,
            minute: body.time.1,
            mode: body.repeat_mode.clone(),
            last_active_day: Default::default(),
        };
        let id = stg.add(body).map_err(UserAlarmError::StorageError)?;
        self.state.alarm_list.borrow_mut().insert(id, ele);
        Ok(HandleResult::Finish(Message::UserAlarm(
            UserAlarmMessage::AddResponse(id),
        )))
    }

    fn delete(&self, ctx: Rc<dyn Context>, id: usize) -> Result<HandleResult> {
        let stg = UserAlarmStorage(StorageClient(ctx));
        let ret = stg.delete(id).map_err(UserAlarmError::StorageError)?;
        self.state.alarm_list.borrow_mut().remove(&id);
        Ok(HandleResult::Finish(Message::UserAlarm(
            UserAlarmMessage::DeleteResponse(ret),
        )))
    }

    fn get(&self, ctx: Rc<dyn Context>, id: usize) -> Result<HandleResult> {
        let stg = UserAlarmStorage(StorageClient(ctx));
        let ret = stg.get(id).map_err(UserAlarmError::StorageError)?;
        Ok(HandleResult::Finish(Message::UserAlarm(
            UserAlarmMessage::GetResponse(ret),
        )))
    }

    fn list(&self, ctx: Rc<dyn Context>) -> Result<HandleResult> {
        let stg = UserAlarmStorage(StorageClient(ctx));
        let ret = stg.get_id_list().map_err(UserAlarmError::StorageError)?;
        Ok(HandleResult::Finish(Message::UserAlarm(
            UserAlarmMessage::ListResponse(ret),
        )))
    }

    fn handle_error(r: Result<HandleResult>) -> HandleResult {
        match r {
            Ok(x) => x,
            Err(e) => HandleResult::Finish(Message::UserAlarm(UserAlarmMessage::Error(e))),
        }
    }
}

impl Node for UserAlarmService {
    fn node_name(&self) -> NodeName {
        NodeName::Alarm
    }

    fn handle_message(&self, ctx: Rc<dyn Context>, msg: MessageWithHeader) -> HandleResult {
        match msg.body {
            Message::Lifecycle(m) => match m {
                LifecycleMessage::Init => {
                    self.init(ctx.clone());
                    let state = self.state.clone();
                    self.timer
                        .start(slint::TimerMode::Repeated, Duration::from_secs(10), {
                            let ctx = ctx.clone();
                            move || {
                                Self::on_timer_check(ctx.clone(), state.clone());
                            }
                        });
                    return HandleResult::Finish(Message::Empty);
                }
                _ => {}
            },
            Message::UserAlarm(m) => match m {
                UserAlarmMessage::AddRequest(req) => {
                    return Self::handle_error(self.add(ctx, req));
                }
                UserAlarmMessage::DeleteRequest(req) => {
                    return Self::handle_error(self.delete(ctx, req));
                }
                UserAlarmMessage::GetRequest(req) => {
                    return Self::handle_error(self.get(ctx, req));
                }
                UserAlarmMessage::ListRequest => {
                    return Self::handle_error(self.list(ctx));
                }
                _ => {}
            },
            _ => {}
        }
        HandleResult::Discard
    }
}
