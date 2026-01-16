# Troubleshooting Guide

Solutions to common problems with ROTT.

## Quick Diagnostics

Run these commands to gather information about your setup:

```bash
# Check ROTT status and configuration
rott status

# Check configuration details
rott config show

# Check sync status (if sync is enabled)
rott sync status
```

---

## Sync Issues

### Sync Not Connecting

**Symptoms:**
- `rott sync` hangs or fails
- `rott sync status` shows disconnected

**Solutions:**

1. **Verify sync server is running:**
   ```bash
   # If using Docker
   docker ps | grep sync
   
   # Check logs
   docker logs rott-sync
   ```

2. **Check network connectivity:**
   ```bash
   # Test connection to sync server
   nc -zv YOUR_SERVER_IP 3030
   ```

3. **Verify configuration:**
   ```bash
   rott config show
   # Ensure sync_url is correct and sync_enabled is true
   ```

4. **Check URL format:**
   ```bash
   # Correct format (note: ws://, not http://)
   rott config set sync_url ws://192.168.1.100:3030
   ```

5. **Firewall issues:**
   ```bash
   # On the sync server, ensure port 3030 is open
   sudo ufw allow 3030
   ```

### "Pending Sync" State After Join

**Symptoms:**
- Just ran `rott init --join <id>`
- `rott status` shows "pending sync"
- No data appearing

**Solution:**

This is expected! After joining, you need to sync to pull your data:

```bash
# Perform the initial sync
rott sync

# Check status again
rott status
```

If sync fails, verify your sync server is reachable (see above).

### Data Not Appearing After Sync

**Symptoms:**
- Sync completes without errors
- But data from other devices isn't showing

**Solutions:**

1. **Verify same root document ID:**
   ```bash
   # Run on both devices
   rott status
   # Root document IDs must match exactly
   ```

2. **Check both devices point to same sync server:**
   ```bash
   rott config show
   # sync_url should be identical on all devices
   ```

3. **Force a sync on the source device:**
   ```bash
   # On the device that has the data
   rott sync
   ```

4. **Wait for propagation:**
   - Sync isn't instant—give it a few seconds
   - Run `rott sync` on the receiving device

### Changes Not Syncing

**Symptoms:**
- Make changes on one device
- They don't appear on another

**Solutions:**

1. **Ensure sync is enabled on both devices:**
   ```bash
   rott config set sync_enabled true
   ```

2. **Manually trigger sync:**
   ```bash
   # On device A (source)
   rott sync
   
   # On device B (destination)
   rott sync
   ```

3. **Check sync server logs for errors:**
   ```bash
   docker logs rott-sync
   ```

---

## Storage Issues

### Permission Denied Errors

**Symptoms:**
- Errors mentioning "permission denied"
- Can't read or write data

**Solutions:**

1. **Check data directory permissions:**
   ```bash
   ls -la ~/.local/share/rott/
   # Should be owned by your user
   ```

2. **Fix ownership:**
   ```bash
   sudo chown -R $USER:$USER ~/.local/share/rott/
   ```

3. **Check config directory:**
   ```bash
   ls -la ~/.config/rott/
   sudo chown -R $USER:$USER ~/.config/rott/
   ```

### Disk Full Errors

**Symptoms:**
- "No space left on device" errors
- "Disk full" messages

**Solutions:**

1. **Check available space:**
   ```bash
   df -h ~/.local/share/rott/
   ```

2. **Free up space on the disk**

3. **Check if SQLite WAL files are large:**
   ```bash
   ls -la ~/.local/share/rott/*.db*
   # .db-wal files should not be huge
   ```

### Corrupt Document Errors

**Symptoms:**
- "Document is corrupted" errors
- "Invalid format" errors

**Solutions:**

1. **ROTT automatically creates backups:**
   - Look for `.corrupt.backup` files in your data directory
   - These preserve the corrupted state for debugging

2. **If you have sync enabled:**
   - Delete the local corrupt file
   - Sync will restore from the server:
   ```bash
   rm ~/.local/share/rott/document.automerge
   rott sync
   ```

3. **If no sync (local only):**
   - Check for backups in your data directory
   - Consider restoring from your own backups

4. **Report the issue:**
   - Corruption shouldn't happen normally
   - File a bug report with details

---

## CLI Issues

### Command Not Found

**Symptoms:**
- `rott: command not found`

**Solutions:**

1. **Check if installed:**
   ```bash
   which rott
   ```

2. **Add to PATH (if installed locally):**
   ```bash
   # Add to ~/.bashrc or ~/.zshrc
   export PATH="$HOME/.cargo/bin:$PATH"
   ```

3. **Reinstall:**
   ```bash
   cargo install --path crates/rott-cli
   # or use the install script
   ./install.sh
   ```

### JSON Output Issues

**Symptoms:**
- JSON output is malformed
- Can't parse output in scripts

**Solutions:**

1. **Use the --json flag:**
   ```bash
   rott link list --json
   rott status --json
   ```

2. **Pipe through jq for validation:**
   ```bash
   rott link list --json | jq .
   ```

---

## TUI Issues

### TUI Won't Start

**Symptoms:**
- `rott tui` shows errors
- Terminal looks broken

**Solutions:**

1. **Check terminal compatibility:**
   - ROTT TUI requires a modern terminal
   - Try a different terminal emulator

2. **Reset terminal if corrupted:**
   ```bash
   reset
   ```

3. **Check TERM environment:**
   ```bash
   echo $TERM
   # Should be something like xterm-256color
   ```

### Display Issues / Garbled Output

**Symptoms:**
- Characters look wrong
- Layout is broken

**Solutions:**

1. **Ensure UTF-8 locale:**
   ```bash
   echo $LANG
   # Should contain UTF-8, e.g., en_US.UTF-8
   ```

2. **Try a different terminal emulator**

3. **Check terminal size:**
   - TUI needs minimum width/height
   - Try making the terminal window larger

---

## Identity Issues

### Lost Root Document ID

**Symptoms:**
- Don't know your root document ID
- Can't set up new devices

**If you still have ROTT running somewhere:**

```bash
# On a device that's working
rott status
# Copy the root document ID shown
```

**If no devices have ROTT:**
- Your data exists on the sync server but is inaccessible
- There's no recovery mechanism without the ID
- You'll need to start fresh with `rott init`

See [Understanding Your Root Document ID](IDENTITY.md) for prevention tips.

### Wrong Root Document ID on a Device

**Symptoms:**
- Device shows different data than expected
- Root document ID doesn't match other devices

**Solution:**

Reinitialize with the correct ID:

```bash
# First, backup if there's data you want to keep
rott link list > backup.txt

# Reinitialize with correct ID
rott init --join CORRECT_ROOT_DOCUMENT_ID

# Configure sync
rott config set sync_url ws://YOUR_SERVER:3030
rott config set sync_enabled true

# Sync
rott sync
```

---

## Configuration Issues

### Config File Location

ROTT stores configuration at:
- Linux: `~/.config/rott/config.toml`
- macOS: `~/Library/Application Support/rott/config.toml`

### Reset Configuration

```bash
# View current config
rott config show

# Reset a specific value
rott config set sync_url ""

# Or delete the config file to reset all
rm ~/.config/rott/config.toml
```

### Data Directory Location

ROTT stores data at:
- Linux: `~/.local/share/rott/`
- macOS: `~/Library/Application Support/rott/`

---

## Getting More Help

### Enable Debug Logging

```bash
# Set log file in config
rott config set log_file /tmp/rott-debug.log

# Run your command
rott link list

# Check the log
cat /tmp/rott-debug.log
```

### Check Version

```bash
rott --version
```

### Report a Bug

When reporting issues, include:

1. ROTT version (`rott --version`)
2. Operating system and version
3. Output of `rott status --json`
4. Steps to reproduce the problem
5. Any error messages (full text)
6. Debug log if possible

File issues at: https://github.com/evcraddock/rott/issues

---

## Common Error Messages

| Error | Likely Cause | Solution |
|-------|--------------|----------|
| "Permission denied" | File ownership | Fix with `chown` |
| "No space left on device" | Disk full | Free up space |
| "Connection refused" | Sync server down | Start sync server |
| "Document is corrupted" | Data corruption | Restore from sync/backup |
| "Root document not found" | Not initialized | Run `rott init` |
| "Sync URL not configured" | Missing config | Set `sync_url` |
