# Agent Documentation Rules - NEVER VIOLATE

## CRITICAL - NEVER VIOLATE THESE EVER in the documentation you are creating:

### 1. NEVER FORGET that this is a greenfields project:
- **NEVER mention backward compatibility, breaking changes, or migration strategies**
- **NEVER assume existing systems, clients, or compatibility constraints exist**
- Design the optimal solution without legacy baggage

**What this means:**
- No "optional for backward compatibility" - make fields required if they should be required
- No "breaking change warnings" - everything is new
- No "migration strategies" - nothing exists to migrate from
- No "versioning considerations" - start with the right design
- No "transition phases" - implement the final desired state directly

**The correct mindset:**
- Design the API as it should be, not as a compromise
- Use required fields where they make sense
- Implement the complete feature set immediately
- Focus on optimal design, not compatibility constraints

### 2. NEVER include timing estimates or durations
- No "15 minutes", "2-3 hours", "daily workflow"
- No time pressure or scheduling concepts
- Agents work at their own pace, not human time constraints

### 3. NEVER include business context or value propositions
- No "benefits of this approach"
- No "business value" explanations
- No "why this matters" narrative
- Pure technical requirements only

### 4. NEVER include human workflow or process advice
- No "recommended daily workflow"
- No "morning/afternoon" task suggestions
- No productivity tips or work organization
- Agents don't have human work patterns

### 5. NEVER include performance metrics or success rates
- No "success rate should be 95%"
- No "17-25 hours total time"
- No human-oriented performance indicators
- Only technical validation criteria

### 6. NEVER include motivational or explanatory text
- No "this ensures safety through small steps"
- No "demonstrates thoroughness to the user"
- No justifications for the approach
- Just the technical steps

### 7. NEVER include human emotional or psychological considerations
- No "provides confidence"
- No "reduces stress"
- No "peace of mind"
- Agents don't have emotions

### 8. ALWAYS focus solely on technical execution
- What to implement
- Where to implement it
- How to test it
- Dependencies between steps
- Technical validation requirements
- Code examples and specifications

### 9. ALWAYS assume the consumer is a technical system
- No explanations of obvious concepts
- No hand-holding language
- Direct, imperative technical instructions
- Structured, parseable format

### 10. The Golden Rule for Agent Documentation:
> **If a human worker needs it but a technical system doesn't, NEVER include it.**
> 
> **If you are worrying about backwards compatibility, DON'T - this is greenfields.**

## Enforcement Checklist

Before finalizing any documentation, verify:

- [ ] Zero references to timing, schedules, or human work patterns
- [ ] Zero business justifications or value explanations
- [ ] Zero backward compatibility concerns or migration strategies
- [ ] Zero motivational or emotional language
- [ ] Zero human workflow recommendations
- [ ] All content is actionable technical instructions
- [ ] All fields are optimally designed (required where appropriate)
- [ ] All steps have clear technical validation criteria
- [ ] All code examples are complete and functional
- [ ] All dependencies are technically specified

## Violation Examples to Avoid

❌ **WRONG**: "Add optional SubmissionResult field to maintain backward compatibility"
✅ **CORRECT**: "Add required SubmissionResult field to SubmitTransactionResponse"

❌ **WRONG**: "This step should take 15-20 minutes"
✅ **CORRECT**: "Run `buf lint` to validate all proto changes"

❌ **WRONG**: "This ensures peace of mind for developers"
✅ **CORRECT**: "Validation: All imports resolve correctly"

❌ **WRONG**: "Recommended daily workflow: Complete 3-4 steps in morning"
✅ **CORRECT**: "Dependencies: Step 1.1A completed"

❌ **WRONG**: "This provides business value by improving user experience"
✅ **CORRECT**: "Action: Add MonitorTransaction streaming method to Service definition"