# **Agent Task: Fix bug in ./bug-report.md!

## **Persona**

# UI SDK Test Forms

You are an expert in:
- Rust async programming and Solana blockchain development
- Solana Token 2022 program and SPL Token program (you can even look into it over here: /Users/bernardbussy/Projects/github.com/solana-program/token-2022/interface/src/instruction.rs)
- Protocol Buffers & gRPC end-to-end development (our project protos: ./lib/proto: SOURCE OF YOUR TRUTH)
- ProtoSol architecture patterns and code generation

## THE ALMIGHTILY CRITICAL "GOAL" (aka. The GOAL):
**THE GOAL** address the bug in ./bug-report.md

## DELIVERABLE OF THIS PROMPT:
**CRITICAL**: This prompt is to ask you to generate a deliverable called: projects/9_fix_txn_monitoring/implementation-plan.md an implementation plan.
The plan must be such that when fully executed THE GOAL is met.

## **CRITICAL**: More information on the GOAL:
**THE GOAL** address the bug in ./bug-report.md
This seems like a backend error! i.e. in ./api, if so that means that the e2e tests in ./tests/go are not catching this error.
Maybe that is not relly possibile, buf if so confirm we can check if a transaction is successful or not after streaming the result!

Also some front end feedback issue in the form as well. Maybe we need to be explicit on the UI somehow that all amounts are in lamports and not SOL (if that is ineed the truth, check that!!)


## **CRITICAL**: Deliverable:
**CRITICAL**: This prompt is to ask you to generate a deliverable called: rojects/9_fix_txn_monitoring/implementation-plan.md an implementation plan. The plan must be such that when fully executed THE GOAL is met.
**Location**: `projects/9_fix_txn_monitoring/implementation-plan.md`  
**Decription**: a step-by-step, purely technical, super comprehensive implementation plan for an agent to follow to achieve THE GOAL.
**Content**: Small, incremental technical steps
- Step-by-step validation checkpoints
- Technical dependencies and build ordering
- Resource management and cleanup requirements
- Each step builds on previous steps
- No orphaned or hanging code at any stage
- Prioritize incremental progress over large changes

**IMPORTANT: workflow while making this implementation plan**:
We must build up and break down the plan in passes. NO ONE SHOTTING. i.e. build up a sold solid plan bit by bit. Then look at it and, break it down into small, iterative chunks that build on each other. Look at these chunks and then go another round to break it into small steps. Review the results and make sure that the steps are small enough to be implemented safely with strong testing, but big enough to move the implementation forward. Iterate until you feel that the steps are right sized for achieving this goal!

This workflow ends with you taking a final look at implementation-plan.md: and taking a moment to write a "pre-review" advice section before each implementation step that could need it (i.e. this is optional) to add reassurances or review requirements like:
- note that code in this section is pseudo code and not final level, final implementation determined by agent! (i.e. here in the plan we have summary level code that you the implmementing agent must determine)
- check X files for special reference before doing this step!
- extra loosely associated file to look at before jumping in to this step: path, to some file

**FORBIDDEN CONTENT**:
- Implementation timelines, schedules, or time allocations
- Human workflow recommendations or project management advice
- References to daily/weekly/monthly patterns
- Time estimates for individual steps or overall completion

**Required HEDAER CONTENT for the implementation-plan.md file:**: some standard help for the Implementing Agent bout how to about its work:
"""
---
## CRITICAL: Context Management, Progress and Quality
Some critical information for the implemenation agent.

### Important Notice to Implementation Agent on Step Comletion

**You DO NOT need to complete all steps in one session.** There is NO requirement to fit everything in a single context window. This implementation can and should be done methodically, with breaks and context resets as needed.

### On Progress Tracking

1. **Create Progress File**: Before starting implementation, create/update your progress task tracking file .claude-task in the same directory as this plan
2. **Update Progress**: After completing each step or substep, update the progress file with:
   - Timestamp
   - Step completed
   - Any important findings or deviations
   - Next step to tackle
3. **Context Reset Protocol**: If you need to clear context or feel overwhelmed:
   - Update progress file with current state
   - Note any pending work
   - When resuming, use the magic phrase: "carry on with this task implementation-plan and take a look at the progress.md to see where you got up to"
4. **Eventually Consistent**: The progress file is eventually consistent - you may have progressed further than the last entry. Always verify the actual state before continuing (only relevant on RESTARTING TASK).

### On Quality Over Speed

**NEVER** simulate or fake implementation. If you find yourself writing comments like "simulating token mint creation" instead of actual code, STOP immediately, update the progress file, and request a context reset.
---
"""

## Research and Analysis Required
Construct yourself a comprehensive research todo list and execute it to get info you need as you build the implementation plan. It should cover:
- Existing lib/proto/protosol/solana api protobuf architecture and context. PROTOBUF files are source of truth in this repo.
- Existing ./api backend implementation of the services
- Existing tests/go integration e2e tests
- Existing UI implementation of these calls
- Rust service implementation patterns
- Code generation workflow requirements

THINK VERY HARD and lets get this perfect implemenation-plan.md written to achive the GOAL!!