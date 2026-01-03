#!/usr/bin/env python3
"""
Generate Pattern Catalog from Hielements pattern library files.

This script scans the patterns/ directory, extracts pattern definitions
from .hie files, and generates a comprehensive markdown catalog.
"""

import os
import re
from pathlib import Path
from typing import List, Dict, Optional


class Pattern:
    """Represents a single pattern extracted from a .hie file."""
    
    def __init__(self, name: str, category: str, file_path: str):
        self.name = name
        self.category = category
        self.file_path = file_path
        self.description = ""
        self.use_cases = ""
        self.content = ""
    
    def __repr__(self):
        return f"Pattern({self.name}, {self.category})"


def extract_patterns_from_file(file_path: Path, category: str) -> List[Pattern]:
    """Extract pattern information from a .hie file."""
    patterns = []
    
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    # Extract description and use cases from header comments
    description_match = re.search(r'##\s+Description:\s*\n##\s+(.*?)(?:\n##\s+Use Cases:|$)', content, re.DOTALL)
    use_cases_match = re.search(r'##\s+Use Cases:\s*\n((?:##\s+.*\n)*)', content)
    
    description = ""
    use_cases = ""
    
    if description_match:
        description = description_match.group(1).strip()
        description = re.sub(r'\n##\s*', ' ', description)
    
    if use_cases_match:
        use_cases_text = use_cases_match.group(1)
        use_cases = re.sub(r'##\s+-\s+', '\n- ', use_cases_text).strip()
    
    # Extract pattern name from filename
    pattern_name = file_path.stem.replace('_', ' ').title()
    
    pattern = Pattern(pattern_name, category, str(file_path))
    pattern.description = description
    pattern.use_cases = use_cases
    pattern.content = content
    
    patterns.append(pattern)
    
    return patterns


def scan_patterns_directory(patterns_dir: Path) -> Dict[str, List[Pattern]]:
    """Scan the patterns directory and extract all patterns organized by category."""
    categories = {}
    
    # Define category order and display names
    category_names = {
        'structural': 'Structural Patterns',
        'behavioral': 'Behavioral Patterns',
        'creational': 'Creational Patterns',
        'infrastructure': 'Infrastructure Patterns',
        'cross-cutting': 'Cross-Cutting Patterns',
        'testing': 'Testing Patterns',
        'compiler': 'Compiler/Interpreter Patterns'
    }
    
    for category_dir in sorted(patterns_dir.iterdir()):
        if category_dir.is_dir() and category_dir.name in category_names:
            category = category_dir.name
            display_name = category_names[category]
            patterns = []
            
            for hie_file in sorted(category_dir.glob('*.hie')):
                patterns.extend(extract_patterns_from_file(hie_file, category))
            
            if patterns:
                categories[display_name] = patterns
    
    return categories


def generate_markdown_catalog(categories: Dict[str, List[Pattern]]) -> str:
    """Generate markdown documentation from extracted patterns."""
    
    lines = [
        "# Hielements Pattern Catalog",
        "",
        "This catalog documents common software engineering patterns and their implementation in Hielements. "
        "It serves both as a reference for users and as a test suite for the language's prescriptive capabilities.",
        "",
        "> **Note**: This documentation is automatically generated from the pattern library in the `patterns/` directory. "
        "Every pattern uses the prescriptive features of Hielements (patterns, `requires`, `forbids`, `allows`, `check`, `ref`, `uses`).",
        "",
        "---",
        "",
        "## Table of Contents",
        ""
    ]
    
    # Generate table of contents
    for category_name, patterns in categories.items():
        category_anchor = category_name.lower().replace(' ', '-').replace('/', '')
        lines.append(f"- [{category_name}](#{category_anchor})")
        for pattern in patterns:
            pattern_anchor = pattern.name.lower().replace(' ', '-').replace('(', '').replace(')', '').replace('/', '')
            lines.append(f"  - [{pattern.name}](#{pattern_anchor})")
    
    lines.extend(["", "---", ""])
    
    # Generate pattern sections
    for category_name, patterns in categories.items():
        lines.append(f"## {category_name}")
        lines.append("")
        
        for pattern in patterns:
            pattern_anchor = pattern.name.lower().replace(' ', '-')
            lines.append(f"### {pattern.name}")
            lines.append("")
            
            if pattern.description:
                lines.append(f"**Description**: {pattern.description}")
                lines.append("")
            
            if pattern.use_cases:
                lines.append(f"**Use Cases**:")
                lines.append(pattern.use_cases)
                lines.append("")
            
            lines.append("**Hielements Implementation**:")
            lines.append("")
            lines.append("```hielements")
            lines.append(pattern.content)
            lines.append("```")
            lines.append("")
            lines.append("---")
            lines.append("")
    
    # Add usage guidelines section
    lines.extend([
        "## Pattern Usage Guidelines",
        "",
        "### When to Use Patterns",
        "",
        "- **DO** use patterns when you have multiple components with similar structure",
        "- **DO** use patterns to enforce architectural decisions across teams",
        "- **DO** use patterns as documentation for expected component structure",
        "- **DON'T** use patterns for truly unique one-off components",
        "- **DON'T** over-engineer with patterns when a simple element suffices",
        "",
        "### Pattern Composition",
        "",
        "Patterns can be composed through multiple `implements`:",
        "",
        "```hielements",
        "## Service implementing multiple patterns",
        "element production_service implements microservice, observability, resilience {",
        "    ## Microservice bindings",
        "    scope api<rust> binds microservice.api.module = rust.module_selector('service::api')",
        "    ",
        "    ## Observability bindings  ",
        "    scope metrics<rust> binds observability.metrics.module = rust.module_selector('service::metrics')",
        "    ",
        "    ## Resilience bindings",
        "    scope circuit_breaker<rust> binds resilience.circuit_breaker.module = rust.module_selector('service::resilience')",
        "}",
        "```",
        "",
        "### Contributing New Patterns",
        "",
        "When adding new patterns to this catalog:",
        "",
        "1. Create a `.hie` file in the appropriate category directory",
        "2. Include a description comment block with the pattern's intent and use cases",
        "3. Implement using Hielements prescriptive features (`pattern`, `requires`, `forbids`, `allows`, `check`, `ref`, `uses`)",
        "4. Provide at least one concrete implementation example",
        "5. Regenerate this catalog using: `python3 scripts/generate_pattern_catalog.py`",
        "",
        "---",
        "",
        "**Note**: This catalog is automatically generated from the Hielements pattern library. "
        "To add or modify patterns, edit the `.hie` files in the `patterns/` directory and regenerate this documentation.",
    ])
    
    return '\n'.join(lines)


def main():
    """Main entry point."""
    # Find the patterns directory
    script_dir = Path(__file__).parent
    repo_root = script_dir.parent
    patterns_dir = repo_root / 'patterns'
    output_file = repo_root / 'doc' / 'patterns_catalog.md'
    
    if not patterns_dir.exists():
        print(f"Error: Patterns directory not found at {patterns_dir}")
        return 1
    
    print(f"Scanning patterns directory: {patterns_dir}")
    categories = scan_patterns_directory(patterns_dir)
    
    total_patterns = sum(len(patterns) for patterns in categories.values())
    print(f"Found {total_patterns} patterns in {len(categories)} categories")
    
    print("Generating markdown catalog...")
    markdown = generate_markdown_catalog(categories)
    
    # Write output
    output_file.parent.mkdir(parents=True, exist_ok=True)
    with open(output_file, 'w', encoding='utf-8') as f:
        f.write(markdown)
    
    print(f"✓ Pattern catalog generated: {output_file}")
    print(f"  {total_patterns} patterns documented")
    
    return 0


if __name__ == '__main__':
    exit(main())
