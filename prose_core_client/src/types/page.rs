#[derive(PartialEq, Debug)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub is_complete: bool,
}
