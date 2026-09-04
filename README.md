# 🌟 Nova Launcher

> A modern, premium desktop Minecraft launcher inspired by Apple macOS, visionOS, and the Liquid Glass visual language. Built with **Tauri v2**, **React**, **TypeScript**, **Rust**, and **SQLite**, structured using **Clean / Hexagonal Architecture**.

---

## 📸 Overview & Vision

Nova Launcher is designed from the ground up to be modular, robust, and extensible. It manages isolated Minecraft installations, downloads Vanilla assets and libraries with concurrency limits and SHA-1 verification, automatically discovers Java runtimes, and constructs native JVM/game arguments with rule evaluators.

Conceptualized similarly to **Modrinth App** and **CurseForge Desktop**, but built with a distinct, translucent, and minimalist aesthetic inspired by Apple, Raycast, and Linear.

---

## ⚡ Key Features

- **Isolated Instances**: Complete separation between instances (`instances/<id>/game/` with dedicated saves, options, resource packs, and shader packs).
- **Official Version Manifest Client**: Real-time fetching and local caching of Mojang's `version_manifest_v2.json` (Releases, Snapshots, Betas, and Alphas).
- **Concurrent Download Manager**: Bounded concurrency worker pool (6-10 simultaneous downloads) with rolling speed calculations, SHA-1 checksum verification, and live event streaming to the frontend.
- **Robust Minecraft Installer**:
  1. Version JSON parsing
  2. Client JAR download
  3. Rule-evaluated libraries download
  4. Asset index and asset objects download
  5. Native JAR extraction (extracts `.dll` files, skips `META-INF`)
  6. File verification and integrity checks
- **Java Runtime Manager**:
  - Auto-discovery across Windows registry, `JAVA_HOME`, `PATH`, and standard installation folders (`C:\Program Files\Java`, `Eclipse Adoptium`, `Microsoft`, etc.).
  - Probes `java -version` and parses major versions (Java 8, 17, 21, 25).
  - Version compatibility validation.
  - Per-instance or global manual override.
- **Argument Rule Evaluator**:
  - Full evaluation of Minecraft launch rules across Operating Systems (Windows, macOS, Linux), architectures (x86, x64, ARM), and features.
  - Modern (`arguments.jvm`, `arguments.game`) and legacy (`minecraftArguments`) support.
- **Process Supervisor & Streaming Logs**:
  - Spawns Java process asynchronously.
  - Pipes `stdout` and `stderr` in real-time to the frontend and writes to `<game>/logs/latest.log` and `launcher-data/logs/launcher.log`.
  - Tracks total play duration and last played timestamps.
- **Liquid Glass Design System**:
  - Translucent surfaces (`backdrop-filter: blur(24px) saturate(180%)`).
  - Subtle borders, top highlights, depth shadows, rounded corners (16px - 28px).
  - Reusable components: `GlassPanel`, `GlassCard`, `GlassButton`, `GlassModal`, `GlassInput`, `GlassBadge`, `GlassProgress`.
- **Extensible Architecture**:
  - Interfaces and traits prepared for `ModLoaderInstaller` (Fabric, Forge, NeoForge).
  - Interfaces for `ModProvider` (Modrinth, CurseForge).
  - Interfaces for `AuthenticationProvider` (Microsoft OAuth2 + Xbox Live, with development offline mode active).

---

## 🏗️ Architecture

Nova Launcher strictly adheres to **Clean Architecture / Hexagonal Architecture** principles, ensuring that the Minecraft core does not depend on Tauri or any particular presentation framework.

```
src-tauri/src/
├── domain/                    # Pure domain models, value objects, domain errors, repository traits
│   ├── entities/              # Instance, InstanceStatus, ModLoader, MinecraftVersion, JavaRuntime
│   ├── value_objects/         # RamConfig, GameDirectory, VersionId
│   ├── repositories/          # InstanceRepository, SettingsRepository traits
│   └── errors/                # LauncherError with structured error codes
│
├── application/               # Application use cases, DTOs, and outbound ports
│   ├── ports/                 # ManifestClientPort, DownloadManagerPort, JavaDetectorPort, LauncherPort
│   ├── dto/                   # CreateInstanceDto, UpdateInstanceDto, VersionFilterDto, SettingsDto
│   └── use_cases/             # CreateInstance, ListInstances, LaunchInstance, InstallInstance, etc.
│
├── infrastructure/            # Implementations of outbound ports
│   ├── persistence/           # SQLite connection manager, migrations, SqliteInstanceRepository
│   ├── minecraft/             # ManifestClient, ArgumentRuleEvaluator, ArgumentBuilder, Installer, Launcher
│   ├── downloads/             # DownloadManager (Tokio concurrency pool, progress events)
│   ├── java/                  # JavaDetector, JavaVersionParser
│   ├── process/               # ProcessSupervisor (Async Tokio child supervision)
│   ├── logging/               # Tracing, log file appenders, in-memory log buffer
│   └── auth/                  # DevOfflineAuthenticationProvider, MicrosoftAuthProvider placeholder
│
├── presentation/              # Inbound adapters (Tauri Commands)
│   ├── commands/              # instance_commands, minecraft_commands, java_commands, settings_commands, log_commands
│   └── state.rs               # AppState dependency injection container
│
└── shared/                    # Utilities and configuration
    ├── config/                # LauncherPaths
    └── utils/                 # FileVerifier (SHA-1), ZipExtractor (Natives)
```

### Frontend Architecture

```
src/
├── app/                       # App router, TanStack Query provider, global event listeners
├── components/ui/glass/       # Liquid Glass component library
├── features/
│   ├── home/                  # Hero card, recent instances grid, quick play
│   ├── instances/             # Instance grid, 5-step create wizard modal, instance detail view
│   ├── minecraft/             # Version queries & filters
│   ├── downloads/             # Real-time download manager view
│   ├── settings/              # General, Minecraft, Java, Appearance, Downloads, Advanced tabs
│   └── logs/                  # Live filterable terminal (ERROR, WARN, INFO, DEBUG)
├── layouts/                   # MainLayout with GlassSidebar and top translucent header
├── services/tauri/            # Decoupled API calls (no raw invoke in components)
├── store/                     # Lightweight Zustand stores (App, Downloads)
└── styles/                    # Tailwind CSS & Liquid Glass tokens
```

---

## 🚀 Getting Started

### Prerequisites

- **Node.js**: v18+ (tested on Node v24)
- **pnpm**: v9+ (or v11)
- **Rust & Cargo**: v1.75+ (tested on Rust 1.98)
- **Java**: Java 21 or Java 17 for running Minecraft (LTS recommended)

### Installation

1. Clone or navigate to the repository:
   ```bash
   cd d:/Launcher-Minecraft
   ```

2. Install frontend dependencies:
   ```bash
   pnpm install
   ```

3. Run automated backend tests:
   ```bash
   cargo test --manifest-path src-tauri/Cargo.toml
   ```

4. Build frontend bundle:
   ```bash
   pnpm build
   ```

5. Run in development mode:
   ```bash
   pnpm tauri dev
   ```

6. Build release desktop application:
   ```bash
   pnpm tauri build
   ```

---

## 🧪 Testing

Nova Launcher includes comprehensive unit and integration tests for all critical business logic:

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

Test suites include:
- `instance_tests`: Verifies instance creation, SQLite persistence, retrieval, updating, deletion, and isolated filesystem directories.
- `download_tests`: Verifies SHA-1 file verification against valid and corrupted files.
- `rule_evaluator_tests`: Verifies argument rules across OS, architecture, and feature flags.
- `java_parser_tests`: Verifies output parsing across Oracle, OpenJDK, Temurin, and legacy Java 8 formats.
- `argument_builder_tests`: Verifies classpath assembly and placeholder substitution (`${auth_player_name}`, `${version_name}`, `${classpath}`).

---

## 🗺️ Roadmap

- [x] Phase 1: Clean Architecture & Scaffolding
- [x] Phase 2: Persistence & Instance Management (SQLite)
- [x] Phase 3: Official Minecraft Version Manifest Client
- [x] Phase 4: Concurrent Download Manager with SHA-1 verification
- [x] Phase 5: Minecraft Vanilla Installer & Native Extraction
- [x] Phase 6: Java Runtime Manager & Compatibility Validator
- [x] Phase 7: Process Supervisor & Argument Rule Evaluator
- [x] Phase 8: Liquid Glass UI Component Library
- [x] Phase 9: Full Frontend Pages Integration
- [ ] Phase 10: Fabric, Forge, and NeoForge mod loader integration
- [ ] Phase 11: Modrinth & CurseForge mod/modpack browser
- [ ] Phase 12: Microsoft OAuth2 & Xbox Live authentication

---

## 📄 License

MIT License. Developed for the next generation of Minecraft players.