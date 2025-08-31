# ProtoSol Projects Directory

## Overview

This directory contains standardized project tasks for the ProtoSol codebase. Each project follows a consistent structure for reliable execution by Claude or other AI agents.

## Standardized File Structure

Every project MUST include these files:

### 1. `prompt.md`
Original task requirements including:
- Persona/expertise required
- Objective and deliverables
- Context and references
- Success criteria

### 2. `implementation-plan.md`
Detailed step-by-step plan following the gold standard format:
- **Context Management section** at the top explaining progress tracking
- **Pre-Review Requirements** for each step
- **Complete code** (no placeholders)
- **Validation steps**
- Based on: `add-rpc-client-wrapper/implementation-plan.md`

### 3. `.claude-task` (REQUIRED)
Task metadata, configuration, and built-in progress tracking:
- Task metadata and configuration
- Quick start instructions  
- Built-in Progress Log with timestamps:
```markdown
## Progress Log
### YYYY-MM-DD HH:MM:SS
- Action taken
- Findings
- Next step
- Status: [In Progress/Blocked/Complete]
```

## Executing Projects

Use the universal slash command:
```
/execute-project projects/[project-name]
```

This command will:
1. Validate the project structure
2. Read the implementation plan
3. Check .claude-task Progress Log for current state
4. Execute the task step-by-step
5. Handle context resets gracefully

## Creating New Projects

1. Create a new directory: `projects/your-project-name/`
2. Add `prompt.md` with requirements
3. Create `implementation-plan.md` following the gold standard
4. Copy and customize `.claude-task` from the template
5. The execution agent will update the Progress Log section automatically

## Gold Standard Example

See `projects/add-rpc-client-wrapper/` for the reference implementation that established these patterns.

## Key Principles

- **Standardization**: Consistent structure across all projects
- **Resumability**: Work can be paused and continued anytime
- **Quality**: Pre-review requirements prevent common mistakes
- **Reality**: Real implementation only, no simulations
- **Traceability**: Every action logged with timestamps

## Common Commands

```bash
# Generate code after proto changes
./scripts/code-gen/generate/all.sh

# Run linting (MANDATORY after code changes)
./scripts/lint/all.sh

# Start local services for testing
./scripts/tests/start-validator.sh
./scripts/tests/start-backend.sh
```

## Notes

- Projects can span multiple sessions
- Context resets are expected and handled
- Progress Log in .claude-task is "eventually consistent"
- Always verify actual state before continuing