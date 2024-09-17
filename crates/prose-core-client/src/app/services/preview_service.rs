// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use prose_markup::MarkdownParser;
use prose_proc_macros::InjectDependencies;

#[derive(InjectDependencies)]
pub struct PreviewService {}

impl PreviewService {
    pub fn preview_markdown(&self, markdown: impl AsRef<str>) -> String {
        let parser = MarkdownParser::new(markdown.as_ref());
        parser.convert_to_html()
    }
}
