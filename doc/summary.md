# Summary

Hielements is a language to describe and enforce software structure. Hielements does not substitute traditional languages, it complements them. It can be used both for greenfield and brownfield development. Given a software, the language allows definining *elements* by specifying the element *scope* (e.g. files, folders, functions), *rules* for these elements (that will be actually statically checked), and *connection points*, to make the relationship between elements explicit.

# Why

Hielements allows developers (humans and agents) to easily understand the actual logical structure of a software; this structure is not described in documentation files, but with a formal language that is actually enforced, so the representation is guaranteed to be in sync with the actual software.

This simplifies reasoning and evolving large, complex software projects.

# Language Description

# Elements

Elements are a logical grouping of software elements. They are defined by:
- a *scope*
- several *rules*
- *connection points*
- *Children* elements


## Scope
An element can encompass multiple languages and artifacts, e.g. some python functions, a Terraform configuration file, and a Docker file. The element's scope is specified by a set of *scope selectors*. 

Examples:

```
    orders_module = python.python_module_selector('orders')
    orders_db = python.python_module_selector('orders_db')
    docker_file = docker.docker_file_selector('orders_service.dockerfile')

```

## Rules
Rules allow actually making sure the elements have specific characteristics. After all, the purpose of having higher-level components is to be able to specify their semantics.

For examples, we might want to be sure that the element exposes port 8080 in the container:

```
    docker.check_exposes_port(docker_file, 8080)
```

## Connection points

Elements don't live in isolation. They live in (often complex) relationships among them. Connection points allow specifying these relationships. As an example, if we want to make sure a python main module is the entry point of a dockerfile, we need to use connection points:

``` 
    # Note: sample syntax just to make the example
    orders_module = python.python_module_selector('orders')
    docker_file = docker.docker_file_selector('orders_service.dockerfile')

    docker.check_entry_point(docker_file, orders_module.main_module)
```

## Children Elements

To have a truly hierarchical representation, we need to be able to specify compositions. This is also needed to make relevant properties of the element explicit at top level.

```
    element full_orders:

        element python_orders_element
        element python_orders_db_element
        element docker_file_element

        check docker.check_entry_point(docker_file_element, python_orders_element.main_module)

    element python_orders_element:
        scope orders = python.python_module_selector('orders')
        ref main_module = python.get_main_module(orders)

    element python_orders_db_element:
        scope python.python_module_selector('orders_db')

```

# Example use cases

Hielements could be used:
- In greenfield development, by describing software top down (even with the help of agents), and the creating the actual implementation
- In brownfield development; the Hielements representation of a system can be created with the help of agentic AI / code analysis tools; to make changes to the software, the Hielements description is analyzed, changed, and then actual code can be changed

In all of these situations, Hielements acts as "design guardrail", making sure the software is aligned to a specific design.

# Implementation

- Hielements is an interpreted language
- The Hielements core language only provides basic keywords and structures. Actual scope and rules implementation is implemented using external software; the Hielements provides a way to invoke external tools, passing parameters and getting the results.
- Hielements support for different languages is implemented via specific Hielements libraries. Some of these libraries are included in the base Hielements distribution (such as python, docker, files and folders). Users can create their own Hielements libraries to support other languages.
- The interpreter itself is written in Rust
- The Hielements toolset includes:
    - The interpreter
    - A Language Server Protocol
    - VSCode Extension for syntax highlighting, interaction with the Language Server Protocol
- To allow a full-fledged implementation of the Language Server Protocol, consider using an intermediate representation within the interpreter.





