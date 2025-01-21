use crate::base::calc_hash_bytes;

use speculoos::assert_that;

#[test]
fn default_hashing() {
    const PASSWORD: &str = "wasm18tnvnwkklyv4dyuj8x357n7vray4v4zur4crdj";
    const SALT: &str = "16898739935670952395686488112";

    const HASH_BYTES: &[u8; 32] = &[
        29, 244, 252, 166, 105, 232, 244, 214, 91, 151, 71, 223, 4, 50, 225, 64, 35, 214, 21, 191,
        196, 41, 144, 25, 192, 29, 99, 168, 195, 10, 205, 163,
    ];

    let hash = calc_hash_bytes(PASSWORD, SALT).unwrap();

    assert_that(&hash).is_equal_to(HASH_BYTES);
}
