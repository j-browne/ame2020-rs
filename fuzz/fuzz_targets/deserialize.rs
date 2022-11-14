#![no_main]
use ame2020::Nuclide;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _sets: Result<Vec<Nuclide>, _> = serde_json::from_reader(data);
});
