# /execute-project

## Universal Project Execution Command

### Usage
```
/execute-project projects/[project-name]
```

### Purpose

Execute standardized project tasks using a consistent file structure based on the ProtoSol project patterns.

### Standardized Project File Structure

Every project MUST use these standardized files:

#### Required Files

1. **`prompt.md`**
   - Original task requirements
   - Persona/context for the implementer
   - Objective and deliverables
   - Architecture references
   - Success criteria

2. **`implementation-plan.md`**
   - CRITICAL: Context Management and Progress Tracking section
   - Step-by-step implementation with:
     - Pre-Review Requirements for each step
     - Action type (CREATE, MODIFY, VERIFY_EXISTS, etc.)
     - File paths
     - Complete code to implement
     - Validation steps
   - Based on the gold standard: projects/add-rpc-client-wrapper/implementation-plan.md

3. **`.claude-task`** (REQUIRED)
   - Task metadata and configuration
   - Built-in progress tracking with timestamps
   - Quick start instructions
   - Context management settings
   - Success criteria and validation commands

### Execution Protocol

When invoked with `/execute-project projects/[project-name]`:

#### Step 1: Validate Project Structure
```bash
# Check for required files
ls -la [project-path]/prompt.md
ls -la [project-path]/implementation-plan.md
ls -la [project-path]/.claude-task
```

#### Step 2: Read Context Management Section
Always read the "Context Management and Progress Tracking" section in implementation-plan.md FIRST. This section will tell you:
- How to track progress
- When to take breaks
- How to handle context resets
- Quality requirements

#### Step 3: Check Current Progress
```bash
# Read .claude-task file to check progress
cat [project-path]/.claude-task

# If progress section doesn't exist, it will be added when updating progress
# Progress is tracked within the .claude-task file itself
```

#### Step 4: Verify Actual State
Before continuing from recorded progress, ALWAYS verify the actual state:
- Check if files mentioned as created actually exist
- Run compilation commands to verify code state
- Run tests to see what's actually working

#### Step 5: Execute Implementation Plan

For each step in implementation-plan.md:

1. **Create Todo Item (REQUIRED)**
   - Use TodoWrite tool to add step to todo list
   - Mark as in_progress when starting
   - Provides visibility to user

2. **Read Pre-Review Requirements**
   - These are CRITICAL self-review checks
   - Never skip these
   - They prevent common mistakes

3. **Execute the Step**
   - Follow the action type (CREATE, MODIFY, etc.)
   - Use the exact code provided
   - For ANY code changes:
     a. Make the change
     b. **IMMEDIATELY** run appropriate lint script (`./scripts/lint/*.sh`)
     c. Fix any linting issues before proceeding
     d. **NEVER** use ignore directives - fix root causes

4. **Validate**
   - Run validation commands specified
   - If validation fails, mark todo as blocked
   - Only proceed when validation passes

5. **Update Progress & Todo**
   - Update progress section in `.claude-task` file:
   ```markdown
   ## Progress Log
   ### YYYY-MM-DD HH:MM:SS
   - Completed Step N: [description]
   - [Any important findings]
   - Next: Step N+1
   - Status: In Progress
   ```
   - Mark todo as completed in TodoWrite
   - Create next todo if applicable

#### Step 6: Handle Context Management

Based on the plan's context management section:
- Take breaks after specified number of steps
- Update progress before any context reset
- Use the resume protocol when continuing

### Quality Assurance Protocol

#### Pre-Review Requirements Are MANDATORY
Never skip the "Pre-Review Requirements" section of any step. These include:
- Goal verification
- Path consistency checks
- Import validation
- Pattern compliance
- Implementation validation checklists

#### Never Simulate Implementation
If you find yourself about to write:
- "Simulating..."
- "This would..."
- "Placeholder for..."
- Mock implementations

STOP immediately, update progress file, and request a context reset.

### Context Reset and Resume Protocol

#### When to Reset Context
- After completing 2 major steps
- When feeling overwhelmed
- When compilation errors become complex
- Before starting a new major section

#### How to Resume
When someone says: "Continue the project in projects/[project-name]" or uses the magic phrase from the plan:

1. Read .claude-task file (specifically the Progress Log section)
2. Verify actual state (files, compilation, tests)
3. Account for "eventually consistent" nature - you may have done more
4. Continue from the verified state

### Standard Response Template

When `/execute-project projects/[project-name]` is invoked:

```
I'll execute the project task in projects/[project-name] following the standardized implementation plan.

Let me check the project structure and current progress...

[Validate required files exist]
[Read Context Management section from implementation-plan.md]
[Check .claude-task progress section]
[Verify actual state]
[Begin/continue execution]
```

### File Not Found Handling

If required files are missing:

**Missing prompt.md:**
```
Error: No prompt.md found. This file should contain the original task requirements.
Please create prompt.md with the task description before proceeding.
```

**Missing implementation-plan.md:**
```
No implementation-plan.md found. Based on prompt.md, I'll create an implementation plan first.
[Read prompt.md]
[Generate implementation-plan.md following the gold standard format]
[Update .claude-task with progress section]
[Begin execution]
```

**Missing .claude-task:**
```
No .claude-task file found. I'll create one using the template and begin tracking progress...
```

### Success Criteria

A project is complete when:
1. ✅ All steps in implementation-plan.md are executed
2. ✅ Code compiles without errors
3. ✅ Tests pass
4. ✅ Linting is clean (`./scripts/lint/all.sh`)
5. ✅ .claude-task Progress Log shows "Status: Complete"
6. ✅ Original requirements from prompt.md are met

### Common Validation Commands

For ProtoSol projects:
```bash
buf lint                                 # Validate protos
./scripts/code-gen/generate/all.sh      # Generate code
cargo build                              # Validate Rust
go test -v                               # Run Go tests
./scripts/lint/all.sh                    # Run all linting
```

### Example Usage

```bash
# Start a new project
/execute-project projects/add-rpc-client-wrapper
# Response: Reads plan, creates progress file, begins Step 1

# Continue an in-progress project
/execute-project projects/add-rpc-client-wrapper
# Response: Reads progress, verifies state, continues from last step

# Handle a project missing implementation plan
/execute-project projects/new-feature
# Response: Reads prompt.md, generates implementation-plan.md, begins execution
```

### Key Principles

1. **Standardization**: All projects use the same file structure
2. **Traceability**: Every action is logged with timestamps
3. **Resumability**: Work can be paused and resumed anytime
4. **Quality**: Pre-review requirements prevent mistakes
5. **Reality**: Never simulate, always implement real code

### Gold Standard Reference

The implementation plan format is based on:
`projects/add-rpc-client-wrapper/implementation-plan.md`

This includes:
- Context Management and Progress Tracking section at the top
- Pre-Review Requirements for each step
- Complete, production-ready code
- Clear validation steps
- No placeholders or simulations

### Final Notes

This command is designed to:
- Work with any project that follows the standard structure
- Handle interruptions and context resets gracefully
- Maintain quality through systematic checks
- Provide clear progress visibility
- Enable collaboration across sessions

When in doubt, refer to the gold standard example and follow its patterns exactly.