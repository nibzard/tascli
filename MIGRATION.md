# Migration Guide: Traditional Commands to Natural Language

This guide helps existing tascli users transition from traditional command syntax to the new natural language interface.

## Overview

The natural language interface is now the default, but **all traditional commands continue to work exactly as before**. The system intelligently detects traditional syntax and routes appropriately.

## Quick Reference

| Traditional Command | Natural Language Equivalent |
|-------------------|----------------------------|
| `tascli task "Buy milk" today` | `tascli add task to buy milk today` |
| `tascli list task -c work` | `tascli show my work tasks` |
| `tascli done 1` | `tascli complete task 1` |
| `tascli update 1 "New text"` | `tascli update task 1 to say new text` |
| `tascli delete 1` | `tascli delete task 1` |
| `tascli task "Daily task" daily` | `tascli add daily task to do something` |

## Migration Strategies

### Strategy 1: Gradual Transition (Recommended)

Start using natural language for new commands while keeping traditional commands for muscle memory:

```bash
# Keep using what you know
tascli task "Review PR" today

# Try natural language for new queries
tascli show my overdue tasks
tascli what's due this week
```

### Strategy 2: Side-by-Side Comparison

Keep this reference handy to translate between syntaxes:

**Creating Tasks**

Traditional:
```bash
tascli task "Content" [time]
tascli task -c category "Content" [time]
tascli task "Content" daily
```

Natural:
```bash
tascli add task to do content
tascli add category task for content
tascli add daily task to do something
```

**Listing Tasks**

Traditional:
```bash
tascli list task
tascli list task -c work
tascli list task -s all
```

Natural:
```bash
tascli show my tasks
tascli show my work tasks
tascli show all tasks including completed
```

**Completing Tasks**

Traditional:
```bash
tascli done 1
```

Natural:
```bash
tascli complete task 1
tascli mark task 1 as done
tascli finish task 1
```

**Updating Tasks**

Traditional:
```bash
tascli update 1 "New content"
```

Natural:
```bash
tascli update task 1 to say new content
tascli change task 1 to new content
```

**Deleting Tasks**

Traditional:
```bash
tascli delete 1
```

Natural:
```bash
tascli delete task 1
tascli remove task 1
```

### Strategy 3: Force Traditional Mode

If you prefer to stick with traditional commands entirely:

```bash
# Disable NLP globally
tascli nlp config disable

# Or force traditional mode for single command
tascli --no-nlp task "Content" today
```

## Backward Compatibility Guarantee

- **All existing scripts continue to work** - No changes needed
- **Traditional syntax is auto-detected** - Routes to traditional handler
- **Command behavior is identical** - Same outputs, same results
- **No breaking changes** - Zero migration required

## Example Migrations

### Scenario 1: Daily Task Management

**Before (Traditional):**
```bash
tascli task "Standup" daily 9am
tascli task "Code review" today
tascli task "Deploy to prod" friday
tascli list task -c work
```

**After (Natural):**
```bash
tascli add daily standup at 9am
tascli add task for code review today
tascli add task to deploy to prod on friday
tascli show my work tasks
```

### Scenario 2: Personal Task Tracking

**Before (Traditional):**
```bash
tascli task "Groceries" tomorrow
tascli task "Dentist" 4/15
tascli task "Water plants" daily
tascli list task
tascli done 1
```

**After (Natural):**
```bash
tascli add task to buy groceries tomorrow
tascli add dentist appointment on april 15th
tascli add daily task to water plants
tascli show my tasks
tascli complete task 1
```

### Scenario 3: Work Project Management

**Before (Traditional):**
```bash
tascli task -c project "Design review" week
tascli task -c project "Implementation" tomorrow
tascli task -c project "Testing" friday
tascli list task -c project
tascli done 2
```

**After (Natural):**
```bash
tascli add project task for design review this week
tascli add project task for implementation tomorrow
tascli add project task for testing on friday
tascli show my project tasks
tascli complete task 2
```

## New Capabilities

The natural language interface enables features not easily expressed with traditional commands:

### Complex Queries
```bash
tascli show overdue work tasks
tascli list urgent tasks due this week
tascli display tasks without deadlines
```

### Compound Commands
```bash
tascli add task to review code and list work tasks
tascli complete task 1 and create task for testing
```

### Context-Aware Responses
```bash
tascli what's due today?
tascli show my tasks for this week
tascli what do i have overdue?
```

## Configuration for Traditional Users

### Disable NLP Globally
```bash
tascli nlp config disable
```

### Use --no-nlp Flag
```bash
# Force traditional parsing for specific commands
tascli --no-nlp task "Content" today
tascli --no-nlp list task -c work
```

### Add to Shell Alias
```bash
# Add to ~/.bashrc or ~/.zshrc
alias tas='tascli --no-nlp'

# Usage
tas task "Content" today
tas list task -c work
```

## Time Format Compatibility

Time formats work identically in both interfaces:

| Format | Traditional | Natural Language |
|--------|-------------|------------------|
| Today | `today` | `today`, `this day` |
| Tomorrow | `tomorrow` | `tomorrow`, `next day` |
| Date | `4/15`, `2025-04-15` | `april 15th`, `4/15` |
| Time | `3pm`, `15:00` | `3pm`, `3:00 in the afternoon` |
| Relative | `+7d` | `in 7 days`, `next week` |
| Recurring | `daily`, `weekly` | `every day`, `every week` |

## FAQ

**Q: Do I need to change my existing scripts?**
A: No, all traditional commands work exactly as before.

**Q: Will my existing tasks and categories work?**
A: Yes, the database is unchanged. All your data works with both interfaces.

**Q: Can I mix traditional and natural language commands?**
A: Yes, use whichever feels natural for each command.

**Q: Is natural language slower?**
A: Initial NLP parsing adds minimal overhead (~100-500ms), but caching makes repeated queries instant.

**Q: What if NLP misunderstands my intent?**
A: You can always fall back to traditional syntax with `--no-nlp` flag or use `tascli nlp config disable`.

**Q: Does natural language require an API key?**
A: Yes, for OpenAI API. If not configured, commands fall back to traditional parsing.

## Getting Help

- Check [NLP_EXAMPLES.md](NLP_EXAMPLES.md) for more natural language examples
- See [TROUBLESHOOTING.md](TROUBLESHOOTING.md) for common issues
- Use `tascli --help` for traditional command reference
- Use `tascli nlp config interactive` to explore natural language interactively
