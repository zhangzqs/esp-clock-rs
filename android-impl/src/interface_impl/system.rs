use slint_app::System;

pub struct MockSystem;

impl System for MockSystem {
    fn restart(&self) {
        println!("restart");
    }

    fn get_free_heap_size(&self) -> usize {
        0
    }

    fn get_largest_free_block(&self) -> usize {
        0
    }
}
