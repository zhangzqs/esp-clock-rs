#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum NodeName {
    // App框架调度器
    Scheduler,
    // Http客户端
    HttpClient,
    // 天气查询器
    WeatherClient,
    // 获取时间日期
    DateTimeClient,
    // 单个操作按钮事件产生器，可产生一些按键事件
    OneButton,
    // 页面ui路由器
    Router,
    // 启动页ui与逻辑
    BootPage,
    // 首页ui与逻辑
    HomePage,
    // 菜单页ui与逻辑
    MenuPage,
    // 天气页ui与逻辑
    WeatherPage,
}