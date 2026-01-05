//! MCP Prompt Handlers for Hielements
//!
//! Prompts provide guidance templates for common agent tasks.

use std::collections::HashMap;
use rust_mcp_sdk::schema::{ContentBlock, GetPromptResult, Prompt, PromptArgument, PromptMessage, Role, TextContent};
use tracing::debug;

/// Handler for MCP prompts
pub struct PromptHandler;

impl PromptHandler {
    /// Create a new prompt handler
    pub fn new() -> Self {
        Self
    }

    /// Helper to create a PromptArgument
    fn make_arg(name: &str, description: &str, required: bool) -> PromptArgument {
        PromptArgument {
            name: name.to_string(),
            description: Some(description.to_string()),
            required: Some(required),
            title: None,
        }
    }

    /// Helper to create a Prompt
    fn make_prompt(name: &str, description: &str, arguments: Vec<PromptArgument>) -> Prompt {
        Prompt {
            name: name.to_string(),
            description: Some(description.to_string()),
            arguments,
            icons: vec![],
            meta: None,
            title: None,
        }
    }

    /// List all available prompts
    pub fn list_prompts(&self) -> Vec<Prompt> {
        vec![
            Self::make_prompt(
                "architect_system",
                "Guide for designing a new system architecture with Hielements",
                vec![
                    Self::make_arg("system_description", "Description of the system to architect", true),
                    Self::make_arg("technology_stack", "Primary technologies (e.g., 'rust', 'python', 'docker')", false),
                ],
            ),
            Self::make_prompt(
                "analyze_architecture",
                "Guide for analyzing an existing Hielements specification",
                vec![
                    Self::make_arg("specification", "The Hielements specification content to analyze", true),
                ],
            ),
            Self::make_prompt(
                "create_pattern",
                "Guide for creating a reusable architectural pattern",
                vec![
                    Self::make_arg("pattern_purpose", "What the pattern should accomplish", true),
                    Self::make_arg("pattern_type", "Type: structural, behavioral, infrastructure, cross-cutting", false),
                ],
            ),
            Self::make_prompt(
                "fix_violations",
                "Guide for fixing architectural violations",
                vec![
                    Self::make_arg("violations", "The check failures or errors to address", true),
                ],
            ),
            Self::make_prompt(
                "implement_pattern",
                "Guide for implementing an architectural pattern",
                vec![
                    Self::make_arg("pattern_name", "Name of the pattern to implement", true),
                    Self::make_arg("context", "Context about the existing codebase", false),
                ],
            ),
        ]
    }

    /// Get a specific prompt
    pub fn get_prompt(
        &self,
        name: &str,
        arguments: Option<HashMap<String, String>>,
    ) -> Result<GetPromptResult, String> {
        debug!("Getting prompt: {} with args: {:?}", name, arguments);
        
        let args = arguments.unwrap_or_default();

        match name {
            "architect_system" => self.architect_system_prompt(args),
            "analyze_architecture" => self.analyze_architecture_prompt(args),
            "create_pattern" => self.create_pattern_prompt(args),
            "fix_violations" => self.fix_violations_prompt(args),
            "implement_pattern" => self.implement_pattern_prompt(args),
            _ => Err(format!("Unknown prompt: {}", name)),
        }
    }

    fn architect_system_prompt(&self, args: HashMap<String, String>) -> Result<GetPromptResult, String> {
        let system_description = args.get("system_description")
            .map(|s| s.as_str())
            .unwrap_or("a software system");
        
        let tech_stack = args.get("technology_stack")
            .map(|s| s.as_str())
            .unwrap_or("general");

        let prompt_text = format!(
            "You are an expert software architect helping to design a system using Hielements.\n\n\
            ## System to Design\n{}\n\n\
            ## Technology Stack\n{}\n\n\
            ## Your Task\n\
            Create a Hielements specification that:\n\
            1. Defines the main components as `element` declarations\n\
            2. Uses appropriate scopes to bind to code/artifacts\n\
            3. Establishes architectural checks to enforce rules\n\
            4. Uses patterns where applicable\n\n\
            Please generate a well-structured Hielements specification.",
            system_description, tech_stack
        );

        Ok(GetPromptResult {
            description: Some("System architecture design guide".to_string()),
            messages: vec![PromptMessage {
                role: Role::User,
                content: ContentBlock::TextContent(TextContent::from(prompt_text)),
            }],
            meta: None,
        })
    }

    fn analyze_architecture_prompt(&self, args: HashMap<String, String>) -> Result<GetPromptResult, String> {
        let specification = args.get("specification")
            .map(|s| s.as_str())
            .unwrap_or("");

        let prompt_text = format!(
            "You are an expert software architect analyzing a Hielements specification.\n\n\
            ## Specification to Analyze\n```hielements\n{}\n```\n\n\
            ## Analysis Tasks\n\
            1. Structure Analysis: Identify the main elements and their relationships\n\
            2. Pattern Recognition: Identify any patterns being used\n\
            3. Check Coverage: Assess the completeness of architectural checks\n\
            4. Improvement Suggestions: Recommend enhancements\n\n\
            Please provide a thorough analysis with actionable recommendations.",
            specification
        );

        Ok(GetPromptResult {
            description: Some("Architecture analysis guide".to_string()),
            messages: vec![PromptMessage {
                role: Role::User,
                content: ContentBlock::TextContent(TextContent::from(prompt_text)),
            }],
            meta: None,
        })
    }

    fn create_pattern_prompt(&self, args: HashMap<String, String>) -> Result<GetPromptResult, String> {
        let pattern_purpose = args.get("pattern_purpose")
            .map(|s| s.as_str())
            .unwrap_or("reusable architectural constraint");
        
        let pattern_type = args.get("pattern_type")
            .map(|s| s.as_str())
            .unwrap_or("structural");

        let prompt_text = format!(
            "You are an expert software architect creating a reusable Hielements pattern.\n\n\
            ## Pattern Purpose\n{}\n\n\
            ## Pattern Type\n{}\n\n\
            ## Your Task\n\
            Create a Hielements pattern that:\n\
            1. Uses the `pattern` keyword to declare the blueprint\n\
            2. Defines unbounded scopes with language annotations\n\
            3. Includes appropriate checks for the pattern's constraints\n\
            4. Documents the pattern with comments\n\n\
            Please generate a well-documented, reusable pattern.",
            pattern_purpose, pattern_type
        );

        Ok(GetPromptResult {
            description: Some("Pattern creation guide".to_string()),
            messages: vec![PromptMessage {
                role: Role::User,
                content: ContentBlock::TextContent(TextContent::from(prompt_text)),
            }],
            meta: None,
        })
    }

    fn fix_violations_prompt(&self, args: HashMap<String, String>) -> Result<GetPromptResult, String> {
        let violations = args.get("violations")
            .map(|s| s.as_str())
            .unwrap_or("");

        let prompt_text = format!(
            "You are an expert software architect helping to fix architectural violations.\n\n\
            ## Violations to Address\n{}\n\n\
            ## Your Task\n\
            For each violation:\n\
            1. Understand the Issue: Explain what the check is verifying\n\
            2. Root Cause: Identify why the check is failing\n\
            3. Fix Options: Provide potential solutions\n\
            4. Implementation: Show how to fix the code or specification\n\n\
            Please analyze each violation and provide actionable fixes.",
            violations
        );

        Ok(GetPromptResult {
            description: Some("Violation fix guide".to_string()),
            messages: vec![PromptMessage {
                role: Role::User,
                content: ContentBlock::TextContent(TextContent::from(prompt_text)),
            }],
            meta: None,
        })
    }

    fn implement_pattern_prompt(&self, args: HashMap<String, String>) -> Result<GetPromptResult, String> {
        let pattern_name = args.get("pattern_name")
            .map(|s| s.as_str())
            .unwrap_or("pattern");
        
        let context = args.get("context")
            .map(|s| s.as_str())
            .unwrap_or("No additional context provided");

        let prompt_text = format!(
            "You are an expert software architect helping to implement an architectural pattern.\n\n\
            ## Pattern to Implement\n{}\n\n\
            ## Context\n{}\n\n\
            ## Your Task\n\
            1. Understand the Pattern: Review the pattern's requirements\n\
            2. Map to Codebase: Identify how pattern elements map to actual code\n\
            3. Create Bindings: Write the element implementation with binds clauses\n\
            4. Verify Compliance: Ensure all pattern requirements are satisfied\n\n\
            Please provide a complete pattern implementation.",
            pattern_name, context
        );

        Ok(GetPromptResult {
            description: Some("Pattern implementation guide".to_string()),
            messages: vec![PromptMessage {
                role: Role::User,
                content: ContentBlock::TextContent(TextContent::from(prompt_text)),
            }],
            meta: None,
        })
    }
}

impl Default for PromptHandler {
    fn default() -> Self {
        Self::new()
    }
}
