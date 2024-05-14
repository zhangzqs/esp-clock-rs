#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum AppName {
    // 天气查询器
    WeatherClient,
    // App框架调度器
    Scheduler,
    // 首页
    HomePage,
    // 菜单页
    MenuPage,
    // 天气页
    WeatherPage,
}
