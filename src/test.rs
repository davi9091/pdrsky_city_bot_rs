use super::*;

#[test]
fn test_handle_text() {
    let patterns = get_patterns();

    assert_eq!(
        handle_text("", &patterns),
        String::from(""),
        "empty string returns an empty string"
    );

    assert_eq!(
        handle_text("питерский", &patterns),
        String::from("Пидорский*"),
        "lowercase pattern for city works correctly"
    );

    assert_eq!(
        handle_text("ПиТеРсКиЙ", &patterns),
        String::from("Пидорский*"),
        "regex is case-insensitive for city"
    );

    assert_eq!(
        handle_text("питерец", &patterns),
        String::from("Пидор*"),
        "pattern for singular person works"
    );

    assert_eq!(
        handle_text("питерцы", &patterns),
        String::from("Пидоры*"),
        "pattern for plural person works"
    );
}
