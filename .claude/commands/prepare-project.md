# /prepare-project

## Interactive Project Structure Setup Command

### Usage
```
/prepare-project [project-name]
```

### Purpose

This command guides you through creating a properly structured project that can be executed using `/execute-project`. It asks appropriate questions to gather requirements and generates the standardized project files based on the proven pattern established in `projects/add-rpc-client-wrapper`.

### What This Command Does

The command will:
1. **Create the project directory** under `projects/[project-name]`
2. **Interview you systematically** to understand your requirements
3. **Generate standardized project files**:
   - `prompt.md` - Task requirements and context
   - `implementation-plan.md` - Detailed step-by-step plan
   - `.claude-task` - Task metadata and progress tracking
4. **Validate the project structure** for completeness

### Interactive Interview Process

The command conducts a structured interview with appropriate follow-up questions. The conversation flow adapts based on your project type and requirements.

#### Core Question Categories

1. **Project Overview**
   - What is the main objective of this project?
   - What specific functionality are you trying to implement?
   - Is this adding new features, fixing bugs, refactoring, or something else?

2. **Technical Context**
   - Which parts of the codebase will this affect? (API, proto definitions, tests, etc.)
   - Are there existing patterns or services you want to follow?
   - Do you have specific technical requirements or constraints?

3. **Architecture & Integration**
   - How should this integrate with existing systems?
   - Are there dependencies on other services or components?
   - What validation/testing approach should be used?

4. **Implementation Approach**
   - Do you prefer incremental steps or larger chunks?
   - Are there any critical pre-review requirements?
   - What constitutes success for this project?

5. **Quality & Validation**
   - What tests need to pass?
   - Are there specific linting or build requirements?
   - How should progress be tracked and validated?

### Execution Flow

When you run `/prepare-project [project-name]`, the agent will:

1. **Initialize Project Structure**
   ```bash
   mkdir -p projects/[project-name]
   cd projects/[project-name]
   ```

2. **Conduct Interview**
   - Ask structured questions based on project context
   - Follow up with clarifying questions
   - Adapt conversation flow based on your responses
   - Continue until sufficient detail is gathered

3. **Generate `prompt.md`**
   - Original task requirements
   - Technical persona/context
   - Architecture references
   - Success criteria
   - Based on your interview responses

4. **Generate `implementation-plan.md`**
   - Context Management and Progress Tracking section
   - Step-by-step implementation plan
   - Pre-Review Requirements for each step
   - Complete code templates where possible
   - Validation steps
   - Following the gold standard pattern

5. **Generate `.claude-task`**
   - Task metadata and configuration
   - Progress tracking structure
   - Quick start instructions
   - Validation commands
   - Error recovery protocols

6. **Validate Structure**
   - Confirm all required files exist
   - Check file format compliance
   - Verify completeness

### Interview Question Examples

The conversation might look like:

```
🎯 Let's set up your project: [project-name]

What is the main objective of this project? For example:
- Add a new gRPC service
- Implement a new program wrapper
- Fix a specific bug
- Refactor existing functionality
- Add new test coverage

[Your response]

Great! You want to [restate their goal]. 

What specific functionality are you trying to implement? Please describe:
- The exact methods, endpoints, or features needed
- How users/systems will interact with this functionality
- Any specific technical requirements

[Your response continues...]

Which parts of the ProtoSol codebase will this affect?
- Proto definitions (lib/proto/...)
- Rust backend services (api/src/api/...)
- Generated SDKs (lib/rust, lib/go, lib/ts)
- Integration tests (tests/go/...)
- Other areas?

[Continue until sufficient detail...]
```

### Question Adaptation Strategy

The interview adapts based on project type:

- **New Service Projects**: Focus on proto design, service patterns, integration
- **Bug Fix Projects**: Focus on problem analysis, root cause, validation
- **Refactoring Projects**: Focus on scope, backwards compatibility, testing
- **Test Projects**: Focus on coverage, scenarios, validation approaches

### Generated File Quality

The generated files will:

- **Follow exact patterns** from the gold standard example
- **Include complete code templates** where patterns are established
- **Provide specific validation steps** relevant to your project
- **Include appropriate pre-review requirements**
- **Set up proper context management** for complex implementations

### Context Management Integration

Generated projects include proper context management:

- **Progress tracking sections** in `.claude-task`
- **Context reset protocols** in `implementation-plan.md`
- **Resume instructions** for continued work
- **Quality gates** to prevent shortcuts

### Example Usage

```bash
/prepare-project add-token-program-support

# Interactive conversation begins:
# 🎯 Let's set up your project: add-token-program-support
# What is the main objective of this project?
# > I want to add support for SPL Token program operations like mint, transfer, burn

# Conversation continues with follow-up questions...
# After completion:
# ✅ Project structure created successfully!
# 
# Files generated:
# - projects/add-token-program-support/prompt.md
# - projects/add-token-program-support/implementation-plan.md  
# - projects/add-token-program-support/.claude-task
#
# Ready to execute with: /execute-project projects/add-token-program-support
```

### Interview Completion Criteria

The interview ends when you have gathered:

1. **Clear objective** - What needs to be built/fixed/changed
2. **Technical approach** - How it should be implemented
3. **Integration points** - Where it connects to existing code
4. **Validation approach** - How to verify it works
5. **Quality requirements** - Testing, linting, documentation needs
6. **Implementation strategy** - Step breakdown and dependencies

### Quality Assurance

Generated projects ensure:

- **No placeholders** - Real, actionable implementation steps
- **Complete code examples** - Following established patterns
- **Proper validation** - Tests and verification steps
- **Context-aware** - Fits with existing architecture
- **Resumable** - Can be interrupted and continued

### Key Principles

1. **Thorough Discovery** - Ask enough questions to create quality plans
2. **Pattern Following** - Use proven project structure patterns  
3. **Adaptive Conversation** - Adjust questions based on project needs
4. **Complete Generation** - No half-finished or template files
5. **Validation Ready** - Generated projects ready for `/execute-project`

### Success Criteria

A project is properly prepared when:

1. ✅ All three required files exist and are complete
2. ✅ `prompt.md` clearly describes the requirements
3. ✅ `implementation-plan.md` has actionable steps with code
4. ✅ `.claude-task` has proper metadata and tracking setup
5. ✅ Project can be successfully executed with `/execute-project`

### Notes for Claude

This command should:
- **Ask thoughtful follow-up questions** to understand the full scope
- **Adapt conversation flow** based on project complexity
- **Generate complete, actionable files** not templates
- **Follow the gold standard pattern** established in `projects/add-rpc-client-wrapper`
- **Create projects ready for immediate execution**
- **Ensure quality through structured interview process**

The goal is to eliminate the overhead of project setup and let users focus on the actual implementation work.