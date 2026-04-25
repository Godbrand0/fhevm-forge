// Template registry — all embedded paths are resolved relative to this file.
// Every constant here is referenced from generator.rs via include_str!().
// This module exists as documentation of all embedded files.

pub const TEMPLATES: &[&str] = &[
    "blank",
    "erc7984",
    "lending",
    "auction",
    "voting",
];

pub fn is_valid_template(name: &str) -> bool {
    TEMPLATES.contains(&name)
}
