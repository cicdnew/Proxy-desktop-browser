# Windsurf Plugin Usage Guide for IntelliJ IDEA Ultimate
## How to Use Claude Opus 4.5 Prompts for Browser Development

---

## Setup Instructions

### 1. Install Windsurf Plugin in IntelliJ IDEA

1. Open IntelliJ IDEA Ultimate
2. Go to **Settings/Preferences** → **Plugins**
3. Search for "Windsurf" in the Marketplace
4. Click **Install** and restart IDE
5. Configure Windsurf with your Claude API key

### 2. Configure Claude Opus 4.5

1. Open Windsurf settings in IntelliJ
2. Select **Claude Opus 4.5** as the model
3. Set temperature to **0.7** for balanced creativity/accuracy
4. Enable **Long Context Mode** for complex implementations

---

## How to Use the Development Prompts

### Method 1: Sequential Implementation

**Start with Phase 1, then proceed through phases in order:**

1. Open the file you want to implement (e.g., `webview_manager.rs`)
2. Open Windsurf AI assistant (usually Ctrl+Shift+A or Cmd+Shift+A)
3. Copy the entire prompt from the relevant phase document
4. Paste into Windsurf chat
5. Review the generated code
6. Accept or modify as needed
7. Proceed to next component

**Example Workflow:**
```
Day 1: Phase 1 - Core Browser Engine
  → WebView Manager
  → Tab Manager  
  → Browser Controls

Day 2: Phase 2 - Proxy Integration
  → HTTP Proxy Implementation
  → Network Interceptor
  → Virtual IP Rotation

Day 3-4: Phase 3 - Free Proxy Providers
  → Provider Abstraction
  → Validator
  → Database

Day 5-6: Phase 4-5 - UI Components
  → Tab Bar
  → Address Bar
  → Settings Panel
  → Status Bar

Day 7: Phase 6 - Advanced Features
  → Cookie Isolation
  → Fingerprinting
  → Download Manager

Day 8: Phase 7 - Testing & Security

Day 9-10: Phase 8 - Deployment
```

---

## Method 2: Feature-by-Feature Implementation

**Implement complete features across multiple files:**

### Feature: Tab Management
```
1. Open DEVELOPMENT_PHASES.md → Section 1.2
2. Copy Tab Manager prompt
3. Implement in webview_manager.rs
4. Then implement UI in TabBar.svelte
5. Test the complete feature
```

### Feature: Proxy Routing
```
1. Open DEVELOPMENT_PHASES.md → Section 2.1-2.3
2. Implement proxy.rs
3. Implement http_client.rs  
4. Implement rotation.rs
5. Test proxy switching
```

---

## Method 3: Using Windsurf's Multi-File Context

Windsurf can handle multiple files simultaneously:

1. **Select multiple files** in project tree
2. Right-click → **Windsurf** → **Add to Context**
3. Use this compound prompt:

```
I have the following files in context:
- crates/browser-core/src/proxy.rs
- crates/browser-core/src/http_client.rs
- crates/virtual-ip/src/rotation.rs

Using the prompts from DEVELOPMENT_PHASES.md Phase 2, implement:
1. Complete proxy connection system in proxy.rs
2. HTTP interceptor in http_client.rs
3. Rotation logic in rotation.rs

Ensure they work together seamlessly with proper error handling and async support.
```

---

## Best Practices for Windsurf + Claude

### 1. Context Management
- Keep prompts focused (one component at a time)
- Include relevant type definitions in context
- Reference existing code when implementing related features

### 2. Iterative Refinement
```
First Prompt: "Implement basic WebView manager as described in Phase 1.1"
Second Prompt: "Add error handling and logging to the WebView manager"
Third Prompt: "Add unit tests for WebView manager"
```

### 3. Code Review Workflow
```
After implementation:
"Review the WebView manager implementation for:
1. Memory safety issues
2. Potential panics
3. Missing error handling
4. Performance bottlenecks"
```

---

## Quick Reference: Prompt Templates

### Template 1: New Component
```
Implement [COMPONENT_NAME] as described in [PHASE_FILE] section [SECTION_NUMBER].

Requirements:
- Follow the exact structure provided
- Use the specified dependencies
- Include comprehensive error handling
- Add inline documentation
- Make it async-compatible with tokio

File: [FILE_PATH]
```

### Template 2: Integration
```
I have implemented:
- [Component A] in [file A]
- [Component B] in [file B]

Now help me integrate them according to [PHASE_FILE] section [SECTION].
Ensure proper error propagation and event handling.
```

### Template 3: Testing
```
Create comprehensive tests for [COMPONENT_NAME] covering:
1. Happy path scenarios
2. Error cases
3. Edge cases
4. Integration with [OTHER_COMPONENT]

Use the test structure from PHASE_7_TESTING_SECURITY.md
```

### Template 4: UI Component
```
Implement [COMPONENT_NAME] Svelte component as described in [PHASE_FILE].

Requirements:
- TypeScript for type safety
- Responsive design
- Dark mode support
- Proper event handling
- IPC communication with Rust backend

File: [FILE_PATH]
```

---

## Handling Large Implementations

For complex features, break prompts into smaller chunks:

### Example: Proxy System (Large)

**Chunk 1: Data Structures**
```
From DEVELOPMENT_PHASES.md Phase 2.1, implement only the data structures:
- ProxyConfig
- ProxyType enum  
- ProxyManager struct

File: crates/browser-core/src/proxy.rs
```

**Chunk 2: Connection Logic**
```
Now implement the connection methods:
- connect()
- disconnect()
- check_health()

Use the ProxyManager struct from previous implementation.
```

**Chunk 3: Error Handling**
```
Add comprehensive error handling using the BrowserError types from PHASE_7_TESTING_SECURITY.md
```

---

## Debugging with Windsurf

When code doesn't work:

```
The [COMPONENT] implementation is failing with error: [ERROR_MESSAGE]

Current code:
[PASTE CODE]

Expected behavior:
[DESCRIBE]

Help me debug and fix this issue.
```

---

## Progressive Enhancement Strategy

### Week 1: Minimal Viable Browser
- Basic tab management
- Simple navigation
- Single proxy support

### Week 2: Proxy Features
- Multiple proxy support
- Basic rotation
- Provider integration (1-2 sources)

### Week 3: Advanced Features
- Cookie isolation
- Fingerprinting protection
- Download manager

### Week 4: Polish & Deploy
- Complete UI
- Testing
- Build & distribution

