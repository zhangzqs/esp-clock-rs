use std::{fmt, time::Duration};

use serde::{Deserialize, Serialize};

pub type ToneFrequency = u16;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToneDuration(pub u16);

impl From<Duration> for ToneDuration {
    fn from(value: Duration) -> Self {
        Self(value.as_millis() as u16)
    }
}

impl From<ToneDuration> for Duration {
    fn from(val: ToneDuration) -> Self {
        Duration::from_millis(val.0 as _)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ToneSeries(pub Vec<(ToneFrequency, ToneDuration)>);

impl fmt::Debug for ToneSeries {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let l = self.0.len();
        f.debug_tuple("ToneSeries")
            .field(&format!("[(ToneFrequency, ToneDuration);{l}]"))
            .finish()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum BuzzerMessage {
    /// 不停地播放一个固定音符
    ToneForever(ToneFrequency),
    /// 播放一系列音符
    ToneSeriesRequest(ToneSeries),
    /// true: 播放正常结束
    /// false: 播放Off调用中断结束
    ToneSeriesResponse(bool),
    /// 关闭蜂鸣器播放
    Off,
}
