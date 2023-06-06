mod test_connector;

#[cfg(feature = "test-helpers")]
pub use test_connector::{Response, TestConnection, TestConnector};

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
