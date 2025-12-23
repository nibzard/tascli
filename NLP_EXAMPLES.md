# Natural Language Examples

This guide provides comprehensive examples of using tascli's natural language interface.

## Quick Start

First, configure your OpenAI API key:

```bash
tascli nlp config set-key sk-your-api-key-here
```

## Creating Tasks

### Basic Task Creation

```bash
# Simple task
tascli add task to buy groceries

# Task with deadline
tascli add task to review PRs today
tascli create a task for buying milk tomorrow
tascli remind me to call mom on friday

# Task with category
tascli add a work task to fix the bug
tascli create personal task for dentist appointment
```

### Recurring Tasks

```bash
# Daily task
tascli add a daily task to water plants
tascli create recurring task to take vitamins daily

# Weekly task
tascli remind me to clean the house every saturday
tascli add weekly task for team standup every monday

# Monthly task
tascli create monthly task to pay rent on the 1st
tascli add task to pay credit card every 15th
```

### Tasks with Specific Times

```bash
# Specific date
tascli add task to submit report on 4/15
tascli create task for dentist appointment on 2025-05-20

# Specific time
tascli add task to call client at 3pm
tascli remind me about meeting at 2:30pm

# Date and time
tascli add task for doctor appointment on friday at 10am
tascli create task to submit taxes by april 15th 11:59pm
```

### Relative Times

```bash
# Relative to today
tascli add task to finish project in 3 days
tascli create task to review code in 2 hours

# Next/last occurrences
tascli add task for next monday
tascli remind me about meeting next friday
tascli create task due by end of month
```

## Listing and Searching Tasks

### Basic Listing

```bash
# List all tasks
tascli show my tasks
tascli list all tasks
tascli display tasks

# List by category
tascli show my work tasks
tascli list personal tasks
tascli display all tasks in category project
```

### Filtering by Status

```bash
# Overdue tasks
tascli show overdue tasks
tascli list all overdue tasks
tascli display tasks that are overdue

# Upcoming tasks
tascli show upcoming tasks
tascli list tasks due this week
tascli display tasks for tomorrow

# Unscheduled tasks
tascli show tasks without deadline
tascli list unscheduled tasks
```

### Time-Based Queries

```bash
# Due today
tascli show tasks due today
tascli what tasks are due today
tascli list today's tasks

# Due tomorrow
tascli show tasks due tomorrow
tascli what's due tomorrow

# This week/month
tascli show tasks due this week
tascli list tasks for this month
```

### Priority and Urgency

```bash
# Urgent tasks
tascli show urgent tasks
tascli list high priority tasks
tascli display all urgent items
```

### Searching

```bash
# Search by keyword
tascli search for tasks about bug
tascli find tasks containing review
tascli show tasks matching project

# Combined filters
tascli show overdue work tasks
tascli list urgent tasks for project
tascli display tasks due today in category work
```

## Completing Tasks

### Mark as Done

```bash
# By index
tascli complete task 1
tascli mark task 2 as done
tascli finish the first task

# Multiple tasks
tascli complete tasks 1, 2, and 3
tascli mark task 1 and 2 as done
tascli finish first 3 tasks

# Natural language
tascli i finished the review task
tascli mark the grocery shopping as complete
```

## Updating Tasks

```bash
# Update content
tascli update task 1 to say fix authentication bug
tascli change task 2 to buy milk and eggs

# Update deadline
tascli move task 1 to tomorrow
tascli reschedule task 2 to next friday
tascli change deadline of task 1 to today

# Update category
tascli move task 1 to work category
tascli change category of task 2 to personal
```

## Deleting Tasks

```bash
# By index
tascli delete task 1
tascli remove task 2
tascli delete the first task

# Multiple tasks
tascli delete tasks 1 and 2
tascli remove tasks 1, 2, 3
```

## Records

### Creating Records

```bash
# Basic record
tascli log feeding 100ml
tascli record workout completed

# With category
tascli log feeding 100ml in category baby
tascli record gym session in category fitness

# With time
tascli record meeting at 2pm
tascli log lunch at 12:30pm
```

### Listing Records

```bash
# Recent records
tascli show recent records
tascli list records from today

# By category
tascli show feeding records
tascli list workout records

# Date range
tascli show records from yesterday
tascli list records for this week
```

## Compound Commands

Execute multiple operations in a single command:

```bash
# Create and list
tascli add task to review code and show all tasks

# Complete and create
tascli finish task 1 and create task for testing

# Multiple operations
tascli add work task for code review, add personal task for groceries, then list all tasks
```

## Conditional Commands

```bash
# Conditional based on task count
tascli if there are more than 5 tasks, show urgent tasks

# Conditional based on time
tascli if it's friday, list tasks for weekend

# Conditional based on previous result
tascli if the last task was completed, create a new task
```

## Interactive Mode

Enter interactive mode for multi-step conversations:

```bash
# Start interactive mode
tascli nlp config interactive

# Example conversation
> add task to review code
> when is it due?
> tomorrow
> also add task for testing
> show my tasks
> exit
```

## Advanced Features

### Command Preview

See what commands will be executed before running:

```bash
# Enable preview
tascli nlp config enable-preview

# Commands will show preview before execution
tascli add task to review prs and list work tasks

# Auto-confirm (skip prompt)
tascli nlp config enable-auto-confirm
```

### Transparency

See how NLP interprets your input:

```bash
# Enable transparency
tascli nlp config enable-transparency

# Commands will show mapped tascli commands
tascli add task to review prs today
# Output: Interpreted as: tascli task "review prs" today
```

### Suggestions

Get suggestions for partial input:

```bash
# Get suggestions
tascli nlp config suggest "add t"
# Suggests: add task, add task today, add task to, etc.

# Show all patterns
tascli nlp config patterns
```

### Personalization

Create shortcuts for frequent commands:

```bash
# Create shortcut
tascli nlp config create-shortcut daily "show my tasks for today"

# List shortcuts
tascli nlp config list-shortcuts

# Delete shortcut
tascli nlp config delete-shortcut daily

# Reset personalization
tascli nlp config personalization-reset
```

## Configuration Commands

```bash
# Enable/disable NLP
tascli nlp config enable
tascli nlp config disable

# Set API key
tascli nlp config set-key sk-your-key-here

# Show configuration
tascli nlp config show

# Cache management
tascli nlp config cache-stats
tascli nlp config clear-cache
```

## Troubleshooting

See [TROUBLESHOOTING.md](TROUBLESHOOTING.md) for common issues and solutions.
