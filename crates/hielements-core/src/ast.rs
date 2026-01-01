//! Abstract Syntax Tree for Hielements.

use crate::span::Span;
use serde::{Deserialize, Serialize};

/// A complete Hielements program.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Program {
    /// Import statements
    pub imports: Vec<ImportStatement>,
    /// Template declarations
    pub templates: Vec<Template>,
    /// Top-level element declarations
    pub elements: Vec<Element>,
    /// Source span
    pub span: Span,
}

/// An import statement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportStatement {
    /// Import path (library name or file path)
    pub path: ImportPath,
    /// Optional alias
    pub alias: Option<Identifier>,
    /// Selective imports (for `from X import Y, Z`)
    pub selective: Vec<Identifier>,
    /// Source span
    pub span: Span,
}

/// Import path variants.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImportPath {
    /// Simple identifier path: `import python`
    Identifier(Vec<Identifier>),
    /// String path: `import './module.hie'`
    String(StringLiteral),
}

/// A template declaration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    /// Documentation comment
    pub doc_comment: Option<String>,
    /// Template name
    pub name: Identifier,
    /// Scope declarations
    pub scopes: Vec<ScopeDeclaration>,
    /// Connection point declarations
    pub connection_points: Vec<ConnectionPointDeclaration>,
    /// Check declarations
    pub checks: Vec<CheckDeclaration>,
    /// Hierarchical requirements (requires_descendant)
    pub hierarchical_requirements: Vec<HierarchicalRequirement>,
    /// Connection boundaries (allows_connection/forbids_connection)
    pub connection_boundaries: Vec<ConnectionBoundary>,
    /// Nested elements
    pub elements: Vec<Element>,
    /// Source span
    pub span: Span,
}

/// Template implementation specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateImplementation {
    /// Template name being implemented
    pub template_name: Identifier,
    /// Source span
    pub span: Span,
}

/// Template binding (e.g., `template.element.scope = expression`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateBinding {
    /// Path to the template property (e.g., ["template_name", "element_name", "scope"])
    pub path: Vec<Identifier>,
    /// Expression value to bind
    pub expression: Expression,
    /// Source span
    pub span: Span,
}

/// An element declaration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Element {
    /// Documentation comment
    pub doc_comment: Option<String>,
    /// Element name
    pub name: Identifier,
    /// Templates this element implements
    pub implements: Vec<TemplateImplementation>,
    /// Scope declarations
    pub scopes: Vec<ScopeDeclaration>,
    /// Connection point declarations
    pub connection_points: Vec<ConnectionPointDeclaration>,
    /// Check declarations
    pub checks: Vec<CheckDeclaration>,
    /// Template bindings (when implementing templates)
    pub template_bindings: Vec<TemplateBinding>,
    /// Hierarchical requirements (requires_descendant)
    pub hierarchical_requirements: Vec<HierarchicalRequirement>,
    /// Connection boundaries (allows_connection/forbids_connection)
    pub connection_boundaries: Vec<ConnectionBoundary>,
    /// Nested elements
    pub children: Vec<Element>,
    /// Source span
    pub span: Span,
}

/// A scope declaration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeDeclaration {
    /// Scope name
    pub name: Identifier,
    /// Scope expression (selector)
    pub expression: Expression,
    /// Source span
    pub span: Span,
}

/// A connection point declaration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPointDeclaration {
    /// Connection point name
    pub name: Identifier,
    /// Type annotation (mandatory)
    pub type_annotation: TypeAnnotation,
    /// Expression defining the connection point
    pub expression: Expression,
    /// Source span
    pub span: Span,
}

/// A type annotation for connection points.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeAnnotation {
    /// Type identifier (e.g., "string", "integer", "TokenStream")
    pub type_name: Identifier,
    /// Source span
    pub span: Span,
}

/// A check declaration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckDeclaration {
    /// Check expression (must be a function call)
    pub expression: Expression,
    /// Source span
    pub span: Span,
}

/// An expression.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expression {
    /// Identifier reference
    Identifier(Identifier),
    /// Member access: `a.b`
    MemberAccess {
        object: Box<Expression>,
        member: Identifier,
        span: Span,
    },
    /// Function call: `f(a, b)`
    FunctionCall {
        function: Box<Expression>,
        arguments: Vec<Expression>,
        span: Span,
    },
    /// String literal
    String(StringLiteral),
    /// Number literal
    Number(NumberLiteral),
    /// Boolean literal
    Boolean(BooleanLiteral),
    /// List literal
    List {
        elements: Vec<Expression>,
        span: Span,
    },
}

impl Expression {
    pub fn span(&self) -> Span {
        match self {
            Expression::Identifier(id) => id.span,
            Expression::MemberAccess { span, .. } => *span,
            Expression::FunctionCall { span, .. } => *span,
            Expression::String(s) => s.span,
            Expression::Number(n) => n.span,
            Expression::Boolean(b) => b.span,
            Expression::List { span, .. } => *span,
        }
    }
}

/// An identifier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identifier {
    pub name: String,
    pub span: Span,
}

impl Identifier {
    pub fn new(name: impl Into<String>, span: Span) -> Self {
        Self {
            name: name.into(),
            span,
        }
    }
}

/// A string literal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringLiteral {
    pub value: String,
    pub span: Span,
}

impl StringLiteral {
    pub fn new(value: impl Into<String>, span: Span) -> Self {
        Self {
            value: value.into(),
            span,
        }
    }
}

/// A number literal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumberLiteral {
    pub value: f64,
    pub span: Span,
}

impl NumberLiteral {
    pub fn new(value: f64, span: Span) -> Self {
        Self { value, span }
    }
}

/// A boolean literal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BooleanLiteral {
    pub value: bool,
    pub span: Span,
}

impl BooleanLiteral {
    pub fn new(value: bool, span: Span) -> Self {
        Self { value, span }
    }
}

/// A hierarchical requirement that must be satisfied by at least one descendant.
/// Used with `requires_descendant` keyword.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchicalRequirement {
    /// Kind of requirement (scope, check, or element)
    pub kind: HierarchicalRequirementKind,
    /// Source span
    pub span: Span,
}

/// Types of hierarchical requirements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HierarchicalRequirementKind {
    /// Requires a descendant with a matching scope
    Scope(ScopeDeclaration),
    /// Requires a descendant with a matching check
    Check(CheckDeclaration),
    /// Requires a descendant element with specific structure
    Element(Box<Element>),
}

/// A connection boundary constraint that applies to this element and all descendants.
/// Used with `allows_connection` and `forbids_connection` keywords.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionBoundary {
    /// Whether this allows or forbids the connection
    pub kind: ConnectionBoundaryKind,
    /// Target pattern (e.g., "api_gateway.*", "database.connection")
    pub target_pattern: ConnectionPattern,
    /// Source span
    pub span: Span,
}

/// Types of connection boundaries for architectural dependencies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionBoundaryKind {
    /// Allows connections (imports/dependencies) only to matching targets
    Allows,
    /// Forbids connections (imports/dependencies) to matching targets
    Forbids,
    /// Requires connections (imports/dependencies) to matching targets
    Requires,
}

/// A connection pattern for matching connection targets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPattern {
    /// Path components (e.g., ["api_gateway", "public_api"])
    pub path: Vec<Identifier>,
    /// Whether this is a wildcard match (ends with .*)
    pub wildcard: bool,
    /// Source span
    pub span: Span,
}
