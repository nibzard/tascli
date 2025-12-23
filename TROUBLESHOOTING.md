# Troubleshooting Guide

This guide helps you diagnose and resolve common issues with tascli's natural language interface.

## Quick Diagnostics

Check your NLP configuration status:

```bash
tascli nlp config show
```

## Common Issues

### Issue: NLP Not Responding

**Symptoms:** Commands hang or timeout

**Solutions:**
1. Check your API key is set:
   ```bash
   tascli nlp config show
   ```

2. Verify OpenAI API access:
   ```bash
   curl https://api.openai.com/v1/models \
     -H "Authorization: Bearer $YOUR_API_KEY"
   ```

3. Check timeout settings (default 30s):
   ```bash
   # Increase timeout in config
   tascli nlp config set-timeout 60
   ```

4. Test with a simple command:
   ```bash
   tascli show my tasks
   ```

### Issue: "NLP not configured" Error

**Symptoms:** Error message about NLP configuration

**Solutions:**
1. Set your OpenAI API key:
   ```bash
   tascli nlp config set-key sk-your-api-key-here
   ```

2. Enable NLP if disabled:
   ```bash
   tascli nlp config enable
   ```

3. Verify configuration:
   ```bash
   tascli nlp config show
   ```

### Issue: NLP Misinterprets Commands

**Symptoms:** Wrong command executed or unexpected results

**Solutions:**
1. Enable transparency to see interpretation:
   ```bash
   tascli nlp config enable-transparency
   ```

2. Use command preview:
   ```bash
   tascli nlp config enable-preview
   ```

3. Try rephrasing:
   ```bash
   # Instead of: tascli make a task for...
   # Try: tascli add task to...
   ```

4. Use traditional syntax as fallback:
   ```bash
   tascli --no-nlp task "Content" today
   ```

5. Create shortcuts for frequent commands:
   ```bash
   tascli nlp config create-shortcut daily "show my tasks for today"
   ```

### Issue: Slow Response Times

**Symptoms:** Commands take longer than expected

**Solutions:**
1. Check cache statistics:
   ```bash
   tascli nlp config cache-stats
   ```

2. Clear cache if corrupted:
   ```bash
   tascli nlp config clear-cache
   ```

3. Common commands are cached and should be instant on repeat

4. Check network connectivity to OpenAI API

5. Consider disabling NLP for faster pure-traditional usage:
   ```bash
   tascli nlp config disable
   # or per-command
   tascli --no-nlp task "Content" today
   ```

### Issue: Category Not Recognized

**Symptoms:** Tasks created in wrong category

**Solutions:**
1. Use explicit category syntax:
   ```bash
   tascli add work task for code review
   tascli add task in category personal for groceries
   ```

2. Check existing categories:
   ```bash
   tascli list task
   ```

3. Update task category after creation:
   ```bash
   tascli move task 1 to work category
   ```

### Issue: Time/Date Misinterpreted

**Symptoms:** Wrong deadline assigned

**Solutions:**
1. Use specific date formats:
   ```bash
   tascli add task for review on april 15th
   tascli add task for meeting at 2025-04-15 3pm
   ```

2. Enable transparency to see interpretation:
   ```bash
   tascli nlp config enable-transparency
   ```

3. Verify result:
   ```bash
   tascli show my tasks
   ```

4. Update if wrong:
   ```bash
   tascli move task 1 to tomorrow
   ```

### Issue: Interactive Mode Problems

**Symptoms:** Interactive mode not working properly

**Solutions:**
1. Start interactive mode explicitly:
   ```bash
   tascli nlp config interactive
   ```

2. Built-in commands in interactive mode:
   - `exit` - Exit interactive mode
   - `help` - Show help
   - `context` - Show current context
   - `clear` - Clear context
   - `repeat` - Repeat last command
   - `history` - Show command history

3. Use Ctrl+D to exit

### Issue: Database Errors

**Symptoms:** Errors related to tascli.db

**Solutions:**
1. Check database location:
   ```bash
   # Default: ~/.local/share/tascli/tascli.db
   ```

2. Verify database file exists:
   ```bash
   ls -la ~/.local/share/tascli/tascli.db
   ```

3. Check permissions:
   ```bash
   chmod 644 ~/.local/share/tascli/tascli.db
   ```

4. Custom database location in config:
   ```json
   {
       "data_dir": "/custom/path"
   }
   ```
   at `~/.config/tascli/config.json`

### Issue: Cache Problems

**Symptoms:** Stale results or cache errors

**Solutions:**
1. Clear cache:
   ```bash
   tascli nlp config clear-cache
   ```

2. Check cache stats:
   ```bash
   tascli nlp config cache-stats
   ```

3. Cache is stored at: `~/.local/share/tascli/nlp_cache.db`

### Issue: API Key Problems

**Symptoms:** Authentication errors or quota issues

**Solutions:**
1. Verify API key format:
   ```bash
   # Should start with: sk-
   tascli nlp config show
   ```

2. Update API key:
   ```bash
   tascli nlp config set-key sk-new-key-here
   ```

3. Check OpenAI quota and billing:
   - Visit https://platform.openai.com/account/usage
   - Verify account has credits

4. Test API key directly:
   ```bash
   curl https://api.openai.com/v1/models \
     -H "Authorization: Bearer sk-your-key"
   ```

### Issue: Traditional Commands Not Working

**Symptoms:** Traditional syntax no longer works

**Solutions:**
1. Verify traditional syntax:
   ```bash
   tascli --no-nlp task "Content" today
   ```

2. Check NLP is not interfering:
   ```bash
   tascli nlp config show
   ```

3. Use `--no-nlp` flag to force traditional parsing

## Advanced Debugging

### Enable Debug Logging

Set environment variable for detailed logs:

```bash
RUST_LOG=debug tascli show my tasks
```

### View Mapped Commands

Enable transparency to see exactly what commands are executed:

```bash
tascli nlp config enable-transparency
tascli add task to review prs today
# Output: Interpreted as: tascli task "review prs" today
```

### Test Pattern Matching

Check available patterns:

```bash
tascli nlp config patterns
```

### Get Suggestions

Get suggestions for partial input:

```bash
tascli nlp config suggest "add t"
```

## Performance Tuning

### Cache Configuration

Cache is automatic and persistent (SQLite-based). TTL defaults to 7 days.

### Timeout Configuration

Adjust timeout for slower connections:

```bash
# Note: This requires manual config edit
# Edit ~/.config/tascli/config.json:
{
    "nlp": {
        "timeout_seconds": 60
    }
}
```

### Disable NLP for Performance

For maximum performance with traditional commands only:

```bash
tascli nlp config disable
```

## Getting Help

### Check Version

```bash
tascli --version
```

### Help Commands

```bash
tascli --help           # General help
tascli task --help      # Command-specific help
tascli nlp --help       # NLP-specific help
```

### Interactive Help

```bash
tascli nlp config interactive
> help
```

### Report Issues

If problems persist:
1. Gather diagnostic info: `tascli nlp config show`
2. Enable transparency and reproduce issue
3. Report at: https://github.com/Aperocky/tascli/issues

## Known Limitations

1. **API Required**: NLP requires OpenAI API key and internet connection
2. **Latency**: First-time queries have API call latency (~100-500ms)
3. **Cost**: Minimal API usage costs (~$0.01-0.05/month for moderate use)
4. **Ambiguity**: Some complex queries may be misinterpreted (use preview mode)

## Best Practices

1. **Start Simple**: Begin with basic commands, gradually use more complex queries
2. **Use Preview**: Enable preview mode to verify interpretations before execution
3. **Cache Benefits**: Repeat commands are instant due to caching
4. **Hybrid Approach**: Mix traditional and natural language based on preference
5. **Create Shortcuts**: For frequently used complex commands

## Migration from Traditional Commands

See [MIGRATION.md](MIGRATION.md) for detailed guidance on transitioning from traditional command syntax.

## More Examples

See [NLP_EXAMPLES.md](NLP_EXAMPLES.md) for comprehensive natural language examples.
