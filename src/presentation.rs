pub mod log {
    use colored::*;
    use std::{
        fmt::Display,
        io::{stdout, Write},
    };

    /// Toggles coloring based on environment.
    /// For instance, colors do not work for `cmd`on Windows.
    pub fn check_support_for_colors() {
        let term = term::stdout().unwrap();
        if !term.supports_color() {
            colored::control::set_override(false);
        }
    }

    /// Print a header. Includes a preliminary newline.
    pub fn header<S: AsRef<str>>(text: S) {
        println!(
            "\n{open_brace} {text} {close_brace}",
            open_brace = "[".green(),
            text = text.as_ref(),
            close_brace = "]".green()
        );
    }

    /// Print the text without any frills.
    pub fn basic<S: AsRef<str>>(text: S) {
        println!("{}", text.as_ref());
    }

    /// Print a step.
    pub fn step<A: Display, B: Display>(process: A, text: B) {
        println!(
            "{open_paren} {process} {close_paren} {text}",
            open_paren = "(".purple(),
            process = process,
            close_paren = ")".purple(),
            text = text
        )
    }

    /// Print a prompt without a new line.
    pub fn prompt<S: AsRef<str>>(text: S) {
        print!("{}: ", text.as_ref().blue());
        stdout().flush().unwrap();
    }

    /// Print an error.
    pub fn error<S: AsRef<str>>(text: S) {
        println!("\n\t[ Error ]\n\t{}\n", text.as_ref().red());
    }
}
