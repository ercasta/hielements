# Hielements V2

Hielements V2 is a second generation of the language. This second generation is incompatible with the original hielements language. The original hielements language is deprecated and unsupported. 

The goal of hielements v2 is having a clearer separation of responsibilities between *descriptive* and *prescriptive* parts of the language.

Specifically:
- The *prescriptive* part is made of *element templates*, the requires / forbids / allows keywords, and checks
- The *descriptive* part is made of elements that *implement* templates and scopes that bind to actual code.

It is possible to use the descriptive part without the prescriptive one; in this case, no enforcement / checks are performed

Element templates declare scopes and rules:

```
template observable:
    element metrics implements measurable:
        allows language rust

        scope module<rust>  # angular brackets specify the language; in original hielements, language was specified after columns
        connection_point prometheus: MetricsHandler

        check files.exists(module, 'Cargo.toml')
 
```
scope in templates is always unbounded.

Elements *binds* these:

```
element observable_component implements observable:
    scope main_module<rust> binds observable.metrics.module = rust.module_selector('payments::api')
    connection_point main_handler: MetricsHandler binds observable.metrics.prometheus  = rust.function_selector(main_module, 'handler')
```

The implements and binds keywords, and the language specification via angular brackets are optional, and only used when the prescriptive features of the language are used. 


