use serde::{Deserialize, Serialize};

#[derive(Debug, Hash, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum NodeName {
    // App框架调度器
    Scheduler,
    // Http客户端
    HttpClient,
    // 天气查询器
    WeatherClient,
    // 获取时间的客户端
    TimestampClient,
    // 定时器服务
    Timer,
    // 单个操作按钮事件产生器，可产生一些按键事件
    OneButton,
    // 页面ui路由器
    Router,
    // 本地存储
    Storage,
    // WiFi
    WiFi,
    // 蜂鸣器
    Buzzer,
    // MIDI播放器
    Midi,
    // 性能监视
    Performance,
    // 全局Alert对话框
    AlertDialog,
    // 启动页ui与逻辑
    BootPage,
    // 首页ui与逻辑
    HomePage,
    // 菜单页ui与逻辑
    MenuPage,
    // 天气页ui与逻辑
    WeatherPage,
    // 音乐播放器
    MusicPage,
    // 其他扩展节点
    Other(String),
}
