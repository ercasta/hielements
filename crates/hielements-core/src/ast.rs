//! Abstract Syntax Tree for Hielements.

use crate::span::Span;
use serde::{Deserialize, Serialize};

/// A complete Hielements program.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Program {
    /// Import statements
    pub imports: Vec<ImportStatement>,
    /// Language declarations
    pub languages: Vec<LanguageDeclaration>,
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
    /// Component requirements (requires/allows/forbids [descendant] ...)
    pub component_requirements: Vec<ComponentRequirement>,
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
    /// Component requirements (requires/allows/forbids [descendant] ...)
    pub component_requirements: Vec<ComponentRequirement>,
    /// Nested elements
    pub children: Vec<Element>,
    /// Source span
    pub span: Span,
}

/// A scope declaration (V2 supports unbounded scopes and bindings).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeDeclaration {
    /// Scope name
    pub name: Identifier,
    /// Optional language type annotation using angular brackets (e.g., `scope x<python> = ...`)
    pub language: Option<Identifier>,
    /// Optional binding path for template scope binding (e.g., `binds template.element.scope`)
    pub binds: Option<Vec<Identifier>>,
    /// Scope expression (selector) - None for unbounded scopes in templates
    pub expression: Option<Expression>,
    /// Source span
    pub span: Span,
}

/// A connection point declaration (V2 supports unbounded connection points and bindings).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPointDeclaration {
    /// Connection point name
    pub name: Identifier,
    /// Type annotation (mandatory)
    pub type_annotation: TypeAnnotation,
    /// Optional binding path for template connection point binding (e.g., `binds template.element.cp`)
    pub binds: Option<Vec<Identifier>>,
    /// Expression defining the connection point - None for unbounded in templates
    pub expression: Option<Expression>,
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

// ============================================================================
// New Unified Syntax Types
// ============================================================================

/// Unified component requirement that supports the new syntax:
/// `requires [descendant] element name [: Type] [implements template]`
/// `allows [descendant] connection to pattern`
/// `forbids [descendant] connection_point name: Type`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentRequirement {
    /// The action: requires, allows, or forbids
    pub action: RequirementAction,
    /// Whether this applies to descendants (true) or immediate children (false)
    pub is_descendant: bool,
    /// The component specification
    pub component: ComponentSpec,
    /// Source span
    pub span: Span,
}

/// The action for a component requirement
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RequirementAction {
    /// Requires the component to exist
    Requires,
    /// Allows the component (whitelisting)
    Allows,
    /// Forbids the component (blacklisting)
    Forbids,
}

/// The component specification in a requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentSpec {
    /// Scope requirement: `scope name = expr`
    Scope(ScopeDeclaration),
    /// Check requirement: `check expr`
    Check(CheckDeclaration),
    /// Element requirement: `element name [: Type] [implements template]`
    Element {
        /// Element name (placeholder for reference)
        name: Identifier,
        /// Optional type annotation
        type_annotation: Option<TypeAnnotation>,
        /// Optional template implementation
        implements: Option<Identifier>,
        /// Optional element body (nested scopes, checks, etc.)
        body: Option<Box<Element>>,
    },
    /// Connection requirement: `connection to pattern`
    Connection(ConnectionPattern),
    /// Connection point requirement: `connection_point name: Type [= expr]`
    ConnectionPoint {
        /// Connection point name
        name: Identifier,
        /// Type annotation
        type_annotation: TypeAnnotation,
        /// Optional expression
        expression: Option<Expression>,
    },
    /// Language requirement: `language <name>`
    Language(Identifier),
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

// ============================================================================
// Language Declaration Types
// ============================================================================

/// A language declaration with optional connection checks.
/// Can be a simple declaration (`language python`) or include connection checks:
/// ```hielements
/// language python:
///     connection_check can_import(source: scope[], target: scope[]):
///         return python.imports_allowed(source, target)
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageDeclaration {
    /// Language name
    pub name: Identifier,
    /// Connection check definitions for this language
    pub connection_checks: Vec<ConnectionCheckDeclaration>,
    /// Source span
    pub span: Span,
}

/// A connection check declaration.
/// Defines a check that verifies connections between scopes for a specific language.
/// ```hielements
/// connection_check can_import(source: scope[], target: scope[]):
///     return python.imports_allowed(source, target)
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionCheckDeclaration {
    /// Check name
    pub name: Identifier,
    /// Parameters (all are scope[])
    pub parameters: Vec<ConnectionCheckParameter>,
    /// Expression body (the check to execute)
    pub body: Expression,
    /// Source span
    pub span: Span,
}

/// A connection check parameter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionCheckParameter {
    /// Parameter name
    pub name: Identifier,
    /// Source span
    pub span: Span,
}
