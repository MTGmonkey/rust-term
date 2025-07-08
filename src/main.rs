//! this crate runs a terminal

#![expect(
    clippy::needless_return,
    clippy::cargo_common_metadata,
    clippy::blanket_clippy_restriction_lints,
    clippy::multiple_crate_versions,
    unused_doc_comments,
    reason = ""
)]

use rust_term::{Model, flags, init};

#[expect(clippy::undocumented_unsafe_blocks, reason = "clippy be trippin")]
fn main() -> iced::Result {
    /// SAFETY call does occur *before* the initialization of a Model
    /// SAFETY call does occur *before* any opportunity to call `print_err()`
    unsafe {
        init(flags().run());
    };
    return iced::application("test", Model::update, Model::view)
        .theme(Model::theme)
        .default_font(iced::Font::MONOSPACE)
        .decorations(false)
        .subscription(Model::subscription)
        .run();
}
