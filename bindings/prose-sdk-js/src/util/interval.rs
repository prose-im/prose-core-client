// prose-core-client/prose-sdk-js
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn setInterval(closure: &Closure<dyn FnMut()>, millis: u32) -> f64;
    fn clearInterval(token: f64);
}

#[wasm_bindgen]
pub struct Interval {
    _closure: Closure<dyn FnMut()>,
    token: f64,
}

impl Interval {
    pub fn new<F: 'static>(millis: u32, f: F) -> Interval
    where
        F: FnMut(),
    {
        // Construct a new closure.
        let closure = Closure::new(f);

        // Pass the closure to JS, to run every n milliseconds.
        let token = setInterval(&closure, millis);

        Interval {
            _closure: closure,
            token,
        }
    }
}

// When the Interval is destroyed, cancel its `setInterval` timer.
impl Drop for Interval {
    fn drop(&mut self) {
        clearInterval(self.token);
    }
}
