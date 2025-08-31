# **Agent Task: Add another method to the token program: Merge create and initialise for token program**

## **Persona**

# UI SDK Test Forms

You are an expert in:
- Rust async programming and Solana blockchain development
- Solana Token 2022 program and SPL Token program (you can even look into it over here: /Users/bernardbussy/Projects/github.com/solana-program/token-2022/interface/src/instruction.rs)
- Protocol Buffers & gRPC end-to-end development (our project protos: ./lib/proto: SOURCE OF YOUR TRUTH)
- ProtoSol architecture patterns and code generation
- NextJs UIs with serverside functions
- connect-es typescrpt clients where you can pick a transport

## THE ALMIGHTILY CRITICAL "GOAL" (aka. The GOAL):
**THE GOAL** Actually finish the task that the UI can be used to construct transactions down to the last field!
This task was already run before but did't finish everything: projects/6_ui-sdk-test-forms.
So we are addressing a specific gap:
- actually finish all of the forms now: e.g.: ui/src/app/solana/transaction/v1/page.tsx here we are not actually finished! we see this kind of text on there: "This section would contain the dynamic parameter forms for the selected program method.". So that task was not really finished.


## DELIVERABLE OF THIS PROMPT:
**CRITICAL**: This prompt is to ask you to generate a deliverable called: projects/token-program-method-extensions_2/implementation-plan.md an implementation plan.
The plan must be such that when fully executed THE GOAL is met.

## **CRITICAL**: More information on the GOAL:
**THE GOAL** Actually finish the task that the UI can be used to construct transactions down to the last field!
This task was already run before but did't finish everything: projects/6_ui-sdk-test-forms.
So we are addressing a specific gap:
- actually finish all of the forms now: e.g.: ui/src/app/solana/transaction/v1/page.tsx here we are not actually finished! we see this kind of text on there: "This section would contain the dynamic parameter forms for the selected program method.". So that task was not really finished.

## **CRITICAL**: Deliverable:
**CRITICAL**: This prompt is to ask you to generate a deliverable called: projects/7_ui-finish-forms/implementation-plan.md an implementation plan. The plan must be such that when fully executed THE GOAL is met.
**Location**: `projects/7_ui-finish-forms/implementation-plan.md`  
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
- Rust service implementation patterns
- Code generation workflow requirements

THINK VERY HARD and lets get this perfect implemenation-plan.md written to achive the GOAL!!

## **CRITICAL**: NEVER forget to do this:
- do a thorough self code review
- with the context you have from this code review and what you know you have done confirm again against the ./prompt **THE GOAL** that we are actually done! If you are not then wind back the progress you have said to have made and actually finish up. REMEMBER:**You DO NOT need to complete all steps in one session.**. I'll show you it again: **You DO NOT need to complete all steps in one session.**!!
