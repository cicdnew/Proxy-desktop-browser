# Phase 8: Deployment & Distribution

## 8.1 Build Configuration

**File**: `Cargo.toml` (workspace root)

**Claude Opus 4.5 Prompt:**
```
Configure the Rust workspace for optimized production builds with proper dependencies and features.

UPDATE CARGO.TOML:
```toml
[workspace]
members = [
    "crates/api-server",
    "crates/browser-core",
    "crates/virtual-ip",
]
resolver = "2"

[workspace.package]
version = "1.0.0"
authors = ["Your Name <your.email@example.com>"]
edition = "2021"
license = "MIT OR Apache-2.0"
repository = "https://github.com/yourusername/virtual-ip-browser"

[workspace.dependencies]
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
reqwest = { version = "0.11", features = ["json", "socks", "cookies"] }
anyhow = "1.0"
thiserror = "1.0"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.6", features = ["v4", "serde"] }
rusqlite = { version = "0.30", features = ["bundled"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
panic = "abort"

[profile.dev]
opt-level = 0
debug = true
```

TAURI CONFIGURATION:
**File**: `ui-tauri/tauri.conf.json`
```json
{
  "build": {
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build",
    "devPath": "http://localhost:5173",
    "distDir": "../dist"
  },
  "package": {
    "productName": "Virtual IP Browser",
    "version": "1.0.0"
  },
  "tauri": {
    "allowlist": {
      "all": false,
      "shell": {
        "all": false,
        "open": true
      },
      "protocol": {
        "all": false,
        "asset": true,
        "assetScope": ["**"]
      },
      "fs": {
        "all": false,
        "readFile": true,
        "writeFile": true,
        "readDir": true,
        "createDir": true,
        "scope": ["$APPDATA/*", "$DOWNLOAD/*"]
      },
      "http": {
        "all": true,
        "request": true,
        "scope": ["http://**", "https://**"]
      }
    },
    "bundle": {
      "active": true,
      "category": "Utility",
      "copyright": "Copyright © 2024",
      "deb": {
        "depends": []
      },
      "externalBin": [],
      "icon": [
        "icons/32x32.png",
        "icons/128x128.png",
        "icons/128x128@2x.png",
        "icons/icon.icns",
        "icons/icon.ico"
      ],
      "identifier": "com.virtualipbrowser.app",
      "longDescription": "Privacy-focused browser with virtual IP and proxy routing",
      "macOS": {
        "entitlements": null,
        "exceptionDomain": "",
        "frameworks": [],
        "providerShortName": null,
        "signingIdentity": null
      },
      "resources": [],
      "shortDescription": "Virtual IP Browser",
      "targets": ["msi", "deb", "dmg", "app"],
      "windows": {
        "certificateThumbprint": null,
        "digestAlgorithm": "sha256",
        "timestampUrl": ""
      }
    },
    "security": {
      "csp": "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'"
    },
    "updater": {
      "active": true,
      "endpoints": [
        "https://releases.virtualipbrowser.com/{{target}}/{{arch}}/{{current_version}}"
      ],
      "dialog": true,
      "pubkey": "YOUR_PUBLIC_KEY_HERE"
    },
    "windows": [
      {
        "fullscreen": false,
        "height": 800,
        "resizable": true,
        "title": "Virtual IP Browser",
        "width": 1200,
        "minHeight": 600,
        "minWidth": 800
      }
    ]
  }
}
```

Implement proper build configuration for all platforms.
```

---

## 8.2 Manual Build Process

**Build Commands for Each Platform:**

**Windows:**
```bash
# Install dependencies
cargo install tauri-cli --version ^1.0.0

# Build frontend
cd ui-tauri
npm install
npm run build

# Build application
cd src-tauri
cargo tauri build --target x86_64-pc-windows-msvc
```

**macOS:**
```bash
# Install dependencies
brew install node
xcode-select --install

cargo install tauri-cli --version ^1.0.0

# Build frontend
cd ui-tauri
npm install
npm run build

# Build application (Intel)
cd src-tauri
cargo tauri build --target x86_64-apple-darwin

# Build application (Apple Silicon)
cargo tauri build --target aarch64-apple-darwin
```

**Linux:**
```bash
# Install dependencies
sudo apt-get update
sudo apt-get install -y libwebkit2gtk-4.0-dev \
    build-essential curl wget libssl-dev \
    libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev

cargo install tauri-cli --version ^1.0.0

# Build frontend
cd ui-tauri
npm install
npm run build

# Build application
cd src-tauri
cargo tauri build --target x86_64-unknown-linux-gnu
```

### Build Verification
After building, verify:
- Application launches correctly
- All features work as expected
- Bundle size is reasonable (< 100MB)
- Code signing works (if configured)

### Release Checklist
- [ ] Run full test suite
- [ ] Update version numbers
- [ ] Build on all platforms
- [ ] Test installers
- [ ] Verify update mechanism
- [ ] Create release notes

---

## 8.3 Auto-Update System

**File**: `crates/browser-core/src/updater.rs`

**Claude Opus 4.5 Prompt:**
```
Implement an auto-update system using Tauri's built-in updater.

IMPLEMENTATION:
```rust
use tauri::updater::UpdateResponse;
use tauri::{AppHandle, Manager};

pub struct UpdateManager {
    app_handle: AppHandle,
    check_interval: Duration,
    auto_install: bool,
}

impl UpdateManager {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            check_interval: Duration::from_secs(3600), // 1 hour
            auto_install: false,
        }
    }
    
    pub async fn check_for_updates(&self) -> Result<Option<UpdateResponse>> {
        let update_response = self.app_handle.updater().check().await?;
        
        if update_response.is_update_available() {
            info!(
                "Update available: {} -> {}",
                update_response.current_version(),
                update_response.latest_version()
            );
            
            // Notify UI
            self.app_handle.emit_all("update-available", json!({
                "current_version": update_response.current_version(),
                "latest_version": update_response.latest_version(),
                "release_notes": update_response.body(),
                "release_date": update_response.date(),
            }))?;
            
            Ok(Some(update_response))
        } else {
            info!("No updates available");
            Ok(None)
        }
    }
    
    pub async fn download_and_install(&self, update: UpdateResponse) -> Result<()> {
        info!("Downloading update...");
        
        // Notify UI about download progress
        self.app_handle.emit_all("update-download-started", json!({
            "version": update.latest_version()
        }))?;
        
        update.download_and_install().await?;
        
        info!("Update installed successfully");
        
        // Notify UI to restart
        self.app_handle.emit_all("update-ready", json!({
            "version": update.latest_version(),
            "requires_restart": true
        }))?;
        
        Ok(())
    }
    
    pub async fn start_auto_check(&self) {
        let mut interval = tokio::time::interval(self.check_interval);
        
        loop {
            interval.tick().await;
            
            match self.check_for_updates().await {
                Ok(Some(update)) if self.auto_install => {
                    if let Err(e) = self.download_and_install(update).await {
                        error!("Failed to install update: {}", e);
                    }
                }
                Err(e) => {
                    error!("Failed to check for updates: {}", e);
                }
                _ => {}
            }
        }
    }
}

// Tauri command handlers
#[tauri::command]
pub async fn check_updates(app: AppHandle) -> Result<Option<UpdateInfo>, String> {
    let manager = UpdateManager::new(app);
    manager.check_for_updates().await
        .map(|opt| opt.map(|u| UpdateInfo {
            version: u.latest_version().to_string(),
            notes: u.body().map(|s| s.to_string()),
            date: u.date().map(|d| d.to_string()),
        }))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn install_update(app: AppHandle) -> Result<(), String> {
    let manager = UpdateManager::new(app);
    
    if let Some(update) = manager.check_for_updates().await.map_err(|e| e.to_string())? {
        manager.download_and_install(update).await.map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("No updates available".to_string())
    }
}

#[derive(Serialize)]
pub struct UpdateInfo {
    pub version: String,
    pub notes: Option<String>,
    pub date: Option<String>,
}
```

UI COMPONENT FOR UPDATES:
**File**: `ui-tauri/src/components/UpdateNotification.svelte`
```svelte
<script lang="ts">
  import { invoke } from '@tauri-apps/api/tauri';
  import { listen } from '@tauri-apps/api/event';
  import { onMount } from 'svelte';
  
  let updateAvailable = false;
  let updateInfo: any = null;
  let isDownloading = false;
  let showNotification = false;
  
  onMount(async () => {
    // Listen for update events
    await listen('update-available', (event) => {
      updateInfo = event.payload;
      updateAvailable = true;
      showNotification = true;
    });
    
    await listen('update-download-started', () => {
      isDownloading = true;
    });
    
    await listen('update-ready', () => {
      isDownloading = false;
      showRestartPrompt();
    });
    
    // Check for updates on startup
    checkForUpdates();
  });
  
  async function checkForUpdates() {
    try {
      const update = await invoke('check_updates');
      if (update) {
        updateInfo = update;
        updateAvailable = true;
        showNotification = true;
      }
    } catch (error) {
      console.error('Failed to check for updates:', error);
    }
  }
  
  async function installUpdate() {
    try {
      await invoke('install_update');
    } catch (error) {
      console.error('Failed to install update:', error);
    }
  }
  
  function showRestartPrompt() {
    if (confirm('Update installed! Restart now to apply changes?')) {
      window.__TAURI__.process.relaunch();
    }
  }
</script>

{#if showNotification && updateAvailable}
  <div class="update-notification">
    <div class="update-content">
      <h3>Update Available</h3>
      <p>Version {updateInfo.version} is available</p>
      
      {#if updateInfo.notes}
        <details>
          <summary>Release Notes</summary>
          <div class="release-notes">{updateInfo.notes}</div>
        </details>
      {/if}
      
      {#if isDownloading}
        <p>Downloading update...</p>
        <div class="progress-bar">
          <div class="progress-fill"></div>
        </div>
      {:else}
        <div class="update-actions">
          <button on:click={installUpdate}>Install Update</button>
          <button on:click={() => showNotification = false}>Later</button>
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .update-notification {
    position: fixed;
    bottom: 20px;
    right: 20px;
    background: var(--bg-primary);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    padding: 16px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    z-index: 10000;
    max-width: 400px;
  }
  
  .update-actions {
    display: flex;
    gap: 8px;
    margin-top: 12px;
  }
</style>
```

Implement with proper error handling and user experience considerations.
```

---

## 8.4 Installation & Distribution

**Claude Opus 4.5 Prompt:**
```
Create installation guides and distribution packages for all platforms.

### Windows Installer (MSI)

**File**: `installer/windows/wix.json`
```json
{
  "product_name": "Virtual IP Browser",
  "version": "1.0.0",
  "manufacturer": "Your Company",
  "license": "LICENSE.rtf",
  "banner": "installer/banner.bmp",
  "dialog": "installer/dialog.bmp",
  "upgrade_guid": "YOUR-GUID-HERE",
  "shortcuts": [
    {
      "name": "Virtual IP Browser",
      "target": "[INSTALLDIR]virtual-ip-browser.exe",
      "working_directory": "[INSTALLDIR]",
      "icon": "icons/icon.ico",
      "description": "Launch Virtual IP Browser"
    }
  ],
  "registry_keys": [
    {
      "root": "HKCU",
      "key": "Software\\VirtualIPBrowser",
      "name": "InstallPath",
      "value": "[INSTALLDIR]"
    }
  ]
}
```

### macOS DMG

**Script**: `scripts/create-dmg.sh`
```bash
#!/bin/bash

APP_NAME="Virtual IP Browser"
VERSION="1.0.0"
DMG_NAME="${APP_NAME}-${VERSION}.dmg"
SOURCE_APP="target/release/bundle/macos/${APP_NAME}.app"
DMG_FOLDER="dmg_build"

# Create DMG folder structure
mkdir -p "$DMG_FOLDER"
cp -R "$SOURCE_APP" "$DMG_FOLDER/"
ln -s /Applications "$DMG_FOLDER/Applications"

# Create DMG
hdiutil create -volname "$APP_NAME" -srcfolder "$DMG_FOLDER" -ov -format UDZO "$DMG_NAME"

# Cleanup
rm -rf "$DMG_FOLDER"

echo "DMG created: $DMG_NAME"
```

### Linux (DEB/AppImage)

**File**: `installer/linux/control`
```
Package: virtual-ip-browser
Version: 1.0.0
Section: web
Priority: optional
Architecture: amd64
Depends: libwebkit2gtk-4.0-37, libgtk-3-0
Maintainer: Your Name <your.email@example.com>
Description: Privacy-focused browser with virtual IP routing
 Virtual IP Browser is a privacy-focused web browser that routes
 all traffic through virtual IPs and proxy servers.
```

### Documentation

**File**: `INSTALLATION.md`
```markdown
# Installation Guide

## Windows

1. Download `VirtualIPBrowser-Setup-1.0.0.msi`
2. Double-click to run the installer
3. Follow the installation wizard
4. Launch from Start Menu

## macOS

1. Download `VirtualIPBrowser-1.0.0.dmg`
2. Open the DMG file
3. Drag "Virtual IP Browser" to Applications
4. Launch from Applications folder
5. If blocked by Gatekeeper, go to System Preferences > Security & Privacy

## Linux (Ubuntu/Debian)

```bash
wget https://releases.virtualipbrowser.com/linux/virtual-ip-browser_1.0.0_amd64.deb
sudo dpkg -i virtual-ip-browser_1.0.0_amd64.deb
sudo apt-get install -f  # Fix dependencies if needed
```

## Linux (AppImage)

```bash
wget https://releases.virtualipbrowser.com/linux/VirtualIPBrowser-1.0.0-x86_64.AppImage
chmod +x VirtualIPBrowser-1.0.0-x86_64.AppImage
./VirtualIPBrowser-1.0.0-x86_64.AppImage
```

## Build from Source

```bash
git clone https://github.com/yourusername/virtual-ip-browser.git
cd virtual-ip-browser
cargo build --release
```
```

Implement comprehensive installation and distribution for all target platforms.
```

