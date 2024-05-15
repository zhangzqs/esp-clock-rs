#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum AppName {
    // App框架调度器
    Scheduler,
    // 触摸
    TouchOneButton,
    // 天气查询器
    WeatherClient,
    // 路由器
    Router,
    // 启动页
    BootPage,
    // 首页
    HomePage,
    // 菜单页
    MenuPage,
    // 天气页
    WeatherPage,
}
