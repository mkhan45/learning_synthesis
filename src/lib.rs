#![feature(io_error_other)]
#![feature(local_key_cell_methods)]
#![feature(is_some_and)]
#![feature(adt_const_params)]

use std::str::FromStr;

use wasm_bindgen::prelude::*;

pub mod bank;
pub mod egg_lang;
pub mod enumerative;
pub mod lang;
mod test;
// mod test_datagen;
pub mod vsa;

// pub mod datagen;

use enumerative::duet;
use js_sys::JsString;
use vsa::Lit;

#[wasm_bindgen]
pub fn synthesize(inps: Vec<JsString>, outs: Vec<JsString>, tests: Vec<JsString>) -> js_sys::Map {
    let obj = js_sys::Map::new();

    let inps_rs: Vec<String> = inps.iter().map(|s| s.into()).collect();
    let outs_rs: Vec<String> = outs.iter().map(|s| s.into()).collect();
    let tests_rs: Vec<String> = tests.iter().map(|s| s.into()).collect();
    let examples: Vec<_> = inps_rs
        .into_iter()
        .zip(outs_rs.into_iter())
        .map(|(inp, out)| (Lit::StringConst(inp), Lit::StringConst(out)))
        .collect();

    let synthesized = duet(&examples);

    match synthesized {
        Some(synth) => {
            let synth_str = synth.to_string();

            let mut error = false;
            let results: Vec<JsString> = tests_rs
                .iter()
                .map(|inp| match synth.eval(&Lit::StringConst(inp.to_string())) {
                    Lit::StringConst(s) => JsString::from_str(&s).unwrap(),
                    _ => {
                        error = true;
                        JsString::from_str("error").unwrap()
                    }
                })
                .collect();
            let res_arr: js_sys::Array = js_sys::Array::from_iter(results.iter());

            obj.set(
                &JsString::from_str("program").unwrap(),
                &JsString::from_str(&synth_str).unwrap(),
            )
            .set(&JsString::from_str("test_results").unwrap(), &res_arr)
            .set(
                &JsString::from_str("error").unwrap(),
                &JsValue::from_bool(error),
            )
        }
        None => obj.set(
            &JsString::from_str("error").unwrap(),
            &JsValue::from_bool(true),
        ),
    }
}
