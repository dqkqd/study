use unicode_segmentation::UnicodeSegmentation;

pub fn reverse(input: &str) -> String {
    UnicodeSegmentation::graphemes(input, true)
        .collect::<Vec<&str>>()
        .into_iter()
        .rev()
        .collect()
}
