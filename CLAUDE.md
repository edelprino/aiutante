# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Minions is a Rust-based AI agent framework that integrates with OpenAI's GPT-4 to create specialized AI agents with tool capabilities. The system uses a prompt-driven architecture where agents are defined by markdown files containing instructions and tool specifications.

## Architecture

- **Agent System**: Agents are defined in `army/{agent}.md` files containing prompts and tool metadata
- **Tool Framework**: Built on the `rig-core` crate for AI completions and tool integration
- **OpenAI Integration**: Uses GPT-4 models via the OpenAI API
- **Tool Execution**: Currently supports Python code execution and Android ADB operations

### Key Components

- `src/main.rs`: CLI entry point using clap for argument parsing
- `src/tools/`: Tool implementations following the rig-core Tool trait
- `army/`: Agent definitions as markdown files with embedded tool metadata
- `army/tools/`: YAML tool definition files for external commands

## Development Commands

### Build and Run
```bash
cargo build              # Build the project
cargo check              # Type check without building
cargo run -- run <agent> # Run a specific agent (e.g., fungaiolo)
```

### Available Tools
- **PythonTool**: Executes Python code via `python3 -c`
- **ADB Tools**: Android device interaction via `adb` commands (defined in `army/tools/adb.yml`)

## Agent System

Agents are configured through markdown files in the `army/` directory:
- First line specifies available tools: `tools: [tool1, tool2]`
- Rest of file contains the agent's prompt/instructions
- Tool definitions can reference YAML files in `army/tools/`

### Example Agent Usage
```bash
cargo run -- run fungaiolo
```
This runs the mushroom hunting advisor agent with access to Python tools.

## Dependencies

Key dependencies:
- `rig-core`: AI completion and tool framework
- `clap`: CLI argument parsing
- `tokio`: Async runtime
- `serde`: Serialization for tool arguments
- `openai`: OpenAI API client (via rig-core)

## Environment Setup

Requires:
- OpenAI API key (set via environment variables)
- Python 3 for Python tool execution
- ADB for Android device tools