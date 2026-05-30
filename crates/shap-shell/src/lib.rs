//! `shap-shell` — shell-facing helpers (rendering + prompt-segment string).
//!
//! Intentionally the smallest crate. Holds only output rendering and the
//! prompt-segment builder shared by the binary and future shells.

pub mod render;
