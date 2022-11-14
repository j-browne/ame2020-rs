#![no_main]
use ame2020::Nuclide;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|input: Nuclide| {
    let _s = serde_json::to_string(&input).unwrap();
    let _s = serde_json::to_string_pretty(&input).unwrap();
});
