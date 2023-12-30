use log::info;
use slint_app::EvilApple;

pub struct MockEvilApple;

impl EvilApple for MockEvilApple {
    fn attack_once(&self, _data: &[u8]) {
        info!("mock attack once");
    }
}
