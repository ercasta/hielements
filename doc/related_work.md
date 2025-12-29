# Related Work

This document surveys tools, languages, and approaches related to Hielements. For each, we analyze key similarities and differences to help position Hielements in the broader landscape of software architecture and structure enforcement tools.

---

## 1. Architecture Description Languages (ADLs)

### 1.1 AADL (Architecture Analysis & Design Language)

**Description:** AADL is a standardized, textual and graphical language used to model the software and hardware architecture of embedded, real-time systems.

**Similarities:**
- Formal language for describing software structure
- Supports hierarchical composition of components
- Defines connections between components

**Differences:**
- AADL is domain-specific (embedded/real-time systems); Hielements is general-purpose
- AADL focuses on modeling; Hielements enforces actual code alignment
- AADL does not directly reference implementation artifacts (files, modules); Hielements binds elements to actual code via scope selectors

---

### 1.2 Acme

**Description:** Acme is an ADL designed as an interchange format for architecture description. It provides a generic, extensible core with support for defining components, connectors, and systems.

**Similarities:**
- Hierarchical component composition
- Explicit connection points between components
- Extensible via "families" (similar to Hielements libraries)

**Differences:**
- Acme is primarily a modeling/interchange language, not an enforcement tool
- No direct binding to implementation artifacts
- Acme lacks runtime or static checking against actual source code

---

### 1.3 C4 Model

**Description:** The C4 model is a lightweight approach to visualizing software architecture using four levels of abstraction: Context, Containers, Components, and Code.

**Similarities:**
- Hierarchical representation of software systems
- Focus on making software structure explicit
- Can describe relationships between elements

**Differences:**
- C4 is a documentation/visualization approach, not a formal language
- No enforcement mechanism; diagrams can become stale
- Hielements provides formal, checkable specifications; C4 does not

---

## 2. Architecture Enforcement & Fitness Function Tools

### 2.1 ArchUnit (Java)

**Description:** ArchUnit is a Java library for checking the architecture of Java applications. It allows writing unit tests that verify architectural rules.

**Similarities:**
- Enforces architectural rules at build time
- Checks dependencies, layering, and naming conventions
- Rules are defined in code (similar to Hielements rules)

**Differences:**
- ArchUnit is Java-specific; Hielements is language-agnostic
- ArchUnit rules are written in Java; Hielements has its own DSL
- ArchUnit lacks explicit hierarchical element composition
- Hielements supports cross-language/artifact elements (e.g., Python + Docker)

---

### 2.2 Dependency-Cruiser (JavaScript/TypeScript)

**Description:** Dependency-Cruiser validates and visualizes dependencies in JavaScript/TypeScript projects based on configurable rules.

**Similarities:**
- Enforces dependency rules
- Configuration-based rule definition
- Can be integrated into CI/CD pipelines

**Differences:**
- Limited to JavaScript/TypeScript ecosystem
- Focuses only on dependencies, not broader architectural semantics
- No concept of hierarchical elements or connection points

---

### 2.3 deptrac (PHP)

**Description:** deptrac is a static analysis tool for PHP that enforces architectural boundaries by defining layers and checking dependencies between them.

**Similarities:**
- Layer/module-based architecture enforcement
- YAML-based configuration (declarative)
- Static analysis approach

**Differences:**
- PHP-specific
- Focuses on layer dependencies only
- No support for multi-artifact elements or connection points

---

### 2.4 NDepend (.NET)

**Description:** NDepend is a static analysis tool for .NET that provides architecture validation, code quality metrics, and dependency analysis.

**Similarities:**
- Enforces architectural rules
- Supports custom queries and rules (CQLinq)
- Dependency validation

**Differences:**
- .NET-specific
- Heavyweight commercial tool
- Rules are queries over code metrics rather than structural element definitions
- No cross-technology element support

---

## 3. Infrastructure as Code (IaC) and Configuration Languages

### 3.1 Terraform

**Description:** Terraform is an IaC tool that uses a declarative language (HCL) to define and provision infrastructure.

**Similarities:**
- Declarative description of system components
- Supports modularity and composition
- Enforces desired state

**Differences:**
- Terraform focuses on infrastructure, not software architecture
- Does not analyze or reference application source code
- No concept of software elements like modules, functions, or classes

---

### 3.2 Pulumi

**Description:** Pulumi is an IaC tool that allows defining infrastructure using general-purpose programming languages.

**Similarities:**
- Programmatic definition of system components
- Supports hierarchical composition (components)
- Extensible via libraries

**Differences:**
- Infrastructure-focused, not software structure-focused
- Does not analyze application code
- No architectural rule enforcement for source code

---

### 3.3 CUE

**Description:** CUE is a data validation language and configuration language that unifies schema definition, data templating, and policy enforcement.

**Similarities:**
- Declarative language for defining constraints
- Supports composition and inheritance
- Can enforce policies

**Differences:**
- CUE is data/configuration-focused, not source code-focused
- No direct binding to code artifacts (files, functions, modules)
- Lacks the concept of software elements and connection points

---

## 4. Domain-Specific Languages for Structure/Contracts

### 4.1 StructurizR DSL

**Description:** Structurizr DSL is a textual language for defining software architecture models compatible with the C4 model, enabling architecture-as-code.

**Similarities:**
- Text-based, versionable architecture definitions
- Hierarchical composition of elements (workspaces, systems, containers, components)
- Explicit relationships between elements

**Differences:**
- Primarily for documentation and visualization
- No enforcement against actual source code
- Models are descriptive, not prescriptive

---

### 4.2 jQAssistant

**Description:** jQAssistant scans software artifacts and stores structural information in a Neo4j graph database, allowing queries and constraints to be defined.

**Similarities:**
- Scans actual code and artifacts
- Allows defining and enforcing architectural rules
- Supports multiple artifact types (Java, XML, etc.)

**Differences:**
- Query-based rules (Cypher) rather than a dedicated structure language
- Primarily Java/JVM-focused
- No first-class hierarchical element composition language

---

### 4.3 Design by Contract (DbC) Languages (e.g., Eiffel, JML)

**Description:** Design by Contract languages embed preconditions, postconditions, and invariants directly in code to enforce correctness.

**Similarities:**
- Formal specification of constraints
- Enforcement at compile or runtime

**Differences:**
- DbC focuses on behavioral contracts at method/class level
- No focus on architectural structure or multi-artifact elements
- Tightly coupled to specific programming languages

---

## 5. Static Analysis and Linting Tools

### 5.1 SonarQube

**Description:** SonarQube is a platform for continuous inspection of code quality, supporting multiple languages with rules for bugs, vulnerabilities, and code smells.

**Similarities:**
- Static analysis of source code
- Enforces rules and quality gates
- Supports multiple languages

**Differences:**
- Focuses on code quality metrics, not architectural structure
- No concept of hierarchical elements or explicit connections
- Rules are predefined or custom queries, not a structure language

---

### 5.2 Semgrep

**Description:** Semgrep is a static analysis tool that allows writing custom rules using a pattern-based syntax to find bugs and enforce coding standards.

**Similarities:**
- Custom rule definition
- Supports multiple languages
- Pattern-based matching on code

**Differences:**
- Focuses on code patterns, not architectural structure
- No hierarchical element composition
- No cross-artifact (e.g., code + Docker) analysis

---

## 6. Architectural Modeling and Documentation Tools

### 6.1 ArchiMate

**Description:** ArchiMate is an open and independent enterprise architecture modeling language for describing, analyzing, and visualizing architecture.

**Similarities:**
- Formal language for architectural elements
- Supports hierarchical composition and relationships
- Widely used for enterprise architecture

**Differences:**
- ArchiMate is a modeling language, not an enforcement tool
- Does not bind to actual code artifacts
- Higher abstraction level (enterprise architecture vs. software structure)

---

### 6.2 PlantUML / Mermaid

**Description:** PlantUML and Mermaid are textual diagram languages for creating UML and other diagrams from plain text.

**Similarities:**
- Text-based, versionable architecture representations
- Support for component and class diagrams

**Differences:**
- Visualization only; no enforcement
- No binding to source code
- Diagrams can become stale

---

## 7. Language Workbenches and Meta-Languages

### 7.1 JetBrains MPS

**Description:** MPS is a language workbench for creating domain-specific languages (DSLs), including projectional editors and generators.

**Similarities:**
- Enables defining custom languages for specific domains
- Supports extensibility and language composition

**Differences:**
- MPS is a meta-tool for building languages, not an architecture enforcement tool
- Requires significant investment to create a usable DSL
- No built-in focus on software structure enforcement

---

### 7.2 Xtext

**Description:** Xtext is a framework for developing programming languages and DSLs, including editor support and code generation.

**Similarities:**
- Enables creating custom languages with rich tooling
- Supports Language Server Protocol

**Differences:**
- Xtext is a language development framework, not an architecture tool
- Requires implementing language semantics from scratch
- No default focus on architectural structure

---

## 8. Policy-as-Code Tools

### 8.1 Open Policy Agent (OPA) / Rego

**Description:** OPA is a general-purpose policy engine using the Rego language for expressing policies across cloud-native stacks.

**Similarities:**
- Declarative policy definition
- Enforcement of constraints
- Extensible and embeddable

**Differences:**
- OPA focuses on runtime policies (authorization, admission control)
- Not designed for static source code structure enforcement
- No concept of software elements or hierarchical composition

---

### 8.2 Checkov

**Description:** Checkov is a static analysis tool for infrastructure-as-code, scanning Terraform, CloudFormation, Kubernetes, and more for misconfigurations.

**Similarities:**
- Static analysis and rule enforcement
- Multi-technology support

**Differences:**
- Infrastructure-focused, not application architecture-focused
- No source code analysis for software structure
- No hierarchical element model

---

## Summary Comparison Table

| Tool/Approach           | Enforcement | Multi-Language | Hierarchical Elements | Connection Points | Binds to Code |
|------------------------|-------------|----------------|-----------------------|-------------------|---------------|
| AADL                   | No          | No             | Yes                   | Yes               | No            |
| Acme                   | No          | No             | Yes                   | Yes               | No            |
| C4 Model               | No          | N/A            | Yes                   | Partial           | No            |
| ArchUnit               | Yes         | No (Java)      | Partial               | No                | Yes           |
| Dependency-Cruiser     | Yes         | No (JS/TS)     | No                    | No                | Yes           |
| deptrac                | Yes         | No (PHP)       | Partial               | No                | Yes           |
| NDepend                | Yes         | No (.NET)      | Partial               | No                | Yes           |
| Terraform              | Yes         | N/A (Infra)    | Yes                   | Partial           | No            |
| CUE                    | Yes         | N/A (Config)   | Yes                   | No                | No            |
| Structurizr DSL        | No          | N/A            | Yes                   | Yes               | No            |
| jQAssistant            | Yes         | Partial        | Partial               | No                | Yes           |
| SonarQube              | Yes         | Yes            | No                    | No                | Yes           |
| Semgrep                | Yes         | Yes            | No                    | No                | Yes           |
| ArchiMate              | No          | N/A            | Yes                   | Yes               | No            |
| OPA/Rego               | Yes         | N/A (Policy)   | No                    | No                | No            |
| **Hielements**         | **Yes**     | **Yes**        | **Yes**               | **Yes**           | **Yes**       |

---

## Key Differentiators of Hielements

1. **Cross-Technology Elements:** Hielements uniquely supports defining elements that span multiple languages and artifact types (e.g., Python modules + Dockerfiles) within a single hierarchical structure.

2. **Enforcement via Scope Selectors:** Unlike modeling languages (AADL, C4, ArchiMate), Hielements binds abstract elements to concrete code artifacts and enforces rules against actual implementations.

3. **Explicit Connection Points:** Hielements makes inter-element relationships explicit and checkable, going beyond simple dependency analysis tools.

4. **Hierarchical Composition:** Hielements supports true hierarchical nesting of elements, enabling multi-level architectural representation with inherited properties.

5. **Language-Agnostic Core:** The Hielements core provides keywords and structures, while language-specific support is implemented via extensible libraries—enabling coverage of arbitrary technologies.

6. **Design Guardrails for Greenfield and Brownfield:** Hielements serves both new development (top-down design) and legacy modernization (bottom-up discovery and enforcement), a use case not well-served by most existing tools.
