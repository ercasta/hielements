# README Update: V2 Syntax and Prescriptive/Descriptive Concepts

## Task Description
Update the main README.md to reflect V2 of the Hielements language and introduce the prescriptive/descriptive concepts.

## Changes to hielements.hie
No changes needed to hielements.hie for this task.

## Implementation Changes

### README.md Updates

#### 1. Added V2 Version Note (Top Section)
- Added a note below the subtitle indicating this documentation describes Hielements V2
- Mentioned that V2 is incompatible with V1

#### 2. New "Prescriptive vs Descriptive" Section
Added a dedicated section after "Why Hielements?" explaining:
- **Prescriptive**: Element templates, checks, and constraint keywords (requires/forbids/allows)
- **Descriptive**: Elements and scopes that bind to actual code
- Flexibility to use either mode or both together

#### 3. Updated All Code Examples to V2 Syntax
Updated every code example in the README with V2 syntax:
- Angular brackets for language specification: `scope module<python>` instead of `scope module : python`
- Unbounded scopes in templates: `scope module<rust>` without `=` expression
- `binds` keyword for connecting implementations to templates: `scope api_mod<python> binds microservice.api.module = ...`

**Sections updated:**
- Quick Example
- Reusable Templates
- Reusable Element Templates (Key Features)
- Hierarchical Checks
- Cross-Technology Elements
- Your First Hielements Spec

#### 4. Enhanced "How It Works" Section
Restructured to reflect the prescriptive/descriptive distinction:
- Step 1: Define Elements (Descriptive)
- Step 2: Define Templates (Prescriptive - Optional)
- Step 3: Bind Implementations to Templates
- Step 4: Write Rules
- Step 5: Run Checks

#### 5. Updated Philosophy Section
Enhanced to mention the flexibility of descriptive, prescriptive, and hybrid approaches:
- Descriptive mode for documenting what exists
- Prescriptive mode for enforcing rules
- Hybrid approach for mixing both

#### 6. Added FAQ Entry
New FAQ question: "What's the difference between prescriptive and descriptive modes?"
- Explains when to use each mode
- Clarifies that they can be mixed

## Key V2 Syntax Changes Demonstrated

### Language Annotation with Angular Brackets
**Before (V1):**
```hielements
scope module : python = python.module_selector('orders')
```

**After (V2):**
```hielements
scope module<python> = python.module_selector('orders')
```

### Unbounded Scopes in Templates
**V2 (Templates):**
```hielements
template microservice:
    element api:
        scope module<python>  # No '=' expression - unbounded
```

### Binding with `binds` Keyword
**V2 (Implementation):**
```hielements
element orders_service implements microservice:
    scope api_mod<python> binds microservice.api.module = python.module_selector('orders.api')
```

## Benefits
1. **Clear V2 identification**: Users know they're looking at V2 documentation
2. **Prescriptive/Descriptive clarity**: Explicitly explains the two modes and when to use each
3. **Consistent V2 syntax**: All examples now demonstrate current V2 syntax
4. **Better onboarding**: New users understand the flexibility of descriptive vs prescriptive approaches
5. **Accurate examples**: Code examples match the actual V2 implementation

## Validation
- All code examples are consistent with V2 syntax from language_reference.md
- Examples align with v2_syntax_example.hie
- README maintains comprehensive coverage of features
- Added explanatory context without removing existing valuable content
