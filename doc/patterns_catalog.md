# Hielements Pattern Catalog

This catalog documents common software engineering patterns and their implementation in Hielements. It serves both as a reference for users and as a test suite for the language's prescriptive capabilities.

> **Note**: Every pattern in this catalog uses the prescriptive features of Hielements (patterns, `requires`, `forbids`, `allows`, `check`, `ref`, `uses`). Coverage of this catalog drives language evolution.

---

## Table of Contents

1. [Structural Patterns](#structural-patterns)
   - [Layered Architecture (N-Tier)](#layered-architecture-n-tier)
   - [Hexagonal Architecture (Ports & Adapters)](#hexagonal-architecture-ports--adapters)
   - [Clean Architecture](#clean-architecture)
   - [Microservices Architecture](#microservices-architecture)
   - [Module Structure](#module-structure)
   - [Plugin Architecture](#plugin-architecture)
2. [Behavioral Patterns](#behavioral-patterns)
   - [Event-Driven Architecture](#event-driven-architecture)
   - [Pipeline/Filter Pattern](#pipelinefilter-pattern)
   - [CQRS (Command Query Responsibility Segregation)](#cqrs-command-query-responsibility-segregation)
   - [Saga Pattern](#saga-pattern)
3. [Creational Patterns](#creational-patterns)
   - [Factory Module Pattern](#factory-module-pattern)
   - [Builder Configuration Pattern](#builder-configuration-pattern)
   - [Dependency Injection Container](#dependency-injection-container)
4. [Infrastructure Patterns](#infrastructure-patterns)
   - [Containerized Service](#containerized-service)
   - [Sidecar Pattern](#sidecar-pattern)
   - [Ambassador Pattern](#ambassador-pattern)
   - [API Gateway Pattern](#api-gateway-pattern)
5. [Cross-Cutting Patterns](#cross-cutting-patterns)
   - [Observability Pattern](#observability-pattern)
   - [Resilience Pattern](#resilience-pattern)
   - [Security Boundary Pattern](#security-boundary-pattern)
   - [Configuration Management Pattern](#configuration-management-pattern)
6. [Testing Patterns](#testing-patterns)
   - [Test Pyramid Pattern](#test-pyramid-pattern)
   - [Contract Testing Pattern](#contract-testing-pattern)
7. [Compiler/Interpreter Patterns](#compilerinterpreter-patterns)
   - [Compiler Pipeline Pattern](#compiler-pipeline-pattern)
   - [Visitor Pattern](#visitor-pattern)

---

## Structural Patterns

### Layered Architecture (N-Tier)

**Description**: Organizes code into horizontal layers where each layer has a specific responsibility and can only depend on layers below it.

**Use Cases**: Traditional enterprise applications, web applications with clear separation of concerns.

**Hielements Implementation**:

```hielements
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

### Hexagonal Architecture (Ports & Adapters)

**Description**: Isolates the core domain from external concerns through ports (interfaces) and adapters (implementations).

**Use Cases**: Applications requiring high testability, systems with multiple external integrations.

**Hielements Implementation**:

```hielements
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

### Clean Architecture

**Description**: Uncle Bob's Clean Architecture with concentric layers - entities at the center, surrounded by use cases, interface adapters, and frameworks/drivers.

**Use Cases**: Complex business applications, systems requiring long-term maintainability.

**Hielements Implementation**:

```hielements
import files
import rust

## Clean Architecture Pattern
## Dependencies point inward - outer layers depend on inner layers, never the reverse
pattern clean_architecture {
    ## Entities - Enterprise business rules
    element entities {
        scope module<rust>
        ref business_entities: Entity
    }
    
    ## Use Cases - Application business rules
    element use_cases {
        scope module<rust>
        ref interactors: Interactor
        ref input_boundaries: InputBoundary
        ref output_boundaries: OutputBoundary
    }
    
    ## Interface Adapters - Controllers, Presenters, Gateways
    element interface_adapters {
        element controllers {
            scope module<rust>
            ref request_handlers: Controller
        }
        
        element presenters {
            scope module<rust>
            ref view_models: ViewModel
        }
        
        element gateways {
            scope module<rust>
            ref data_gateways: Gateway
        }
    }
    
    ## Frameworks & Drivers - External tools, DB, UI, Web
    element frameworks {
        scope module<rust>
        ref web_framework: Framework
        ref db_driver: DatabaseDriver
    }
    
    ## The Dependency Rule - dependencies point inward
    check rust.no_dependency(entities.module, use_cases.module)
    check rust.no_dependency(entities.module, interface_adapters.controllers.module)
    check rust.no_dependency(entities.module, frameworks.module)
    
    check rust.no_dependency(use_cases.module, interface_adapters.controllers.module)
    check rust.no_dependency(use_cases.module, frameworks.module)
    
    check rust.no_dependency(interface_adapters.controllers.module, frameworks.module)
}
```

---

### Microservices Architecture

**Description**: System composed of independently deployable services, each with its own database and API.

**Use Cases**: Large-scale distributed systems, teams wanting independent deployments.

**Hielements Implementation**:

```hielements
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

### Module Structure

**Description**: Defines standard structure for a code module/package with clear public API and internal implementation.

**Use Cases**: Any codebase requiring consistent module organization.

**Hielements Implementation**:

```hielements
import files
import rust

## Module Structure Pattern
## Standard structure for a Rust module with public API and internal implementation
pattern rust_module {
    ## Public API - what the module exposes
    element public_api {
        scope module<rust>
        ref exports: Export
        
        ## Must have documentation
        check rust.has_docs(module)
    }
    
    ## Internal implementation
    element internal {
        scope module<rust>
        
        ## Internal code should not be directly accessed from outside
        forbids connection to external.*
    }
    
    ## Tests co-located with code
    element tests {
        scope module<rust>
        
        ## Tests must exist
        check rust.has_tests(module)
    }
    
    ## Required files
    requires descendant scope readme = files.file_selector('README.md')
}

## Python Module Pattern
pattern python_module {
    ## Public API via __init__.py
    element public_api {
        scope module<python>
        ref exports: Export
        
        check files.exists(module, '__init__.py')
    }
    
    ## Internal implementation (underscore prefix convention)
    element internal {
        scope module<python>
        
        ## Check naming convention for internal modules
        check python.matches_naming_convention(module, '_*.py')
    }
    
    ## Test module
    element tests {
        scope module<python>
        
        check python.has_tests(module)
    }
}
```

---

### Plugin Architecture

**Description**: Extensible system where functionality can be added through plugins without modifying the core.

**Use Cases**: IDEs, build tools, applications requiring extensibility.

**Hielements Implementation**:

```hielements
import files
import rust

## Plugin Architecture Pattern
## Core system with extension points for plugins
pattern plugin_architecture {
    ## Plugin Interface - what plugins must implement
    element plugin_interface {
        scope module<rust>
        ref plugin_trait: Trait
        ref lifecycle_hooks: LifecycleHook
        
        ## Plugin interface must be stable
        check rust.trait_exists(module, 'Plugin')
        check rust.trait_exists(module, 'PluginLifecycle')
    }
    
    ## Plugin Registry - manages plugin discovery and loading
    element registry {
        scope module<rust>
        ref register_function: Function
        ref discovery: PluginDiscovery
        
        check rust.function_exists(module, 'register_plugin')
        check rust.function_exists(module, 'discover_plugins')
    }
    
    ## Plugin Host - executes plugins safely
    element host {
        scope module<rust>
        ref execution_context: ExecutionContext
        ref sandbox: Sandbox
        
        ## Plugins must be sandboxed
        check rust.struct_exists(module, 'PluginSandbox')
    }
    
    ## Core functionality - independent of plugins
    element core {
        scope module<rust>
        
        ## Core must not depend on specific plugins
        check rust.no_dependency(module, plugins.*)
    }
    
    ## Extension points - where plugins can hook in
    requires descendant element extension_point
}

## Example: Build Tool with Plugins
element build_tool implements plugin_architecture {
    scope plugin_interface_mod<rust> binds plugin_architecture.plugin_interface.module = rust.module_selector('build::plugins::interface')
    scope registry_mod<rust> binds plugin_architecture.registry.module = rust.module_selector('build::plugins::registry')
    scope host_mod<rust> binds plugin_architecture.host.module = rust.module_selector('build::plugins::host')
    scope core_mod<rust> binds plugin_architecture.core.module = rust.module_selector('build::core')
    
    element extension_point {
        scope module<rust> = rust.module_selector('build::extensions')
        ref before_build: Hook = rust.function_selector(module, 'before_build')
        ref after_build: Hook = rust.function_selector(module, 'after_build')
    }
}
```

---

## Behavioral Patterns

### Event-Driven Architecture

**Description**: System components communicate through events, enabling loose coupling and async processing.

**Use Cases**: Real-time systems, systems requiring high scalability and loose coupling.

**Hielements Implementation**:

```hielements
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

### Pipeline/Filter Pattern

**Description**: Data flows through a series of processing stages (filters), each performing a specific transformation.

**Use Cases**: Data processing systems, compiler pipelines, ETL processes.

**Hielements Implementation**:

```hielements
import files
import rust

## Pipeline/Filter Pattern
## Data flows through sequential processing stages
pattern pipeline {
    ## Pipeline Stage Interface
    element stage_interface {
        scope module<rust>
        ref stage_trait: Trait
        
        check rust.trait_exists(module, 'PipelineStage')
    }
    
    ## Pipeline Orchestrator
    element orchestrator {
        scope module<rust>
        ref pipeline_builder: PipelineBuilder
        ref execution_engine: ExecutionEngine
        
        check rust.struct_exists(module, 'Pipeline')
        check rust.function_exists(module, 'execute')
    }
    
    ## Input Stage
    element input {
        scope module<rust>
        ref source: DataSource
        
        input.module uses stage_interface
    }
    
    ## Processing Stages
    requires descendant element processing_stage {
        scope module<rust>
        ref transform: TransformFunction
        
        processing_stage.module uses stage_interface
    }
    
    ## Output Stage
    element output {
        scope module<rust>
        ref sink: DataSink
        
        output.module uses stage_interface
    }
    
    ## Data flows in one direction
    check rust.no_dependency(input.module, output.module)
}

## Example: Compiler Pipeline
element compiler_pipeline implements pipeline {
    scope stage_interface_mod<rust> binds pipeline.stage_interface.module = rust.module_selector('compiler::pipeline')
    scope orchestrator_mod<rust> binds pipeline.orchestrator.module = rust.module_selector('compiler::orchestrator')
    scope input_mod<rust> binds pipeline.input.module = rust.module_selector('compiler::source')
    scope output_mod<rust> binds pipeline.output.module = rust.module_selector('compiler::output')
    
    ## Lexer Stage
    element lexer_stage {
        scope module<rust> = rust.module_selector('compiler::lexer')
        ref transform: TransformFunction = rust.function_selector(module, 'tokenize')
        
        module uses stage_interface_mod
    }
    
    ## Parser Stage
    element parser_stage {
        scope module<rust> = rust.module_selector('compiler::parser')
        ref transform: TransformFunction = rust.function_selector(module, 'parse')
        
        module uses stage_interface_mod
    }
    
    ## Semantic Analysis Stage
    element semantic_stage {
        scope module<rust> = rust.module_selector('compiler::semantic')
        ref transform: TransformFunction = rust.function_selector(module, 'analyze')
        
        module uses stage_interface_mod
    }
    
    ## Code Generation Stage
    element codegen_stage {
        scope module<rust> = rust.module_selector('compiler::codegen')
        ref transform: TransformFunction = rust.function_selector(module, 'generate')
        
        module uses stage_interface_mod
    }
}
```

---

### CQRS (Command Query Responsibility Segregation)

**Description**: Separates read and write operations into different models, allowing independent optimization.

**Use Cases**: High-performance systems, complex domain models, event-sourced systems.

**Hielements Implementation**:

```hielements
import files
import rust

## CQRS Pattern
## Separates command (write) and query (read) responsibilities
pattern cqrs {
    ## Commands - Write operations
    element commands {
        scope module<rust>
        ref command_handlers: CommandHandler
        ref command_types: Command
        
        check rust.trait_exists(module, 'Command')
        check rust.trait_exists(module, 'CommandHandler')
    }
    
    ## Queries - Read operations
    element queries {
        scope module<rust>
        ref query_handlers: QueryHandler
        ref query_types: Query
        
        check rust.trait_exists(module, 'Query')
        check rust.trait_exists(module, 'QueryHandler')
    }
    
    ## Write Model - Optimized for writes
    element write_model {
        scope module<rust>
        ref aggregates: Aggregate
        ref write_repository: WriteRepository
    }
    
    ## Read Model - Optimized for reads (denormalized)
    element read_model {
        scope module<rust>
        ref projections: Projection
        ref read_repository: ReadRepository
    }
    
    ## Synchronization between models
    element synchronizer {
        scope module<rust>
        ref event_projector: EventProjector
        
        check rust.function_exists(module, 'project_event')
    }
    
    ## Strict separation - commands don't return domain data
    check rust.no_dependency(queries.module, commands.module)
    
    ## Read model is eventually consistent with write model
    check rust.depends_on(synchronizer.module, write_model.module)
}
```

---

### Saga Pattern

**Description**: Manages distributed transactions through a sequence of local transactions with compensating actions.

**Use Cases**: Microservices requiring cross-service transactions, long-running business processes.

**Hielements Implementation**:

```hielements
import files
import rust

## Saga Pattern
## Orchestrates distributed transactions with compensation
pattern saga {
    ## Saga Definition
    element saga_definition {
        scope module<rust>
        ref saga_steps: SagaStep
        ref compensation_steps: CompensationStep
        
        check rust.trait_exists(module, 'Saga')
        check rust.trait_exists(module, 'SagaStep')
    }
    
    ## Saga Orchestrator
    element orchestrator {
        scope module<rust>
        ref execute_saga: Function
        ref rollback: Function
        
        check rust.function_exists(module, 'execute')
        check rust.function_exists(module, 'compensate')
    }
    
    ## Saga State Store
    element state_store {
        scope module<rust>
        ref saga_state: SagaState
        ref state_transitions: StateTransition
        
        check rust.function_exists(module, 'save_state')
        check rust.function_exists(module, 'get_state')
    }
    
    ## Each step must have a compensation
    requires descendant element step {
        scope module<rust>
        ref forward_action: Action
        ref compensation_action: Action
    }
}

## Example: Order Saga
element order_saga implements saga {
    scope saga_def_mod<rust> binds saga.saga_definition.module = rust.module_selector('saga::order')
    scope orchestrator_mod<rust> binds saga.orchestrator.module = rust.module_selector('saga::orchestrator')
    scope state_store_mod<rust> binds saga.state_store.module = rust.module_selector('saga::state')
    
    ## Create Order Step
    element create_order_step {
        scope module<rust> = rust.module_selector('saga::steps::create_order')
        ref forward_action: Action = rust.function_selector(module, 'create_order')
        ref compensation_action: Action = rust.function_selector(module, 'cancel_order')
    }
    
    ## Reserve Inventory Step
    element reserve_inventory_step {
        scope module<rust> = rust.module_selector('saga::steps::inventory')
        ref forward_action: Action = rust.function_selector(module, 'reserve')
        ref compensation_action: Action = rust.function_selector(module, 'release')
    }
    
    ## Process Payment Step
    element payment_step {
        scope module<rust> = rust.module_selector('saga::steps::payment')
        ref forward_action: Action = rust.function_selector(module, 'charge')
        ref compensation_action: Action = rust.function_selector(module, 'refund')
    }
}
```

---

## Creational Patterns

### Factory Module Pattern

**Description**: A module responsible for creating instances of related types.

**Use Cases**: Complex object creation, managing dependencies, test fixtures.

**Hielements Implementation**:

```hielements
import files
import rust

## Factory Module Pattern
pattern factory {
    ## Factory Interface
    element factory_interface {
        scope module<rust>
        ref factory_trait: Trait
        
        check rust.trait_exists(module, 'Factory')
    }
    
    ## Concrete Factory
    element concrete_factory {
        scope module<rust>
        ref create_methods: FactoryMethod
        
        ## Must implement factory interface
        check rust.implements_trait(module, factory_interface.factory_trait)
    }
    
    ## Product Interface
    element product_interface {
        scope module<rust>
        ref product_trait: Trait
    }
    
    ## Factory creates products, not concrete types
    check rust.returns_trait(concrete_factory.create_methods, product_interface.product_trait)
}

## Example: Database Connection Factory
element db_factory implements factory {
    scope factory_interface_mod<rust> binds factory.factory_interface.module = rust.module_selector('db::factory')
    scope concrete_factory_mod<rust> binds factory.concrete_factory.module = rust.module_selector('db::postgres_factory')
    scope product_interface_mod<rust> binds factory.product_interface.module = rust.module_selector('db::connection')
    
    check rust.function_exists(concrete_factory_mod, 'create_connection')
    check rust.function_exists(concrete_factory_mod, 'create_pool')
}
```

---

### Builder Configuration Pattern

**Description**: Uses the builder pattern for complex configuration with validation.

**Use Cases**: Application configuration, complex object construction with validation.

**Hielements Implementation**:

```hielements
import files
import rust

## Builder Configuration Pattern
pattern config_builder {
    ## Configuration Definition
    element config {
        scope module<rust>
        ref config_struct: ConfigStruct
        
        ## Config must be serializable for file-based config
        check rust.derives(module, 'Serialize')
        check rust.derives(module, 'Deserialize')
    }
    
    ## Builder
    element builder {
        scope module<rust>
        ref builder_struct: BuilderStruct
        ref build_method: BuildMethod
        
        check rust.struct_exists(module, 'ConfigBuilder')
        check rust.function_exists(module, 'build')
    }
    
    ## Validation
    element validation {
        scope module<rust>
        ref validate_method: ValidateMethod
        
        check rust.function_exists(module, 'validate')
    }
    
    ## Default Values
    element defaults {
        scope module<rust>
        ref default_impl: DefaultImpl
        
        check rust.implements_trait(config.module, 'Default')
    }
}
```

---

### Dependency Injection Container

**Description**: Manages object creation and dependency resolution.

**Use Cases**: Applications with complex dependency graphs, facilitating testing through dependency substitution.

**Hielements Implementation**:

```hielements
import files
import rust

## Dependency Injection Container Pattern
pattern di_container {
    ## Service Registration
    element registration {
        scope module<rust>
        ref register_service: RegisterFunction
        ref service_descriptor: ServiceDescriptor
        
        check rust.function_exists(module, 'register')
        check rust.function_exists(module, 'register_singleton')
        check rust.function_exists(module, 'register_scoped')
    }
    
    ## Service Resolution
    element resolution {
        scope module<rust>
        ref resolve_service: ResolveFunction
        
        check rust.function_exists(module, 'resolve')
        check rust.function_exists(module, 'try_resolve')
    }
    
    ## Lifetime Management
    element lifetime {
        scope module<rust>
        ref lifetime_types: LifetimeType
        
        check rust.enum_exists(module, 'ServiceLifetime')
    }
    
    ## Container itself
    element container {
        scope module<rust>
        ref container_struct: Container
        
        check rust.struct_exists(module, 'ServiceContainer')
    }
}
```

---

## Infrastructure Patterns

### Containerized Service

**Description**: Service packaged as a container with all necessary infrastructure concerns.

**Use Cases**: Any production service deployment.

**Hielements Implementation**:

```hielements
import files
import rust

## Containerized Service Pattern
pattern containerized_service {
    ## Application Code
    element application {
        scope module<rust>
        ref entrypoint: Entrypoint
    }
    
    ## Dockerfile
    element dockerfile {
        scope dockerfile = files.file_selector('Dockerfile')
        ref exposed_port: integer
        
        ## Security: Don't run as root
        check files.contains(dockerfile, 'USER')
        
        ## Health check required
        check files.contains(dockerfile, 'HEALTHCHECK')
    }
    
    ## Docker Compose for local development
    element compose {
        scope compose_file = files.file_selector('docker-compose.yml')
        
        check files.exists(compose_file, 'docker-compose.yml')
    }
    
    ## Configuration via environment variables
    element config {
        scope env_file = files.file_selector('.env.example')
        
        check files.exists(env_file, '.env.example')
    }
    
    ## Required documentation
    requires descendant scope readme = files.file_selector('README.md')
}
```

---

### Sidecar Pattern

**Description**: Auxiliary container running alongside the main application container, providing supporting features.

**Use Cases**: Service mesh proxies, logging agents, configuration management.

**Hielements Implementation**:

```hielements
import files
import rust

## Sidecar Pattern
## Auxiliary container providing cross-cutting concerns
pattern sidecar {
    ## Main Application Container
    element main_container {
        scope dockerfile = files.file_selector('Dockerfile')
        ref application_port: integer
    }
    
    ## Sidecar Container
    element sidecar_container {
        scope dockerfile = files.file_selector('Dockerfile.sidecar')
        ref sidecar_port: integer
    }
    
    ## Shared Volume for communication
    element shared_volume {
        scope compose_file = files.file_selector('docker-compose.yml')
        ref volume_mount: VolumePath
    }
    
    ## Pod/Deployment Definition
    element deployment {
        scope k8s_manifest = files.file_selector('k8s/deployment.yaml')
        
        ## Both containers must be in same pod
        check files.contains(k8s_manifest, 'containers:')
    }
    
    ## Sidecar lifecycle tied to main container
    check files.contains(deployment.k8s_manifest, 'shareProcessNamespace')
}

## Example: Service Mesh Sidecar
element service_with_proxy implements sidecar {
    scope main_dockerfile binds sidecar.main_container.dockerfile = files.file_selector('Dockerfile')
    scope sidecar_dockerfile binds sidecar.sidecar_container.dockerfile = files.file_selector('Dockerfile.envoy')
    scope compose binds sidecar.shared_volume.compose_file = files.file_selector('docker-compose.yml')
    scope k8s binds sidecar.deployment.k8s_manifest = files.file_selector('k8s/deployment.yaml')
}
```

---

### Ambassador Pattern

**Description**: A helper service that handles network-related tasks like monitoring, logging, or routing.

**Use Cases**: Legacy application modernization, cross-cutting network concerns.

**Hielements Implementation**:

```hielements
import files
import rust

## Ambassador Pattern
## Proxy that handles network concerns for an application
pattern ambassador {
    ## Application
    element application {
        scope module<rust>
        ref service: Service
    }
    
    ## Ambassador Proxy
    element ambassador_proxy {
        scope module<rust>
        ref routing: RoutingConfig
        ref retry_policy: RetryPolicy
        ref circuit_breaker: CircuitBreakerConfig
    }
    
    ## Ambassador Configuration
    element config {
        scope config_file = files.file_selector('ambassador.yaml')
        
        check files.exists(config_file, 'ambassador.yaml')
    }
    
    ## Application talks to ambassador, not directly to external services
    forbids connection to external_service.*
    allows connection to ambassador_proxy.*
}
```

---

### API Gateway Pattern

**Description**: Single entry point for all client requests, handling cross-cutting concerns.

**Use Cases**: Microservices architectures, API management.

**Hielements Implementation**:

```hielements
import files
import rust

## API Gateway Pattern
pattern api_gateway {
    ## Gateway Core
    element gateway {
        scope module<rust>
        ref routing: RouterConfig
        ref middleware_chain: MiddlewareChain
        
        check rust.function_exists(module, 'route')
        check rust.function_exists(module, 'apply_middleware')
    }
    
    ## Authentication/Authorization
    element auth {
        scope module<rust>
        ref auth_handler: AuthHandler
        ref token_validation: TokenValidator
        
        check rust.function_exists(module, 'authenticate')
        check rust.function_exists(module, 'authorize')
    }
    
    ## Rate Limiting
    element rate_limiting {
        scope module<rust>
        ref rate_limiter: RateLimiter
        
        check rust.function_exists(module, 'check_rate_limit')
    }
    
    ## Request/Response Transformation
    element transformation {
        scope module<rust>
        ref request_transformer: Transformer
        ref response_transformer: Transformer
    }
    
    ## API Documentation
    element api_docs {
        scope docs_file = files.file_selector('api/openapi.yaml')
        
        check files.exists(docs_file, 'openapi.yaml')
    }
    
    ## Logging and Monitoring
    requires descendant element observability implements observability_pattern
}
```

---

## Cross-Cutting Patterns

### Observability Pattern

**Description**: Comprehensive visibility into system behavior through metrics, logging, and tracing.

**Use Cases**: Production systems requiring monitoring and debugging capabilities.

**Hielements Implementation**:

```hielements
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

### Resilience Pattern

**Description**: Patterns for building systems that can handle failures gracefully.

**Use Cases**: Distributed systems, services with external dependencies.

**Hielements Implementation**:

```hielements
import files
import rust

## Resilience Pattern
pattern resilience {
    ## Circuit Breaker
    element circuit_breaker {
        scope module<rust>
        ref breaker: CircuitBreaker
        ref state_machine: BreakerState
        
        check rust.struct_exists(module, 'CircuitBreaker')
        check rust.enum_exists(module, 'CircuitState')
    }
    
    ## Retry Policy
    element retry {
        scope module<rust>
        ref retry_policy: RetryPolicy
        ref backoff_strategy: BackoffStrategy
        
        check rust.function_exists(module, 'retry')
        check rust.function_exists(module, 'with_backoff')
    }
    
    ## Timeout
    element timeout {
        scope module<rust>
        ref timeout_config: TimeoutConfig
        
        check rust.function_exists(module, 'with_timeout')
    }
    
    ## Bulkhead (Isolation)
    element bulkhead {
        scope module<rust>
        ref bulkhead_config: BulkheadConfig
        ref semaphore: Semaphore
        
        check rust.struct_exists(module, 'Bulkhead')
    }
    
    ## Fallback
    element fallback {
        scope module<rust>
        ref fallback_handler: FallbackHandler
        
        check rust.function_exists(module, 'with_fallback')
    }
}
```

---

### Security Boundary Pattern

**Description**: Defines security perimeters with access control and isolation.

**Use Cases**: Multi-tenant systems, systems with sensitive data.

**Hielements Implementation**:

```hielements
import files
import rust

## Security Boundary Pattern
pattern security_boundary {
    ## Trust Zone Definition
    element trust_zone {
        scope module<rust>
        ref zone_policy: SecurityPolicy
    }
    
    ## Authentication
    element authentication {
        scope module<rust>
        ref authenticator: Authenticator
        ref credential_validator: CredentialValidator
        
        check rust.function_exists(module, 'authenticate')
    }
    
    ## Authorization
    element authorization {
        scope module<rust>
        ref authorizer: Authorizer
        ref permission_checker: PermissionChecker
        
        check rust.function_exists(module, 'authorize')
        check rust.function_exists(module, 'check_permission')
    }
    
    ## Data Protection
    element data_protection {
        scope module<rust>
        ref encryption: Encryption
        ref key_management: KeyManager
        
        check rust.function_exists(module, 'encrypt')
        check rust.function_exists(module, 'decrypt')
    }
    
    ## Audit Logging
    element audit {
        scope module<rust>
        ref audit_logger: AuditLogger
        
        check rust.function_exists(module, 'log_access')
        check rust.function_exists(module, 'log_modification')
    }
    
    ## Security constraints
    forbids connection to untrusted_zone.*
    requires connection to audit.audit_logger
}
```

---

### Configuration Management Pattern

**Description**: Centralized configuration with validation, defaults, and environment-specific overrides.

**Use Cases**: Any production application requiring configurable behavior.

**Hielements Implementation**:

```hielements
import files
import rust

## Configuration Management Pattern
pattern config_management {
    ## Configuration Schema
    element schema {
        scope module<rust>
        ref config_struct: ConfigStruct
        
        ## Must be deserializable from multiple formats
        check rust.derives(module, 'Deserialize')
    }
    
    ## Configuration Sources
    element sources {
        ## File-based config
        element file_source {
            scope config_dir = files.folder_selector('config')
            
            check files.exists(config_dir, 'default.yaml')
        }
        
        ## Environment variables
        element env_source {
            scope module<rust>
            ref env_parser: EnvParser
            
            check rust.function_exists(module, 'from_env')
        }
        
        ## Remote config (optional)
        element remote_source {
            scope module<rust>
            ref remote_client: RemoteConfigClient
        }
    }
    
    ## Configuration Validation
    element validation {
        scope module<rust>
        ref validator: ConfigValidator
        
        check rust.function_exists(module, 'validate')
    }
    
    ## Hot Reload Support
    element hot_reload {
        scope module<rust>
        ref watcher: ConfigWatcher
        ref reload_handler: ReloadHandler
        
        check rust.function_exists(module, 'watch')
        check rust.function_exists(module, 'reload')
    }
    
    ## Environment-specific configs must exist
    check files.exists(sources.file_source.config_dir, 'production.yaml')
    check files.exists(sources.file_source.config_dir, 'development.yaml')
}
```

---

## Testing Patterns

### Test Pyramid Pattern

**Description**: Testing strategy with many unit tests, fewer integration tests, and even fewer E2E tests.

**Use Cases**: Any project requiring comprehensive test coverage.

**Hielements Implementation**:

```hielements
import files
import rust

## Test Pyramid Pattern
pattern test_pyramid {
    ## Unit Tests (Many, Fast, Isolated)
    element unit_tests {
        scope module<rust> = rust.module_selector('tests::unit')
        
        check rust.has_tests(module)
        ## Unit tests should not have external dependencies
        check rust.no_dependency(module, 'reqwest')
        check rust.no_dependency(module, 'tokio-postgres')
    }
    
    ## Integration Tests (Some, Medium Speed, Test Module Integration)
    element integration_tests {
        scope module<rust> = rust.module_selector('tests::integration')
        
        check rust.has_tests(module)
    }
    
    ## End-to-End Tests (Few, Slow, Full System)
    element e2e_tests {
        scope module<rust> = rust.module_selector('tests::e2e')
        
        check rust.has_tests(module)
    }
    
    ## Test Utilities
    element test_utils {
        scope module<rust> = rust.module_selector('tests::utils')
        
        ref fixtures: TestFixture
        ref mocks: Mock
    }
    
    ## CI Configuration
    element ci {
        scope ci_config = files.file_selector('.github/workflows/test.yml')
        
        check files.exists(ci_config, 'test.yml')
    }
}
```

---

### Contract Testing Pattern

**Description**: Tests that verify the contract between service provider and consumer.

**Use Cases**: Microservices communication, API compatibility.

**Hielements Implementation**:

```hielements
import files
import rust

## Contract Testing Pattern
pattern contract_testing {
    ## Contract Definition
    element contracts {
        scope contracts_dir = files.folder_selector('contracts')
        ref provider_contracts: ProviderContract
        ref consumer_contracts: ConsumerContract
    }
    
    ## Provider Tests
    element provider_tests {
        scope module<rust>
        ref provider_verification: ProviderVerification
        
        check rust.function_exists(module, 'verify_provider')
    }
    
    ## Consumer Tests
    element consumer_tests {
        scope module<rust>
        ref consumer_verification: ConsumerVerification
        
        check rust.function_exists(module, 'verify_consumer')
    }
    
    ## Contract Broker (for sharing contracts)
    element broker {
        scope config = files.file_selector('pact-broker.json')
        
        check files.exists(config, 'pact-broker.json')
    }
}
```

---

## Compiler/Interpreter Patterns

### Compiler Pipeline Pattern

**Description**: Classic compiler architecture with lexing, parsing, analysis, and code generation phases.

**Use Cases**: Compilers, interpreters, DSL implementations.

**Hielements Implementation**:

```hielements
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

### Visitor Pattern

**Description**: Separate algorithms from the data structures they operate on, enabling easy addition of new operations.

**Use Cases**: AST traversal, code transformation, analysis passes.

**Hielements Implementation**:

```hielements
import files
import rust

## Visitor Pattern
pattern visitor {
    ## Visitable Node Hierarchy
    element nodes {
        scope module<rust>
        ref node_trait: NodeTrait
        ref accept_method: AcceptMethod
        
        check rust.trait_exists(module, 'Node')
        check rust.method_exists(module, 'accept')
    }
    
    ## Visitor Interface
    element visitor_interface {
        scope module<rust>
        ref visitor_trait: VisitorTrait
        
        check rust.trait_exists(module, 'Visitor')
        ## Should have visit method for each node type
    }
    
    ## Concrete Visitors
    requires descendant element concrete_visitor {
        scope module<rust>
        ref visitor_impl: VisitorImpl
        
        check rust.implements_trait(module, visitor_interface.visitor_trait)
    }
    
    ## Visitor infrastructure enables adding new operations
    ## without modifying node types
    check rust.no_dependency(nodes.module, concrete_visitor.module)
}

## Example: AST Visitor
element ast_visitor implements visitor {
    scope nodes_mod<rust> binds visitor.nodes.module = rust.module_selector('ast::nodes')
    scope visitor_mod<rust> binds visitor.visitor_interface.module = rust.module_selector('ast::visitor')
    
    ## Type Checker Visitor
    element type_checker_visitor {
        scope module<rust> = rust.module_selector('ast::passes::type_check')
        ref visitor_impl: VisitorImpl = rust.struct_selector(module, 'TypeChecker')
        
        check rust.implements_trait(module, visitor_mod)
    }
    
    ## Optimizer Visitor
    element optimizer_visitor {
        scope module<rust> = rust.module_selector('ast::passes::optimize')
        ref visitor_impl: VisitorImpl = rust.struct_selector(module, 'Optimizer')
        
        check rust.implements_trait(module, visitor_mod)
    }
    
    ## Code Generator Visitor
    element codegen_visitor {
        scope module<rust> = rust.module_selector('ast::passes::codegen')
        ref visitor_impl: VisitorImpl = rust.struct_selector(module, 'CodeGenerator')
        
        check rust.implements_trait(module, visitor_mod)
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

1. Describe the pattern's intent and use cases
2. Implement using Hielements prescriptive features (`pattern`, `requires`, `forbids`, `allows`, `check`, `ref`, `uses`)
3. Provide at least one concrete implementation example
4. Document any constraints or limitations

---

## Appendix: Pattern Quick Reference

| Pattern | Category | Key Features |
|---------|----------|--------------|
| Layered Architecture | Structural | Strict layer dependencies, no skip-level calls |
| Hexagonal | Structural | Ports/adapters, domain isolation |
| Clean Architecture | Structural | Dependency rule, concentric layers |
| Microservice | Structural | Independent deployment, data ownership |
| Plugin Architecture | Structural | Extension points, sandboxing |
| Event-Driven | Behavioral | Loose coupling, async processing |
| Pipeline | Behavioral | Sequential stages, unidirectional flow |
| CQRS | Behavioral | Read/write separation, eventual consistency |
| Saga | Behavioral | Distributed transactions, compensation |
| Factory | Creational | Object creation abstraction |
| DI Container | Creational | Dependency resolution, lifetime management |
| Containerized Service | Infrastructure | Docker, health checks, security |
| Sidecar | Infrastructure | Co-located auxiliary container |
| API Gateway | Infrastructure | Single entry point, cross-cutting concerns |
| Observability | Cross-Cutting | Metrics, logging, tracing |
| Resilience | Cross-Cutting | Circuit breaker, retry, timeout |
| Security Boundary | Cross-Cutting | Auth, authorization, audit |
| Test Pyramid | Testing | Unit > Integration > E2E |
| Compiler | Language | Lexer → Parser → Semantic → Codegen |
| Visitor | Language | Separate algorithms from data |
