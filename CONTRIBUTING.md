# Contributing to Nova Launcher

Thank you for your interest in contributing to Nova Launcher!

## Architecture Guidelines

- **Clean Architecture**: Keep domain models free of framework dependencies. The Minecraft installation and launch logic in `src-tauri/src/` must remain independent of Tauri.
- **Port & Adapter Pattern**: New mod loaders or mod providers must implement their respective ports (`ModLoaderInstallerPort`, `ModProviderPort`).
- **No Raw Invokes in UI**: All Tauri command invocations must reside in `src/services/tauri/`.
- **Liquid Glass Aesthetics**: All new components should leverage design tokens from `src/components/ui/glass/` with subtle borders, top highlights, and balanced backdrop blurs.
- **Error Handling**: Use structured `LauncherError` in Rust and display actionable messages in the UI. Internal stack traces should only go to logs.

## Development Workflow

1. Check existing tests:
   ```bash
   cargo test --manifest-path src-tauri/Cargo.toml
   ```
2. Run typecheck and frontend build:
   ```bash
   pnpm build
   ```
3. Start the live dev environment:
   ```bash
   pnpm tauri dev
   ```