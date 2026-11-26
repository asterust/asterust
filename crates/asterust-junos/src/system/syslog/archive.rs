//! `system { syslog { archive ... } }`, also found under `file`.

use crate::statement_words;
use asterust::config::ConfigLine;

/// A syslog `archive` statement. `archive-sites`, `start-time` and `transfer-interval` are not extracted.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct SyslogArchive {
    /// Maximum number of archived files.
    pub files: Option<String>,
    /// Maximum file size, e.g. `"1m"`.
    pub size: Option<String>,
    /// `Some(true)` for `world-readable`, `Some(false)` for `no-world-readable`.
    pub world_readable: Option<bool>,
}

/// An `archive` written as a block or on one line (`archive size 1m files 5 world-readable;`).
pub(super) fn parse_archive(parent: &ConfigLine) -> Option<SyslogArchive> {
    let mut words = statement_words(parent, "archive")?.into_iter();
    let mut settings = SyslogArchive::default();
    while let Some(word) = words.next() {
        match word {
            "files" => settings.files = words.next().map(str::to_string),
            "size" => settings.size = words.next().map(str::to_string),
            "world-readable" => settings.world_readable = Some(true),
            "no-world-readable" => settings.world_readable = Some(false),
            _ => {}
        }
    }
    Some(settings)
}

#[cfg(test)]
mod tests {
    use crate::system::system;
    use asterust::config::juniper;

    #[test]
    fn extracts_syslog_archive_in_line_and_block_form() {
        let syslog = system(&juniper::parse_braces(
            r#"
system {
    syslog {
        archive {
            files 20;
            size 5m;
            world-readable;
        }
        file messages {
            any notice;
            archive size 100k files 10 no-world-readable;
        }
        file plain {
            any info;
        }
    }
}
"#,
        ))
        .unwrap()
        .syslog
        .unwrap();

        let default = syslog.archive.unwrap();
        assert_eq!(default.files.as_deref(), Some("20"));
        assert_eq!(default.size.as_deref(), Some("5m"));
        assert_eq!(default.world_readable, Some(true));

        let messages = syslog.files[0].archive.as_ref().unwrap();
        assert_eq!(messages.files.as_deref(), Some("10"));
        assert_eq!(messages.size.as_deref(), Some("100k"));
        assert_eq!(messages.world_readable, Some(false));

        assert_eq!(syslog.files[1].archive, None);
    }
}
