//! Types and function used to emit pretty diagnostics for `bindgen`.
//!
//! The entry point of this module is the [`Diagnostic`] type.

use std::fmt::Write;
use std::io::{self, BufRead, BufReader};
use std::{borrow::Cow, fs::File};

use annotate_snippets::{
    display_list::{DisplayList, FormatOptions},
    snippet::{Annotation, Slice as ExtSlice, Snippet},
};

use annotate_snippets::snippet::AnnotationType;

#[derive(Clone, Copy, Debug)]
pub(crate) enum Level {
    Error,
    Warn,
    Info,
    Note,
    Help,
}

impl From<Level> for AnnotationType {
    fn from(level: Level) -> Self {
        match level {
            Level::Error => Self::Error,
            Level::Warn => Self::Warning,
            Level::Info => Self::Info,
            Level::Note => Self::Note,
            Level::Help => Self::Help,
        }
    }
}

/// A `bindgen` diagnostic.
#[derive(Default)]
pub(crate) struct Diagnostic<'a> {
    title: Option<(Cow<'a, str>, Level)>,
    slices: Vec<Slice<'a>>,
    footer: Vec<(Cow<'a, str>, Level)>,
}

impl<'a> Diagnostic<'a> {
    /// Add a title to the diagnostic and set its type.
    pub(crate) fn with_title(
        &mut self,
        title: impl Into<Cow<'a, str>>,
        level: Level,
    ) -> &mut Self {
        self.title = Some((title.into(), level));
        self
    }

    /// Add a slice of source code to the diagnostic.
    pub(crate) fn add_slice(&mut self, slice: Slice<'a>) -> &mut Self {
        self.slices.push(slice);
        self
    }

    /// Add a footer annotation to the diagnostic. This annotation will have its own type.
    pub(crate) fn add_annotation(
        &mut self,
        msg: impl Into<Cow<'a, str>>,
        level: Level,
    ) -> &mut Self {
        self.footer.push((msg.into(), level));
        self
    }

    /// Print this diagnostic.
    ///
    /// The diagnostic is printed using `cargo:warning` if `bindgen` is being invoked by a build
    /// script or using `eprintln` otherwise.
    pub(crate) fn display(&self) {
        std::thread_local! {
            static INVOKED_BY_BUILD_SCRIPT: bool =  std::env::var_os("CARGO_CFG_TARGET_ARCH").is_some();
        }

        let mut title = None;
        let mut footer = vec![];
        let mut slices = vec![];
        if let Some((msg, level)) = &self.title {
            title = Some(Annotation {
                id: Some("bindgen"),
                label: Some(msg.as_ref()),
                annotation_type: (*level).into(),
            })
        }

        for (msg, level) in &self.footer {
            footer.push(Annotation {
                id: None,
                label: Some(msg.as_ref()),
                annotation_type: (*level).into(),
            });
        }

        // add additional info that this is generated by bindgen
        // so as to not confuse with rustc warnings
        footer.push(Annotation {
            id: None,
            label: Some("This diagnostic was generated by bindgen."),
            annotation_type: AnnotationType::Info,
        });

        for slice in &self.slices {
            if let Some(source) = &slice.source {
                slices.push(ExtSlice {
                    source: source.as_ref(),
                    line_start: slice.line.unwrap_or_default(),
                    origin: slice.filename.as_deref(),
                    annotations: vec![],
                    fold: false,
                })
            }
        }

        let snippet = Snippet {
            title,
            footer,
            slices,
            opt: FormatOptions {
                color: true,
                ..Default::default()
            },
        };
        let dl = DisplayList::from(snippet);

        if INVOKED_BY_BUILD_SCRIPT.with(Clone::clone) {
            // This is just a hack which hides the `warning:` added by cargo at the beginning of
            // every line. This should be fine as our diagnostics already have a colorful title.
            // FIXME (pvdrz): Could it be that this doesn't work in other languages?
            let hide_warning = "\r        \r";
            let string = dl.to_string();
            for line in string.lines() {
                println!("cargo:warning={}{}", hide_warning, line);
            }
        } else {
            eprintln!("{}\n", dl);
        }
    }
}

/// A slice of source code.
#[derive(Default)]
pub(crate) struct Slice<'a> {
    source: Option<Cow<'a, str>>,
    filename: Option<String>,
    line: Option<usize>,
}

impl<'a> Slice<'a> {
    /// Set the source code.
    pub(crate) fn with_source(
        &mut self,
        source: impl Into<Cow<'a, str>>,
    ) -> &mut Self {
        self.source = Some(source.into());
        self
    }

    /// Set the file, line and column.
    pub(crate) fn with_location(
        &mut self,
        mut name: String,
        line: usize,
        col: usize,
    ) -> &mut Self {
        write!(name, ":{}:{}", line, col)
            .expect("Writing to a string cannot fail");
        self.filename = Some(name);
        self.line = Some(line);
        self
    }
}

pub(crate) fn get_line(
    filename: &str,
    line: usize,
) -> io::Result<Option<String>> {
    let file = BufReader::new(File::open(filename)?);
    if let Some(line) = file.lines().nth(line.wrapping_sub(1)) {
        return line.map(Some);
    }

    Ok(None)
}