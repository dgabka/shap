//! Output rendering for agent responses.
//!
//! Two modes (FR-003): **stream** prints text deltas live as they arrive;
//! **spinner** shows a loader while the agent works, then prints the final
//! text. The send handler drives a [`Renderer`]: it feeds chunks to
//! [`Renderer::chunk`] and prints the assembled reply with [`Renderer::finish`].

use std::io::Write;
use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};

/// Renders an agent response in stream or spinner mode.
pub struct Renderer {
    mode: Mode,
}

enum Mode {
    /// Live streaming: chunks are written to stdout as they arrive.
    Stream { wrote_any: bool },
    /// Loader then final text: a spinner runs until the reply is complete.
    Spinner { bar: ProgressBar },
}

impl Renderer {
    /// Create a renderer. `stream` selects live streaming vs. spinner-then-final.
    pub fn new(stream: bool) -> Self {
        let mode = if stream {
            Mode::Stream { wrote_any: false }
        } else {
            let bar = ProgressBar::new_spinner();
            bar.set_style(
                ProgressStyle::with_template("{spinner} {msg}")
                    .unwrap_or_else(|_| ProgressStyle::default_spinner()),
            );
            bar.set_message("thinking…");
            bar.enable_steady_tick(Duration::from_millis(100));
            Mode::Spinner { bar }
        };
        Self { mode }
    }

    /// Handle one streamed text delta.
    pub fn chunk(&mut self, text: &str) {
        if let Mode::Stream { wrote_any } = &mut self.mode {
            let mut out = std::io::stdout().lock();
            let _ = out.write_all(text.as_bytes());
            let _ = out.flush();
            *wrote_any = true;
        }
        // Spinner mode intentionally ignores chunks; the final text is printed
        // by `finish`.
    }

    /// Finish rendering, printing `full_text` in spinner mode and a trailing
    /// newline in stream mode.
    pub fn finish(self, full_text: &str) {
        match self.mode {
            Mode::Stream { wrote_any } => {
                if wrote_any {
                    println!();
                }
            }
            Mode::Spinner { bar } => {
                bar.finish_and_clear();
                println!("{}", full_text.trim_end());
            }
        }
    }
}
