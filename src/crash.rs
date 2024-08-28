use std::{fs, panic::PanicInfo, path::PathBuf, time::SystemTime};

use chrono::{DateTime, Utc};
use indoc::indoc;
use serde::Serialize;

/// A crash report.
#[derive(Serialize)]
struct Report {
    captured_at: String,
    package_name: String,
    package_version: String,
    binary_name: Option<String>,
    working_dir: Option<PathBuf>,
    operating_system: String,
    panic_message: Option<String>,
    panic_location: String,
    backtrace: Option<String>,
}

impl Report {
    /// Creates a new report from a panic.
    pub fn new(panic: &PanicInfo) -> Self {
        let captured_at = DateTime::<Utc>::from(SystemTime::now()).to_rfc3339();
        const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");
        const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");
        let binary_name = std::env::args().next();
        let working_dir = std::env::current_dir().ok();
        let os = std::env::consts::OS.to_owned();

        let panic_message = panic
            .payload()
            .downcast_ref::<&str>()
            .map(|message| message.to_string());

        let panic_location = panic
            .location()
            .map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()))
            .expect("panic location should always be set");

        let backtrace = {
            let backtrace = std::backtrace::Backtrace::capture();
            use std::backtrace::BacktraceStatus;
            match backtrace.status() {
                BacktraceStatus::Captured => Some(backtrace.to_string()),
                _ => None,
            }
        };

        Self {
            captured_at,
            package_name: PACKAGE_NAME.to_owned(),
            package_version: PACKAGE_VERSION.to_owned(),
            binary_name,
            working_dir,
            operating_system: os,
            panic_message,
            panic_location,
            backtrace,
        }
    }
}

/// Initializes the crash reporter.
///
/// This installs a panic hook that will (on panic) write a crash report to file
/// and inform the user of the crash. The crash report is written to a TOML file
/// in the OS-specific temp directory with a unique id. If the report cannot be
/// written to file, it is printed to stderr instead as a last-ditch effort. The
/// message displayed to users includes information about the crash report and
/// encourages them to raise an issue on GitHub in the relevant repository.
///
/// The panic hook is only registered for release builds.
pub fn init_crash_reporter() {
    if cfg!(not(debug_assertions)) {
        std::panic::set_hook(Box::new(|panic| {
            let report = Report::new(panic);
            let content = toml::to_string_pretty(&report).expect("report should serialize to toml");
            let output_dir = std::env::temp_dir()
                .join(&report.package_name)
                .join("crash");

            let report_id = ulid::Generator::new()
                .generate()
                .expect("ulid gen should not error")
                .to_string();

            let report_path = output_dir.join(report_id).with_extension("toml");

            let result =
                fs::create_dir_all(output_dir).and_then(|_| fs::write(&report_path, &content));

            if let Err(e) = result {
                eprintln!(
                    "error: failed to save crash report to {}",
                    report_path.display()
                );
                eprintln!("{sep}\n{e}\n{sep}", sep = "-".repeat(20));
                eprintln!("error: writing crash report directly to stderr");
                eprintln!("{sep}\n{content}\n{sep}", sep = "-".repeat(20));
                return;
            }

            eprintln!(
                indoc! {
                "{} has crashed!

                A crash report has been saved to {}. To get support for this problem,
                please raise an issue on GitHub at {}/issues and include the crash
                report to help us better diagnose the problem."},
                report.package_name,
                report_path.display(),
                env!("CARGO_PKG_REPOSITORY")
            );
        }))
    }
}
