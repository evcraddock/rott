# Understanding Your Root Document ID

Your root document ID is the most important piece of information in ROTT. This guide explains what it is, why it matters, and how to keep it safe.

## What Is the Root Document ID?

When you first run `rott init`, ROTT creates a **root document**—an Automerge document that contains all your links and notes. This document has a unique identifier that looks something like:

```
2DrjgbFkSxXwGBKnVGbLjtPnqTo3r
```

This ID is:
- Randomly generated and globally unique
- Base58 encoded (similar to Bitcoin addresses) for readability
- The key to accessing your data across all your devices

## The Root Document ID IS Your Identity

Unlike traditional apps with usernames and passwords, ROTT uses a simpler model:

- **No accounts** - There's no server-side user registration
- **No passwords** - Your root document ID is your credential
- **No email verification** - Just the ID

This means:

| Traditional App | ROTT |
|-----------------|------|
| Username + Password → Your data | Root Document ID → Your data |
| Forgot password? Reset via email | Lost ID? Data inaccessible |
| Account stored on server | ID stored only on your devices |

## Why This Design?

ROTT is **local-first** and **self-hosted**:

1. **Privacy**: No central server knows who you are
2. **Simplicity**: No account management, no password resets
3. **Ownership**: You control your identity completely
4. **Offline-first**: Works without any network connection

The sync server is just a relay—it stores documents but has no concept of users or accounts.

## How Multi-Device Sync Works

```
┌─────────────────┐                    ┌─────────────────┐
│    Device A     │                    │    Device B     │
│                 │                    │                 │
│  Root Doc ID:   │                    │  Root Doc ID:   │
│  2DrjgbFkSxX... │                    │  2DrjgbFkSxX... │
│        │        │                    │        │        │
│        ▼        │                    │        ▼        │
│  ┌───────────┐  │                    │  ┌───────────┐  │
│  │ Your Data │  │◄──── Sync ────────►│  │ Your Data │  │
│  └───────────┘  │     Server         │  └───────────┘  │
└─────────────────┘                    └─────────────────┘
```

Both devices have the **same root document ID**, so they sync the **same data**. The sync server just relays changes—it doesn't know or care who owns what.

## Setting Up Multiple Devices

### First Device

```bash
rott init
```

This creates a new root document and displays its ID. **Write this down.**

### Additional Devices

```bash
rott init --join 2DrjgbFkSxXwGBKnVGbLjtPnqTo3r
```

This tells ROTT to use the existing root document instead of creating a new one.

### View Your Root Document ID Anytime

```bash
rott status
```

## What Happens If You Lose Your Root Document ID?

**If you lose your root document ID and don't have it on any device:**

- You cannot access your data from new devices
- Your data still exists on the sync server, but you can't retrieve it
- There is no recovery mechanism—no "forgot password" flow

**If you still have ROTT running on at least one device:**

- Run `rott status` to see your root document ID
- Write it down immediately
- You can still add new devices using `rott init --join`

## Best Practices for Keeping Your ID Safe

### 1. Write It Down Immediately

After running `rott init`, immediately save your root document ID:

```bash
rott status
# Copy the root document ID shown
```

### 2. Store It in Multiple Places

Suggestions:
- Password manager (1Password, Bitwarden, etc.)
- Encrypted note on your phone
- Written on paper in a secure location
- Separate backup file

### 3. Don't Share It

Your root document ID gives full access to your data. Treat it like a password:
- Don't post it publicly
- Don't send it over unencrypted channels
- Don't store it in plain text in shared locations

### 4. Verify You Have It Before Wiping Devices

Before resetting a device or uninstalling ROTT:
1. Confirm you have the root document ID saved elsewhere
2. Or ensure another device still has ROTT configured

## Creating a New Identity

If you want to start fresh (new root document, no existing data):

```bash
# This creates a brand new root document
rott init --new
```

**Warning:** This abandons your existing data. Only do this if:
- You're starting over intentionally
- You lost your old root document ID and have no devices with it

## Common Questions

### Can I change my root document ID?

No. The ID is derived from the document itself. Changing it would mean creating a new document (and losing your data).

### Can someone guess my root document ID?

Extremely unlikely. The ID contains enough randomness (approximately 160 bits) that guessing is computationally infeasible.

### What if someone gets my root document ID?

They could sync your data if they can reach your sync server. This is why the sync server should only be on your private network or behind a VPN.

### Can I have multiple identities?

Yes, but not simultaneously in the same ROTT installation. You'd need separate config directories. This isn't a common use case.

### Is the root document ID sensitive?

Yes, treat it as a secret:
- It's not as sensitive as a password (useless without sync server access)
- But it should still be kept private
- Anyone with the ID + sync server access can read your data

## Summary

| Concept | Explanation |
|---------|-------------|
| Root document | The Automerge document containing all your data |
| Root document ID | Unique identifier for your root document |
| Your identity | IS your root document ID—no separate accounts |
| Multi-device | All devices use the same root document ID |
| Lost ID | Data becomes inaccessible from new devices |
| Best practice | Save your ID immediately and in multiple places |

---

## Related

- [Sync Server Setup](SYNC_SERVER_SETUP.md) - Set up sync between devices
- [Troubleshooting](TROUBLESHOOTING.md) - Common problems and solutions
