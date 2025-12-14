# Getting Started - Your First Day
## Build Your First Browser Component in 30 Minutes

---

## 🎯 Goal

By the end of this guide, you will have:
- ✅ Set up your development environment
- ✅ Implemented your first component (Tab Manager)
- ✅ Run tests successfully
- ✅ Understood the development workflow

**Time Required**: 30-60 minutes

---

## Step 1: Verify Your Setup (5 minutes)

### Check Rust Installation
```bash
# Should show version 1.70+ or higher
rustc --version
cargo --version

# If not installed:
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Check Node.js Installation
```bash
# Should show version 18+ or higher
node --version
npm --version

# If not installed, download from: https://nodejs.org/
```

### Verify Project Builds
```bash
cd /path/to/your/virtual-ip-browser
cargo build

# Should compile successfully (may take 5-10 minutes first time)
```

### Install System Dependencies (Linux Only)
```bash
sudo apt-get update
sudo apt-get install -y \
    libwebkit2gtk-4.0-dev \
    build-essential \
    curl \
    wget \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev
```

---

## Step 2: Open Project in IntelliJ IDEA (5 minutes)

### Launch IntelliJ IDEA Ultimate
```
1. File → Open
2. Navigate to your project directory
3. Click "Open"
4. Wait for Rust plugin to index (progress bar at bottom)
```

### Configure Windsurf
```
1. File → Settings (or Preferences on macOS)
2. Search for "Windsurf"
3. If not installed:
   - Go to Plugins
   - Search "Windsurf"
   - Install and restart
4. Configure API key for Claude Opus 4.5
```

### Verify Setup
```
1. Open any .rs file
2. Check that syntax highlighting works
3. Try Ctrl+Space for autocomplete
4. Open Windsurf panel (Ctrl+Shift+A or toolbar icon)
```

---

## Step 3: Implement Your First Component (15 minutes)

### Component: Tab Manager

**File to Edit**: `crates/browser-core/src/tab_manager.rs`

### Open the File
```
1. Navigate to: crates/browser-core/src/tab_manager.rs
2. You'll see it's mostly empty or has basic structure
3. Open Windsurf AI assistant (Ctrl+Shift+A)
```

### Use This Prompt in Windsurf

Copy and paste this into Windsurf:

```
I need to implement a comprehensive tab management system for a privacy-focused browser built with Rust and Tauri.

FILE: crates/browser-core/src/tab_manager.rs

REQUIREMENTS:
1. Create a Tab struct with these fields:
   - id: TabId (String wrapper type)
   - title: String
   - url: String
   - favicon: Option<String>
   - loading: bool
   - can_go_back: bool
   - can_go_forward: bool
   - proxy_id: Option<ProxyId>
   - virtual_ip: Option<IpAddr>
   - created_at: DateTime<Utc>
   - last_active: DateTime<Utc>
   - is_pinned: bool
   - is_suspended: bool

2. Create a TabManager struct with these fields:
   - tabs: Vec<Tab>
   - active_tab_index: usize
   - max_tabs: usize

3. Implement these methods:
   - new() -> Self
   - create_tab(url: &str) -> Result<TabId>
   - close_tab(tab_id: TabId) -> Result<()>
   - get_tab(tab_id: TabId) -> Option<&Tab>
   - get_tab_mut(tab_id: TabId) -> Option<&mut Tab>
   - switch_to_tab(tab_id: TabId) -> Result<()>
   - get_active_tab() -> Option<&Tab>
   - get_all_tabs() -> &Vec<Tab>
   - update_tab_url(tab_id: TabId, url: String) -> Result<()>
   - update_tab_title(tab_id: TabId, title: String) -> Result<()>
   - pin_tab(tab_id: TabId) -> Result<()>
   - unpin_tab(tab_id: TabId) -> Result<()>

4. Use these dependencies:
   - serde for serialization
   - chrono for timestamps
   - anyhow for error handling
   - std::net::IpAddr for IP addresses

5. Create wrapper types:
   - TabId as a newtype around String
   - ProxyId as a newtype around String

6. Add proper error handling for:
   - Tab not found
   - Max tabs exceeded
   - Invalid tab index

7. Include comprehensive inline documentation
8. Make all necessary types serializable with serde
9. Use proper Rust idioms and best practices

Please provide the complete implementation with all necessary imports and type definitions.
```

### Review the Generated Code

Windsurf will generate something like this:

```rust
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;

// Type wrappers
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TabId(String);

impl TabId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

// ... rest of implementation
```

### Accept or Refine
- Review the code
- If something is missing, ask Windsurf to add it
- If you see errors, ask Windsurf to fix them

---

## Step 4: Add Missing Dependencies (5 minutes)

The Tab Manager needs some dependencies. Add them to `crates/browser-core/Cargo.toml`:

### Open File
```
crates/browser-core/Cargo.toml
```

### Add These Dependencies
```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.6", features = ["v4", "serde"] }
```

### Rebuild
```bash
cargo build -p browser-core
```

---

## Step 5: Create a Basic Test (5 minutes)

### Ask Windsurf to Generate Tests

In Windsurf, type:

```
Now create comprehensive unit tests for the TabManager implementation in a tests module.

Include tests for:
1. Creating a new tab
2. Closing a tab
3. Switching between tabs
4. Getting active tab
5. Updating tab properties
6. Pinning/unpinning tabs
7. Error cases (tab not found, max tabs exceeded)

Use the standard Rust testing framework with #[cfg(test)] and #[test].
```

### Add Tests to the File

Windsurf will generate something like:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_tab() {
        let mut manager = TabManager::new();
        let tab_id = manager.create_tab("https://example.com").unwrap();
        
        assert_eq!(manager.get_all_tabs().len(), 1);
        
        let tab = manager.get_tab(tab_id).unwrap();
        assert_eq!(tab.url, "https://example.com");
    }
    
    // ... more tests
}
```

---

## Step 6: Run Your Tests (2 minutes)

### In Terminal
```bash
# Run tests for browser-core
cargo test -p browser-core

# Run with output
cargo test -p browser-core -- --nocapture

# Run specific test
cargo test -p browser-core test_create_tab
```

### Expected Output
```
running 7 tests
test tab_manager::tests::test_create_tab ... ok
test tab_manager::tests::test_close_tab ... ok
test tab_manager::tests::test_switch_tab ... ok
test tab_manager::tests::test_get_active_tab ... ok
test tab_manager::tests::test_update_tab ... ok
test tab_manager::tests::test_pin_tab ... ok
test tab_manager::tests::test_max_tabs ... ok

test result: ok. 7 passed; 0 failed; 0 ignored
```

### If Tests Fail
- Read the error message
- Ask Windsurf: "This test is failing with error: [ERROR]. Help me fix it."
- Update code and re-run

---

## Step 7: Commit Your Work (2 minutes)

```bash
git add crates/browser-core/src/tab_manager.rs
git add crates/browser-core/Cargo.toml
git commit -m "feat: implement tab manager with tests"
```

---

## 🎉 Congratulations!

You've just:
- ✅ Set up your development environment
- ✅ Used Windsurf with Claude Opus 4.5 to generate code
- ✅ Implemented a complete, tested component
- ✅ Learned the development workflow

---

## Next Steps

### Tomorrow: Implement WebView Manager
```
1. Open DEVELOPMENT_PHASES.md
2. Go to Phase 1, Section 1.1
3. Copy the WebView Manager prompt
4. Paste into Windsurf
5. Implement and test
```

### This Week: Complete Phase 1
Follow the **Week 1 Roadmap** in **QUICKSTART_GUIDE.md**:
- Monday: Tab & WebView Management ✅ (Done!)
- Tuesday: Browser Controls
- Wednesday: Integration
- Thursday: Tauri Backend
- Friday: Basic UI

---

## 💡 Tips for Success

### 1. Use Windsurf Effectively
- Keep prompts focused on one component
- Ask for tests after implementation
- Request code reviews for complex logic

### 2. Test Continuously
```bash
# After every change
cargo test

# Check code quality
cargo clippy
cargo fmt
```

### 3. Commit Often
```bash
# After each working component
git add .
git commit -m "descriptive message"
```

### 4. Don't Get Stuck
- If something doesn't work, ask Windsurf
- Check the documentation references
- Take breaks - fresh eyes help

### 5. Track Your Progress
Open **IMPLEMENTATION_CHECKLIST.md** and check off:
- [x] Tab Manager implemented
- [ ] WebView Manager (next)
- [ ] Browser Controls
- ...

---

## 🆘 Troubleshooting

### Build Errors
```bash
# Clear and rebuild
cargo clean
cargo build

# Update dependencies
cargo update
```

### Import Errors
Make sure all dependencies are in Cargo.toml:
```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
anyhow = "1.0"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.6", features = ["v4", "serde"] }
```

### Windsurf Not Responding
- Reduce context (close extra files)
- Break prompts into smaller pieces
- Restart IntelliJ IDEA

### Tests Failing
- Read error messages carefully
- Ask Windsurf to fix: "This test fails with: [ERROR]"
- Add debug prints: `println!("{:?}", value);`

---

## 📚 Quick Reference

### Key Documents
- **QUICKSTART_GUIDE.md** - Week-by-week roadmap
- **DEVELOPMENT_PHASES.md** - All implementation prompts
- **WINDSURF_USAGE_GUIDE.md** - How to use Windsurf
- **IMPLEMENTATION_CHECKLIST.md** - Track progress

### Key Commands
```bash
# Build
cargo build

# Test
cargo test

# Format code
cargo fmt

# Lint
cargo clippy

# Run app
cargo run
```

### Next Component Prompt Location
**DEVELOPMENT_PHASES.md** → **Phase 1** → **Section 1.1** (WebView Manager)

---

## 🚀 You're Ready!

You now understand the complete workflow:
1. Choose component from checklist
2. Find prompt in phase documents
3. Use Windsurf to generate code
4. Test and refine
5. Commit and move to next component

**Keep going! You're building something amazing!** 🎯

