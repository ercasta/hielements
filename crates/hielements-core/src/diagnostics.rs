//! Diagnostic types for error reporting.

use crate::span::Span;
use serde::{Deserialize, Serialize};

/// Severity level of a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

/// A diagnostic message with source location.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    /// Severity of the diagnostic
    pub severity: DiagnosticSeverity,
    /// Error code (e.g., "E001")
    pub code: String,
    /// Human-readable message
    pub message: String,
    /// Source file path
    pub file: String,
    /// Source span
    pub span: Span,
    /// Optional context (source line)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    /// Optional help text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,
}

impl Diagnostic {
    pub fn error(code: impl Into<String>, message: impl Into<String>) -> DiagnosticBuilder {
        DiagnosticBuilder::new(DiagnosticSeverity::Error, code.into(), message.into())
    }

    pub fn warning(code: impl Into<String>, message: impl Into<String>) -> DiagnosticBuilder {
        DiagnosticBuilder::new(DiagnosticSeverity::Warning, code.into(), message.into())
    }
}

/// Builder for constructing diagnostics.
pub struct DiagnosticBuilder {
    severity: DiagnosticSeverity,
    code: String,
    message: String,
    file: Option<String>,
    span: Option<Span>,
    context: Option<String>,
    help: Option<String>,
}

impl DiagnosticBuilder {
    pub fn new(severity: DiagnosticSeverity, code: String, message: String) -> Self {
        Self {
            severity,
            code,
            message,
            file: None,
            span: None,
            context: None,
            help: None,
        }
    }

    pub fn with_file(mut self, file: impl Into<String>) -> Self {
        self.file = Some(file.into());
        self
    }

    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }

    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    pub fn build(self) -> Diagnostic {
        Diagnostic {
            severity: self.severity,
            code: self.code,
            message: self.message,
            file: self.file.unwrap_or_default(),
            span: self.span.unwrap_or_default(),
            context: self.context,
            help: self.help,
        }
    }
}

/// Collection of diagnostics.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Diagnostics {
    diagnostics: Vec<Diagnostic>,
}

impl Diagnostics {
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
        }
    }

    pub fn push(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    pub fn extend(&mut self, other: Diagnostics) {
        self.diagnostics.extend(other.diagnostics);
    }

    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == DiagnosticSeverity::Error)
    }

    pub fn errors(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == DiagnosticSeverity::Error)
    }

    pub fn warnings(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == DiagnosticSeverity::Warning)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics.iter()
    }

    pub fn into_vec(self) -> Vec<Diagnostic> {
        self.diagnostics
    }
}

impl IntoIterator for Diagnostics {
    type Item = Diagnostic;
    type IntoIter = std::vec::IntoIter<Diagnostic>;

    fn into_iter(self) -> Self::IntoIter {
        self.diagnostics.into_iter()
    }
}

/// JSON output format for diagnostics.
#[derive(Debug, Serialize, Deserialize)]
pub struct DiagnosticsOutput {
    pub version: String,
    pub status: String,
    pub errors: Vec<Diagnostic>,
    pub warnings: Vec<Diagnostic>,
    pub summary: DiagnosticsSummary,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiagnosticsSummary {
    pub total_errors: usize,
    pub total_warnings: usize,
}

impl DiagnosticsOutput {
    pub fn from_diagnostics(diagnostics: &Diagnostics) -> Self {
        let errors: Vec<_> = diagnostics.errors().cloned().collect();
        let warnings: Vec<_> = diagnostics.warnings().cloned().collect();

        Self {
            version: "1.0".to_string(),
            status: if errors.is_empty() { "ok" } else { "error" }.to_string(),
            summary: DiagnosticsSummary {
                total_errors: errors.len(),
                total_warnings: warnings.len(),
            },
            errors,
            warnings,
        }
    }
}
