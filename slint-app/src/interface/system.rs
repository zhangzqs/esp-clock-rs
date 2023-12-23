pub trait System: Send + Sync {
    /// 重启
    fn restart(&self);

    /// 获取剩余可用堆内存，这可能比最大连续的可分配块的值还要大
    fn get_free_heap_size(&self) -> usize;

    /// 获取最大连续的可分配块
    fn get_largest_free_block(&self) -> usize;
}