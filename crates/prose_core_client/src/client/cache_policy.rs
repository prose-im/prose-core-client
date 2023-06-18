#[derive(Debug, PartialEq, Copy, Clone)]
pub enum CachePolicy {
    ReturnCacheDataElseLoad,
    ReturnCacheDataDontLoad,
    ReloadIgnoringCacheData,
}

impl Default for CachePolicy {
    fn default() -> Self {
        CachePolicy::ReturnCacheDataElseLoad
    }
}
