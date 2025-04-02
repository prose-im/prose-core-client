// prose-core-client/prose-sdk-js
//
// Copyright: 2025, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use prose_core_client::dtos::{
    WorkspaceIcon as SdkWorkspaceIcon, WorkspaceInfo as SdkWorkspaceInfo,
};
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
#[derive(Clone)]
pub struct WorkspaceIcon(SdkWorkspaceIcon);

#[wasm_bindgen]
pub struct WorkspaceInfo(SdkWorkspaceInfo);

#[wasm_bindgen]
impl WorkspaceIcon {
    #[wasm_bindgen(getter)]
    /// An opaque identifier to check if the contents of the `WorkspaceIcon` have changed.
    /// While `ProseClient` caches loaded icons, checking for a change in the `WorkspaceIcon` might
    /// still make sense, since `Client::loadWorkspaceIconDataURL` is asynchronous.
    pub fn id(&self) -> String {
        self.0.id.to_string()
    }
}

#[wasm_bindgen]
impl WorkspaceInfo {
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.0.name.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn icon(&self) -> Option<WorkspaceIcon> {
        self.0.icon.clone().map(WorkspaceIcon)
    }

    #[wasm_bindgen(getter, js_name = "accentColor")]
    pub fn accent_color(&self) -> Option<String> {
        self.0.accent_color.clone()
    }
}

impl From<SdkWorkspaceIcon> for WorkspaceIcon {
    fn from(value: SdkWorkspaceIcon) -> Self {
        Self(value)
    }
}

impl From<SdkWorkspaceInfo> for WorkspaceInfo {
    fn from(value: SdkWorkspaceInfo) -> Self {
        Self(value)
    }
}

impl From<WorkspaceIcon> for SdkWorkspaceIcon {
    fn from(item: WorkspaceIcon) -> Self {
        item.0
    }
}

impl From<WorkspaceInfo> for SdkWorkspaceInfo {
    fn from(item: WorkspaceInfo) -> Self {
        item.0
    }
}
