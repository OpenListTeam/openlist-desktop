# OpenList Desktop

<div align="center">
  <img src="./app-icon.png" alt="OpenList Desktop" width="128" height="128" />
  
  **A cross-platform desktop application for OpenList with cloud storage integration**

  [![License](https://img.shields.io/badge/license-GPL--3.0-blue.svg)](./LICENSE)
  [![Vue](https://img.shields.io/badge/Vue-3.5.17-green.svg)](https://vuejs.org/)
  [![Tauri](https://img.shields.io/badge/Tauri-2.6.0-orange.svg)](https://tauri.app/)
  [![Rust](https://img.shields.io/badge/Rust-2024-red.svg)](https://www.rust-lang.org/)
  
  [English](./README.md) | [中文](./README_zh.md)
</div>

## WIP

This project is still under development and will soon release version 1.0.

## 🔍 Overview

OpenList Desktop is a powerful cross-platform desktop application that provides a user-friendly interface for managing OpenList services and integrating cloud storage through Rclone. Built with modern web technologies and Rust, it offers seamless file management, cloud storage mounting, and service monitoring capabilities.

The application serves as a comprehensive solution for:

- Managing OpenList file management services
- Mounting and managing cloud storage (WebDAV)
- Monitoring service health and performance
- Providing system tray integration for background operations

## ✨ Features

### 🚀 Core Features

- **OpenList Service Management**: Start, stop, and monitor OpenList core services
- **Cloud Storage Integration**: Mount via Rclone
- **Real-time Monitoring**: Track service status, uptime, and performance metrics
- **Process Management**: Advanced process control with auto-restart capabilities
- **System Tray**: Background operation with system tray notifications

### ⚙️ Management Features

- **Service Control**: Start/stop/restart OpenList and Rclone services
- **Configuration Management**: GUI-based configuration for all services
- **Log Monitoring**: Real-time log viewing and management
- **Update Management**: Automatic update checking and installation
- **Auto-startup**: Configure applications to start with system boot

### 🎨 User Experience

- **Modern UI**: Clean, intuitive interface built with Vue.js
- **Multi-language**: Support for English and Chinese
- **Responsive Design**: Optimized for various screen sizes
- **Keyboard Shortcuts**: Efficient navigation with keyboard shortcuts
- **Tutorial System**: Built-in tutorial for new users

## 📸 Screenshots

### Home Dashboard

![Home Dashboard](./screenshot/homepage.png)

The main dashboard provides a comprehensive overview of your OpenList Desktop environment with:

- Service status monitoring
- Quick action buttons for common tasks
- Version management and update notifications
- Service management controls

### Mount Management

![Mount Management](./screenshot/mountpage.png)

Manage your cloud storage connections with ease:

- Add and configure storage remotes
- Mount/unmount cloud storage
- Monitor mount status and statistics
- Configure auto-mounting options

### Settings Configuration

![Settings](./screenshot/settingpage.png)

Comprehensive settings management:

- OpenList service configuration
- Startup and automation preferences
- Theme and language selection

### Log Monitoring

![Log Monitoring](./screenshot/logpage.png)

Keep track of system operations:

- Real-time log streaming
- Filter logs by source and level
- Export and clear log functionality

### Update Management

![Update Management](./screenshot/update.png)

Stay up-to-date with the latest versions:

- Check for OpenList and Rclone updates
- Download and install updates
- Version history and changelog
- Automatic update scheduling

## 📦 Installation

### Prerequisites

- **Operating System**: Windows 10+, macOS 10.15+, or Linux (Ubuntu 18.04+)

### Download Options

#### 1. GitHub Releases (Recommended)

Download the latest release from [GitHub Releases](https://github.com/OpenListTeam/openlist-desktop/releases):

- **Windows**: `OpenList-Desktop_x.x.x_x64_en-US.msi`
- **macOS**: `OpenList-Desktop_x.x.x_x64.dmg`
- **Linux**: `OpenList-Desktop_x.x.x_amd64.AppImage`

#### 2. Build from Source

```bash
# Clone the repository
git clone https://github.com/OpenListTeam/openlist-desktop.git
cd openlist-desktop

# Install dependencies
npm install

# Prepare development environment
npm run prepare-dev

# Build the application
npm run build
npm run tauri build
```

### Installation Steps

#### Windows

1. Download the `.msi` installer
2. Run the installer as Administrator
3. Follow the installation wizard
4. Launch from Start Menu or Desktop shortcut

#### macOS

1. Download the `.dmg` file
2. Open the DMG and drag OpenList Desktop to Applications
3. Right-click and select "Open" (first time only)
4. Grant necessary permissions when prompted

#### Linux

1. Download the `.AppImage` file
2. Make it executable: `chmod +x OpenList-Desktop*.AppImage`
3. Run the AppImage: `./OpenList-Desktop*.AppImage`
4. Optional: Install using AppImageLauncher for system integration

## 🚀 Usage

### First Launch

1. **Initial Setup**: On first launch, the application will guide you through initial configuration
2. **Service Installation**: Install the OpenList service when prompted
3. **Storage Configuration**: Configure your first cloud storage connection
4. **Tutorial**: Complete the interactive tutorial to learn key features

### Basic Operations

#### Starting Services

```
Dashboard → Service Management → Start OpenList Service
Dashboard → Quick Actions → Start Rclone Backend
```

#### Adding Cloud Storage

1. Navigate to **Mount** tab
2. Click **Add Remote** button
3. Configure storage settings:
   - **Name**: Unique identifier for your storage
   - **Type**: Storage provider (WebDAV)
   - **URL**: Storage endpoint URL
   - **Credentials**: Username and password
   - **Mount Point**: Local directory path
4. Click **Save** and **Mount**

#### Monitoring Operations

- **Service Status**: Check the dashboard for service health indicators
- **Logs**: Use the Logs tab to monitor system operations
- **Performance**: View uptime and response metrics on the dashboard

### Advanced Features

#### Auto-mounting Configuration

```javascript
// Configure storage to mount automatically on startup
{
  "autoMount": true,
  "extraFlags": ["--vfs-cache-mode", "full"],
  "mountPoint": "/mnt/cloudstorage"
}
```

#### Custom Rclone Flags

Add custom Rclone flags for optimal performance:

- `--vfs-cache-mode full`: Enable full VFS caching
- `--buffer-size 256M`: Increase buffer size
- `--transfers 10`: Concurrent transfer limit

#### System Tray Operations

- **Right-click tray icon** for quick actions
- **Double-click** to show/hide main window
- **Service status** indicated by icon color

## ⚙️ Configuration

### Application Settings

#### OpenList Service Configuration

```json
{
  "openlist": {
    "port": 5244,
    "api_token": "your-secure-token",
    "auto_launch": true,
    "ssl_enabled": false
  }
}
```

#### Rclone Configuration

```json
{
  "rclone": {
    "config": {
      "mycloud": {
        "type": "webdav",
        "url": "https://cloud.example.com/dav",
        "user": "username",
        "pass": "encrypted-password",
        "mountPoint": "C:/CloudDrive",
        "autoMount": true,
        "extraFlags": ["--vfs-cache-mode", "full"]
      }
    },
  }
}
```

#### Application Preferences

```json
{
  "app": {
    "theme": "auto",
    "auto_update_enabled": true,
    "monitor_interval": 30000
  }
}
```

### Environment Variables

- `OPENLIST_API_TOKEN`: Override default API token
- `OPENLIST_PORT`: Override default port (5244)
- `RCLONE_CONFIG_DIR`: Custom Rclone configuration directory
- `LOG_LEVEL`: Set logging level (debug, info, warn, error)

## 🔧 Development

### Development Environment Setup

#### Prerequisites

- **Node.js**: v18+ with npm
- **Rust**: Latest stable version
- **Git**: Version control

#### Setup Steps

```bash
# Clone repository
git clone https://github.com/OpenListTeam/openlist-desktop.git
cd openlist-desktop

# Install Node.js dependencies
npm install

# Install Rust dependencies
cd src-tauri
cargo fetch

# Prepare development environment
cd ..
npm run prepare-dev

# Start development server
npm run dev
```

#### Development Commands

```bash
# Start development server with hot reload
npm run dev

# Start development without file watching
npm run nowatch

# Run linting
npm run lint

# Fix linting issues
npm run lint:fix

# Type checking
npm run build --dry-run
```

## 🤝 Contributing

We welcome contributions from the community!
## 📄 License

This project is licensed under the **GNU General Public License v3.0** - see the [LICENSE](./LICENSE) file for details.

---

<div align="center">
  <p>Made with ❤️ by the OpenList Team</p>
  <p>
    <a href="https://github.com/OpenListTeam/openlist-desktop">GitHub</a> •
    <a href="https://openlist.team">Website</a> •
    <a href="https://t.me/OpenListTeam">Telegram</a>
  </p>
</div>
