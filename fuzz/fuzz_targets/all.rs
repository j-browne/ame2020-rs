#![no_main]
use ame2020::Iter;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _nuclides = Iter::new(data).collect::<Result<Vec<_>, _>>();
});
