#[cfg(feature = "test-helpers")]
pub mod mocks;

pub trait StrExt {
    fn to_xml_result_string(&self) -> String;
}

impl StrExt for &str {
    fn to_xml_result_string(&self) -> String {
        let mut result = self.to_string();
        result.retain(|c| c != '\n' && c != '\t');
        result.replace("  ", "")
    }
}
