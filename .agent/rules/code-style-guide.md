---
trigger: always_on
---

# Code Style Guide

This document outlines the coding standards and best practices for the AnyCowork project.

## Rust (Backend)

- **Formatting**: Always use `cargo fmt` to format code.
- **Linting**: Run `cargo clippy` to catch common mistakes and improve code quality.
- **Async I/O**: Use `async/await` for all I/O operations to ensure non-blocking execution.
- **Error Handling**: Use `Result<T, String>` for Tauri commands to properly propagate errors to the frontend.
- **Logging**: Use the `log` crate (`log::info!`, `log::error!`) for application logging.
- **Type Safety**: Update types in both Rust and TypeScript when changing APIs to maintain consistency.

## TypeScript (Frontend)

- **Linting**: Follow the configured ESLint rules.
- **Components**: Use functional components with hooks.
- **State Management**: Use **React Query** for server state and data fetching.
- **Type Safety**: Enable and strictly follow TypeScript strict mode.
- **Imports**: Ensure all imports are used; remove unused ones.

## UI Design

- **Guidelines**: MUST follow `docs/ui-design.md` for all frontend work.
- **Icons**: Use `lucide-react` for icons.
- **Components**: Use `shadcn/ui` components and Radix UI primitives.
- **Styling**: Use Tailwind CSS utilities.
- **Responsiveness**: Implement mobile-first responsive design.

## Development Patterns

### Adding New Features

#### Tauri Commands
1. Define the command in `src-tauri/src/lib.rs` using `#[tauri::command]`.
2. Register the command in the `invoke_handler`.
3. Add the corresponding function to the frontend API client in `src/lib/anycowork-api.ts`.

#### Database Tables
1. Generate a migration: `diesel migration generate <migration_name>`.
2. Edit `up.sql` and `down.sql`.
3. Update `src-tauri/src/schema.rs`.
4. Add the model struct in `src-tauri/src/models.rs`.

#### Frontend Components
1. Prioritize `shadcn/ui` components.
2. Follow patterns in `src/routes/`.
3. Define types in `src/lib/anycowork-api.ts`.

## General Guidelines

- **Security**: Never commit API keys or secrets. Store them in `.env`.
- **Testing**: Test both frontend and backend after making changes.
- **File Access**: Agent tools are restricted to the workspace.
- **Approval**: Critical actions require user approval.
