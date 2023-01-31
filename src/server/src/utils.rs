pub fn capitalize_names(name: &str) -> String {
    let cap_name: String = name
        .chars()
        .enumerate()
        .map(|(i, c)| if i == 0 { c.to_ascii_uppercase() } else { c })
        .collect();
    cap_name
}

#[test]
fn test_capitalize_names() {
    assert_eq!(capitalize_names("puppy"), "Puppy".to_string());
    assert_eq!(capitalize_names("Turtle"), "Turtle".to_string())
}
