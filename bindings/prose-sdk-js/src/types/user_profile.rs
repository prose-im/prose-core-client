// prose-core-client/prose-sdk-js
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use prose_core_client::types::Url;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct UserProfile(prose_core_client::types::UserProfile);

#[wasm_bindgen]
#[derive(Debug)]
pub struct Address(prose_core_client::types::Address);

#[wasm_bindgen]
#[derive(Clone, Default, Debug)]
pub struct Job {
    title: Option<String>,
    role: Option<String>,
    organization: Option<String>,
}

impl From<prose_core_client::types::UserProfile> for UserProfile {
    fn from(value: prose_core_client::types::UserProfile) -> Self {
        UserProfile(value)
    }
}

impl From<UserProfile> for prose_core_client::types::UserProfile {
    fn from(value: UserProfile) -> Self {
        value.0
    }
}

#[wasm_bindgen]
impl UserProfile {
    #[wasm_bindgen(constructor)]
    pub fn new() -> UserProfile {
        UserProfile(Default::default())
    }

    #[wasm_bindgen(getter, js_name = "firstName")]
    pub fn first_name(&self) -> Option<String> {
        self.0.first_name.clone()
    }

    #[wasm_bindgen(setter, js_name = "firstName")]
    pub fn set_first_name(&mut self, first_name: Option<String>) {
        self.0.first_name = first_name.clone()
    }

    #[wasm_bindgen(getter, js_name = "lastName")]
    pub fn last_name(&self) -> Option<String> {
        self.0.last_name.clone()
    }

    #[wasm_bindgen(setter, js_name = "lastName")]
    pub fn set_last_name(&mut self, last_name: Option<String>) {
        self.0.last_name = last_name.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn nickname(&self) -> Option<String> {
        self.0.nickname.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_nickname(&mut self, nickname: Option<String>) {
        self.0.nickname = nickname.clone()
    }

    #[wasm_bindgen(getter, js_name = "url")]
    pub fn url(&self) -> Option<String> {
        self.0.url.as_ref().map(|u| u.to_string())
    }

    #[wasm_bindgen(setter)]
    pub fn set_url(&mut self, url: Option<String>) {
        self.0.url = url.and_then(|u| Url::parse(u.as_ref()).ok())
    }

    #[wasm_bindgen(getter)]
    pub fn email(&self) -> Option<String> {
        self.0.email.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_email(&mut self, email: Option<String>) {
        self.0.email = email.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn phone(&self) -> Option<String> {
        self.0.tel.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_phone(&mut self, phone: Option<String>) {
        self.0.tel = phone.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn job(&self) -> Option<Job> {
        Some(Job {
            title: self.0.title.clone(),
            role: self.0.role.clone(),
            organization: self.0.org.clone(),
        })
    }

    #[wasm_bindgen(setter)]
    pub fn set_job(&mut self, job: Option<Job>) {
        let Some(job) = job.clone() else {
            self.0.title = None;
            self.0.role = None;
            self.0.org = None;
            return;
        };

        self.0.title = job.title.clone();
        self.0.role = job.role.clone();
        self.0.org = job.organization.clone();
    }

    #[wasm_bindgen(getter)]
    pub fn address(&self) -> Option<Address> {
        self.0.address.as_ref().map(|a| Address(a.clone()))
    }

    #[wasm_bindgen(setter)]
    pub fn set_address(&mut self, address: Option<Address>) {
        self.0.address = address.as_ref().map(|a| a.0.clone())
    }
}

#[wasm_bindgen]
impl Address {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Address {
        Address(Default::default())
    }

    #[wasm_bindgen(getter, js_name = "city")]
    pub fn locality(&self) -> Option<String> {
        self.0.locality.clone()
    }

    #[wasm_bindgen(setter, js_name = "city")]
    pub fn set_locality(&mut self, locality: Option<String>) {
        self.0.locality = locality.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn country(&self) -> Option<String> {
        self.0.country.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_country(&mut self, country: Option<String>) {
        self.0.country = country.clone()
    }
}

#[wasm_bindgen]
impl Job {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Job {
        Default::default()
    }

    #[wasm_bindgen(getter)]
    pub fn title(&self) -> Option<String> {
        self.title.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_title(&mut self, title: Option<String>) {
        self.title = title.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn role(&self) -> Option<String> {
        self.role.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_role(&mut self, role: Option<String>) {
        self.role = role.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn organization(&self) -> Option<String> {
        self.organization.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_organization(&mut self, organization: Option<String>) {
        self.organization = organization.clone()
    }
}
