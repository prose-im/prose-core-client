// prose-core-client/prose-sdk-js
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
export interface ProseLogger {
    logDebug(message: string)
    logInfo(message: string)
    logWarn(message: string)
    logError(message: string)
}
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "ProseLogger")]
    pub type JSLogger;

    #[wasm_bindgen(method, js_name = "logDebug")]
    pub fn log_debug(this: &JSLogger, msg: &str);

    #[wasm_bindgen(method, js_name = "logInfo")]
    pub fn log_info(this: &JSLogger, msg: &str);

    #[wasm_bindgen(method, js_name = "logWarn")]
    pub fn log_warn(this: &JSLogger, msg: &str);

    #[wasm_bindgen(method, js_name = "logError")]
    pub fn log_error(this: &JSLogger, msg: &str);
}
