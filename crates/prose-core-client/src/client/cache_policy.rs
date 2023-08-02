#[derive(Debug, PartialEq, Copy, Clone, Default)]
pub enum CachePolicy {
    #[default]
    ReturnCacheDataElseLoad,
    ReturnCacheDataDontLoad,
    ReloadIgnoringCacheData,
}
