# Hielements Pattern Catalog

This catalog documents common software engineering patterns and their implementation in Hielements. It serves both as a reference for users and as a test suite for the language's prescriptive capabilities.

> **Note**: This documentation is automatically generated from the pattern library in the `patterns/` directory. Every pattern uses the prescriptive features of Hielements (patterns, `requires`, `forbids`, `allows`, `check`, `ref`, `uses`).

---

## Table of Contents

- [Behavioral Patterns](#behavioral-patterns)
  - [Event Driven](#event-driven)
- [Compiler/Interpreter Patterns](#compilerinterpreter-patterns)
  - [Compiler Pipeline](#compiler-pipeline)
- [Cross-Cutting Patterns](#cross-cutting-patterns)
  - [Observability](#observability)
- [Structural Patterns](#structural-patterns)
  - [Hexagonal](#hexagonal)
  - [Layered Architecture](#layered-architecture)
  - [Microservice](#microservice)

---

## Behavioral Patterns

### Event Driven

**Description**: System components communicate through events, enabling loose coupling and async processing. 

**Use Cases**:
- Real-time systems

- Systems requiring high scalability and loose coupling

**Hielements Implementation**:

```hielements
## Event-Driven Architecture Pattern
##
## Description:
## System components communicate through events, enabling loose coupling
## and async processing.
##
## Use Cases:
## - Real-time systems
## - Systems requiring high scalability and loose coupling

import files
import rust

## Event-Driven Architecture Pattern
pattern event_driven {
    ## Event Definitions
    element events {
        scope module<rust>
        ref event_types: EventType
        
        ## All events must be serializable
        check rust.derives(module, 'Serialize')
        check rust.derives(module, 'Deserialize')
    }
    
    ## Event Bus / Message Broker Integration
    element event_bus {
        scope module<rust>
        ref publish: Publisher
        ref subscribe: Subscriber
        
        check rust.function_exists(module, 'publish')
        check rust.function_exists(module, 'subscribe')
    }
    
    ## Event Producers
    element producers {
        scope module<rust>
        ref event_emitters: EventEmitter
        
        ## Producers must use the event bus
        producers.module uses event_bus
    }
    
    ## Event Consumers / Handlers
    element consumers {
        scope module<rust>
        ref event_handlers: EventHandler
        
        ## Consumers must use the event bus
        consumers.module uses event_bus
    }
    
    ## Event Store (optional but recommended for event sourcing)
    element event_store {
        scope module<rust>
        ref store: EventStore
        ref replay: ReplayFunction
        
        check rust.function_exists(module, 'store_event')
        check rust.function_exists(module, 'replay_events')
    }
    
    ## Decoupling rule - producers don't know about consumers
    check rust.no_dependency(producers.module, consumers.module)
    check rust.no_dependency(consumers.module, producers.module)
}

```

---

## Compiler/Interpreter Patterns

### Compiler Pipeline

**Description**: Classic compiler architecture with lexing, parsing, analysis, and code generation phases. 

**Use Cases**:
- Compilers

- Interpreters

- DSL implementations

**Hielements Implementation**:

```hielements
## Compiler Pipeline Pattern
##
## Description:
## Classic compiler architecture with lexing, parsing, analysis, and code
## generation phases.
##
## Use Cases:
## - Compilers
## - Interpreters
## - DSL implementations

import files
import rust

## Compiler Pipeline Pattern
pattern compiler {
    ## Lexer - Tokenization
    element lexer {
        scope module<rust>
        ref tokens: TokenStream
        ref token_types: TokenType
        
        check rust.struct_exists(module, 'Lexer')
        check rust.enum_exists(module, 'TokenKind')
        check rust.function_exists(module, 'tokenize')
    }
    
    ## Parser - Syntax Analysis
    element parser {
        scope module<rust>
        ref ast: AbstractSyntaxTree
        
        ## Parser uses lexer
        parser.module uses lexer
        
        check rust.struct_exists(module, 'Parser')
        check rust.function_exists(module, 'parse')
    }
    
    ## Semantic Analyzer
    element semantic {
        scope module<rust>
        ref type_checker: TypeChecker
        ref symbol_table: SymbolTable
        
        ## Semantic analyzer uses parser
        semantic.module uses parser
        
        check rust.function_exists(module, 'analyze')
    }
    
    ## Intermediate Representation
    element ir {
        scope module<rust>
        ref ir_types: IRNode
        ref ir_builder: IRBuilder
        
        ir.module uses semantic
    }
    
    ## Code Generator
    element codegen {
        scope module<rust>
        ref code_emitter: CodeEmitter
        
        codegen.module uses ir
        
        check rust.function_exists(module, 'generate')
    }
    
    ## Diagnostics
    element diagnostics {
        scope module<rust>
        ref error_reporter: ErrorReporter
        ref warning_reporter: WarningReporter
        
        check rust.struct_exists(module, 'Diagnostic')
    }
    
    ## Dependencies flow in one direction: lexer → parser → semantic → ir → codegen
    check rust.no_dependency(lexer.module, parser.module)
    check rust.no_dependency(parser.module, semantic.module)
    check rust.no_dependency(semantic.module, codegen.module)
}

```

---

## Cross-Cutting Patterns

### Observability

**Description**: Comprehensive visibility into system behavior through metrics, logging, and tracing (the three pillars of observability). 

**Use Cases**:
- Production systems requiring monitoring and debugging capabilities

- Systems requiring compliance and audit trails

**Hielements Implementation**:

```hielements
## Observability Pattern
##
## Description:
## Comprehensive visibility into system behavior through metrics, logging,
## and tracing (the three pillars of observability).
##
## Use Cases:
## - Production systems requiring monitoring and debugging capabilities
## - Systems requiring compliance and audit trails

import files
import rust

## Observability Pattern (Three Pillars)
pattern observability {
    ## Metrics
    element metrics {
        scope module<rust>
        ref prometheus_endpoint: MetricsEndpoint
        ref custom_metrics: MetricDefinition
        
        check rust.function_exists(module, 'record_metric')
        check rust.function_exists(module, 'expose_metrics')
    }
    
    ## Logging
    element logging {
        scope module<rust>
        ref logger: Logger
        ref log_format: LogFormat
        
        ## Structured logging required
        check rust.uses_crate(module, 'tracing')
        check rust.function_exists(module, 'log')
    }
    
    ## Distributed Tracing
    element tracing {
        scope module<rust>
        ref tracer: Tracer
        ref span_context: SpanContext
        
        check rust.function_exists(module, 'start_span')
        check rust.function_exists(module, 'inject_context')
        check rust.function_exists(module, 'extract_context')
    }
    
    ## Health Checks
    element health {
        scope module<rust>
        ref liveness: LivenessCheck
        ref readiness: ReadinessCheck
        
        check rust.function_exists(module, 'liveness_check')
        check rust.function_exists(module, 'readiness_check')
    }
    
    ## All pillars must be present
    requires descendant element metrics_impl
    requires descendant element logging_impl
    requires descendant element tracing_impl
}

```

---

## Structural Patterns

### Hexagonal

**Description**: Isolates the core domain from external concerns through ports (interfaces) and adapters (implementations). 

**Use Cases**:
- Applications requiring high testability

- Systems with multiple external integrations

**Hielements Implementation**:

```hielements
## Hexagonal Architecture (Ports & Adapters) Pattern
##
## Description:
## Isolates the core domain from external concerns through ports (interfaces)
## and adapters (implementations).
##
## Use Cases:
## - Applications requiring high testability
## - Systems with multiple external integrations

import files
import rust

## Hexagonal Architecture Pattern (Ports & Adapters)
## Core domain is isolated from external dependencies
pattern hexagonal {
    ## Core Domain - Pure business logic, no external dependencies
    element domain {
        scope module<rust>
        ref entities: Entity
        ref value_objects: ValueObject
        ref domain_services: DomainService
    }
    
    ## Ports - Interfaces defining how the domain interacts with the outside world
    element ports {
        ## Inbound ports - How the outside world talks to the domain
        element inbound {
            scope module<rust>
            ref use_cases: UseCase
        }
        
        ## Outbound ports - How the domain talks to the outside world
        element outbound {
            scope module<rust>
            ref repository_ports: RepositoryPort
            ref service_ports: ServicePort
        }
    }
    
    ## Adapters - Concrete implementations of ports
    element adapters {
        ## Primary adapters - Drive the application (REST, GraphQL, CLI)
        element primary {
            scope module<rust>
            ref http_adapter: HttpAdapter
            ref cli_adapter: CliAdapter
        }
        
        ## Secondary adapters - Driven by the application (DB, external services)
        element secondary {
            scope module<rust>
            ref db_adapter: DatabaseAdapter
            ref external_service_adapter: ServiceAdapter
        }
    }
    
    ## Dependency rules - Domain has no outward dependencies
    check rust.no_dependency(domain.module, adapters.primary.module)
    check rust.no_dependency(domain.module, adapters.secondary.module)
    
    ## Ports can only depend on domain
    check rust.depends_on(ports.inbound.module, domain.module)
    check rust.depends_on(ports.outbound.module, domain.module)
    
    ## Adapters implement ports
    check rust.implements_trait(adapters.secondary.module, ports.outbound.repository_ports)
}

## Example Implementation
element payment_service implements hexagonal {
    ## Domain bindings
    scope domain_mod<rust> binds hexagonal.domain.module = rust.module_selector('payment::domain')
    ref entities: Entity binds hexagonal.domain.entities = rust.struct_selector(domain_mod, 'Payment')
    
    ## Ports bindings
    scope inbound_mod<rust> binds hexagonal.ports.inbound.module = rust.module_selector('payment::ports::inbound')
    scope outbound_mod<rust> binds hexagonal.ports.outbound.module = rust.module_selector('payment::ports::outbound')
    
    ## Adapters bindings
    scope primary_mod<rust> binds hexagonal.adapters.primary.module = rust.module_selector('payment::adapters::http')
    scope secondary_mod<rust> binds hexagonal.adapters.secondary.module = rust.module_selector('payment::adapters::postgres')
}

```

---

### Layered Architecture

**Description**: Organizes code into horizontal layers where each layer has a specific responsibility and can only depend on layers below it. 

**Use Cases**:
- Traditional enterprise applications

- Web applications with clear separation of concerns

**Hielements Implementation**:

```hielements
## Layered Architecture (N-Tier) Pattern
## 
## Description:
## Organizes code into horizontal layers where each layer has a specific responsibility
## and can only depend on layers below it.
##
## Use Cases:
## - Traditional enterprise applications
## - Web applications with clear separation of concerns

import files
import rust

## Layered Architecture Pattern
## Enforces strict layer dependencies: presentation → business → data
pattern layered_architecture {
    ## Presentation Layer - UI and API endpoints
    element presentation {
        scope module<rust>
        ref api_endpoints: HttpHandler
    }
    
    ## Business Layer - Business logic and domain services
    element business {
        scope module<rust>
        ref domain_services: DomainService
    }
    
    ## Data Layer - Data access and persistence
    element data {
        scope module<rust>
        ref repositories: Repository
    }
    
    ## Layer dependency rules
    ## Presentation can only depend on business layer
    forbids connection to data.*
    
    ## Business layer cannot depend on presentation
    ## (enforced via check)
    check rust.no_dependency(business.module, presentation.module)
    
    ## Data layer is a leaf - no dependencies on upper layers
    check rust.no_dependency(data.module, business.module)
    check rust.no_dependency(data.module, presentation.module)
}

## Example Implementation
element my_web_app implements layered_architecture {
    scope presentation_mod<rust> binds layered_architecture.presentation.module = rust.module_selector('app::api')
    ref endpoints: HttpHandler binds layered_architecture.presentation.api_endpoints = rust.public_functions(presentation_mod)
    
    scope business_mod<rust> binds layered_architecture.business.module = rust.module_selector('app::services')
    ref services: DomainService binds layered_architecture.business.domain_services = rust.public_functions(business_mod)
    
    scope data_mod<rust> binds layered_architecture.data.module = rust.module_selector('app::repositories')
    ref repos: Repository binds layered_architecture.data.repositories = rust.struct_selector(data_mod, '*Repository')
}

```

---

### Microservice

**Description**: System composed of independently deployable services, each with its own database and API. 

**Use Cases**:
- Large-scale distributed systems

- Teams wanting independent deployments

**Hielements Implementation**:

```hielements
## Microservice Architecture Pattern
##
## Description:
## System composed of independently deployable services, each with its own
## database and API.
##
## Use Cases:
## - Large-scale distributed systems
## - Teams wanting independent deployments

import files
import rust

## Microservice Pattern
## Defines structure for a single, independently deployable service
pattern microservice {
    ## API layer - exposes service functionality
    element api {
        scope module<rust>
        ref rest_endpoint: HttpHandler
        ref api_version: string
    }
    
    ## Domain logic
    element domain {
        scope module<rust>
        ref services: DomainService
    }
    
    ## Data persistence - each service owns its data
    element persistence {
        scope module<rust>
        ref repository: Repository
        ref database_config: DbConfig
    }
    
    ## Container definition
    element container {
        scope dockerfile
        ref exposed_port: integer
    }
    
    ## Service mesh integration
    requires descendant element health_check
    requires descendant check files.exists(container.dockerfile, 'HEALTHCHECK')
    
    ## API documentation requirement
    requires descendant scope api_docs
    
    ## Service isolation - no direct database access from other services
    forbids connection to external_database.*
}

## Microservices System Pattern
## Defines a system of microservices with shared concerns
pattern microservices_system {
    ## API Gateway
    element gateway {
        scope module<rust>
        ref routing_config: RoutingConfig
        
        check rust.function_exists(module, 'route_request')
    }
    
    ## Service registry
    element registry {
        scope module<rust>
        ref service_discovery: ServiceDiscovery
    }
    
    ## At least 2 microservices required
    requires descendant element service implements microservice
    
    ## Shared configuration
    element config {
        scope config_files = files.folder_selector('config')
        check files.exists(config_files, 'services.yaml')
    }
}

## Example: E-commerce microservices
element ecommerce_platform implements microservices_system {
    scope gateway_mod<rust> binds microservices_system.gateway.module = rust.module_selector('gateway')
    scope registry_mod<rust> binds microservices_system.registry.module = rust.module_selector('registry')
    
    ## Orders Service
    element orders_service implements microservice {
        scope api_mod<rust> binds microservice.api.module = rust.module_selector('orders::api')
        scope domain_mod<rust> binds microservice.domain.module = rust.module_selector('orders::domain')
        scope persistence_mod<rust> binds microservice.persistence.module = rust.module_selector('orders::db')
        scope dockerfile binds microservice.container.dockerfile = files.file_selector('orders/Dockerfile')
        
        element health_check {
            scope module<rust> = rust.module_selector('orders::health')
            check rust.function_exists(module, 'health_check')
        }
    }
    
    ## Payments Service
    element payments_service implements microservice {
        scope api_mod<rust> binds microservice.api.module = rust.module_selector('payments::api')
        scope domain_mod<rust> binds microservice.domain.module = rust.module_selector('payments::domain')
        scope persistence_mod<rust> binds microservice.persistence.module = rust.module_selector('payments::db')
        scope dockerfile binds microservice.container.dockerfile = files.file_selector('payments/Dockerfile')
        
        element health_check {
            scope module<rust> = rust.module_selector('payments::health')
        }
    }
}

```

---

## Pattern Usage Guidelines

### When to Use Patterns

- **DO** use patterns when you have multiple components with similar structure
- **DO** use patterns to enforce architectural decisions across teams
- **DO** use patterns as documentation for expected component structure
- **DON'T** use patterns for truly unique one-off components
- **DON'T** over-engineer with patterns when a simple element suffices

### Pattern Composition

Patterns can be composed through multiple `implements`:

```hielements
## Service implementing multiple patterns
element production_service implements microservice, observability, resilience {
    ## Microservice bindings
    scope api<rust> binds microservice.api.module = rust.module_selector('service::api')
    
    ## Observability bindings  
    scope metrics<rust> binds observability.metrics.module = rust.module_selector('service::metrics')
    
    ## Resilience bindings
    scope circuit_breaker<rust> binds resilience.circuit_breaker.module = rust.module_selector('service::resilience')
}
```

### Contributing New Patterns

When adding new patterns to this catalog:

1. Create a `.hie` file in the appropriate category directory
2. Include a description comment block with the pattern's intent and use cases
3. Implement using Hielements prescriptive features (`pattern`, `requires`, `forbids`, `allows`, `check`, `ref`, `uses`)
4. Provide at least one concrete implementation example
5. Regenerate this catalog using: `python3 scripts/generate_pattern_catalog.py`

---

**Note**: This catalog is automatically generated from the Hielements pattern library. To add or modify patterns, edit the `.hie` files in the `patterns/` directory and regenerate this documentation.