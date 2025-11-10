use poise::serenity_prelude::utils::MessageBuilder;
use std::fmt::Display;

/// Formats an integer currency value using `.` as thousands separator
pub fn format_currency(value: i64) -> String {
    let mut number = value.abs().to_string();
    let mut formatted = String::new();

    while number.len() > 3 {
        let chunk = number.split_off(number.len() - 3);
        if formatted.is_empty() {
            formatted = chunk;
        } else {
            formatted = format!("{chunk}.{formatted}");
        }
    }

    if formatted.is_empty() {
        formatted = number;
    } else {
        formatted = format!("{number}.{formatted}");
    }

    if value < 0 {
        format!("-{formatted}")
    } else {
        formatted
    }
}

/// Builds a lightweight "emoji | message" string used across embeds/responses.
pub fn pretty_message(emoji: impl Display, message: impl Display) -> String {
    format!("{} | {}", emoji, message)
}

/// Helper functions to format Discord messages with Markdown safely.
#[allow(dead_code)]
pub mod discord {

    use super::MessageBuilder;
    use poise::serenity_prelude::UserId;

    /// Formats a raw Discord user mention (`<@id>`).
    pub fn mention(id: impl Into<i64>) -> String {
        build(|builder| {
            let user_id = UserId::new(id.into() as u64);
            builder.mention(&user_id);
        })
    }

    /// Wraps text with Discord's inline bold formatting (`**text**`).
    pub fn bold(text: impl AsRef<str>) -> String {
        build(|builder| {
            builder.push_bold_safe(text.as_ref());
        })
    }

    /// Wraps text with italics (`_text_`).
    pub fn italic(text: impl AsRef<str>) -> String {
        build(|builder| {
            builder.push_italic_safe(text.as_ref());
        })
    }

    /// Wraps text with underline markers (`__text__`).
    pub fn underline(text: impl AsRef<str>) -> String {
        build(|builder| {
            builder.push_underline_safe(text.as_ref());
        })
    }

    /// Wraps text with strikethrough markers (`~~text~~`).
    pub fn strikethrough(text: impl AsRef<str>) -> String {
        build(|builder| {
            builder.push_strike_safe(text.as_ref());
        })
    }

    /// Wraps text with spoiler markers (`||text||`).
    pub fn spoiler(text: impl AsRef<str>) -> String {
        build(|builder| {
            builder.push_spoiler_safe(text.as_ref());
        })
    }

    /// Wraps text with inline code markers (`` `text` ``).
    pub fn inline_code(text: impl AsRef<str>) -> String {
        build(|builder| {
            builder.push_mono_safe(text.as_ref());
        })
    }

    /// Formats text as a code block (```text```), optionally adding a language hint.
    pub fn code_block(text: impl AsRef<str>, language: Option<&str>) -> String {
        build(|builder| {
            builder.push_codeblock_safe(text.as_ref(), language);
        })
    }

    /// Prefixes the text with `>` so it renders as a quote.
    pub fn quote(text: impl AsRef<str>) -> String {
        build(|builder| {
            builder.push_quote_safe(text.as_ref());
        })
    }

    /// Returns a normalized version of the input without adding formatting.
    ///
    /// Useful when you just want to sanitize user content before concatenating.
    pub fn escape(text: impl AsRef<str>) -> String {
        build(|builder| {
            builder.push_safe(text.as_ref());
        })
    }

    fn build(apply: impl FnOnce(&mut MessageBuilder)) -> String {
        let mut builder = MessageBuilder::new();
        apply(&mut builder);
        builder.build()
    }
}
