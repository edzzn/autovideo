# AutoVideo AGENTS.md

## Build & Development Commands

### Frontend Development
```bash
# Start development server with hot reload
pnpm tauri dev

# Build frontend for production
pnpm build

# Preview production build
pnpm preview

# Lint frontend code
pnpm run lint

# Type check frontend code
pnpm run typecheck

# Run frontend tests
pnpm test
```

### Backend Development
```bash
# Build Rust backend
cd src-tauri
cargo build

# Run Rust tests
cargo test

# Check Rust code (no build)
cargo check

# Format Rust code
cargo fmt

# Lint Rust code
cargo clippy
```

### Single Test Execution
```bash
# Run specific Rust test file
cargo test --test filename

# Run specific test function
cargo test test_function_name

# Run tests with output
cargo test -- --nocapture

# Run test matching pattern
cargo test silence

# Run only failing tests
cargo test -- --broken

# Test in release mode (faster, no optimization)
cargo test --release
```

---

## Code Style Guidelines

### General Principles

**Do not add any comments in code** — code should be self-explanatory through clear naming and structure.

### Frontend (SvelteKit + Svelte)

**Imports**
- Group imports in logical sections: external → internal → relative
- Use wildcard imports only for re-exports
- Keep imports clean and minimal

**Formatting**
- Use consistent 2-space indentation
- Group related elements: CSS variables → mix → body → utility classes
- Maintain clean separation between styled and unstyled content
- For Tailwind, group utility classes by purpose (layout → spacing → typography → effects)

**Types & Props**
- Use TypeScript for all type definitions
- Define explicit interfaces for component props
- Prefer functional components with destructured props
- Type children as `ReactElement` or `SvelteComponent`
- Use optional props with `?` and provide defaults

**Naming Conventions**
- Components: PascalCase (e.g., `PipelineProgress`, `DropZone`)
- Functions: camelCase (e.g., `formatDuration`, `handleDrop`)
- Constants: UPPER_SNAKE_CASE (e.g., `MAX_FILE_SIZE`)
- Variables: camelCase (e.g., `filePath`, `isProcessing`)
- File names: kebab-case for components (e.g., `pipeline-progress.svelte`)

**State Management**
- Use Svelte stores (`writable`, `readable`) for global state
- Keep store logic focused and isolated
- Use derived stores for computed values
- Avoid inline styles when possible — use CSS classes

**Error Handling**
- Use try/catch blocks for async operations
- Handle errors gracefully with user-friendly messages
- Log errors to console (frontend) or proper logging (backend)
- Never expose sensitive information in error messages to users
- Use TypeScript union types for error handling (`type ErrorType = 'file-not-found' | 'transcription-failed' | ...`)

**Component Structure**
```svelte
<script lang="ts">
  // Imports
  // Type definitions
  // Props
  // Reactive declarations
  // Helper functions
  // Event handlers
  // Lifecycle hooks
</script>

<!-- Template -->
{#if condition}
  <div class="...">
    <!-- Content -->
  </div>
{/if}

<!-- Style block if needed -->
<style lang="css">
  .class-name {
    /* Styles */
  }
</style>
```

### Backend (Rust)

**Formatting**
- Use `cargo fmt` for consistent formatting
- Prefer idiomatic Rust patterns
- Keep function bodies concise — extract large logic into smaller functions

**Naming Conventions**
- Modules: snake_case (e.g., `transcribe`, `pipeline`)
- Functions: snake_case (e.g., `process_video`, `extract_audio`)
- Structs: PascalCase (e.g., `Transcript`, `PipelineConfig`)
- Enums: PascalCase for variants (e.g., `PipelineState`, `PipelineEvent`)
- Constants: UPPER_SNAKE_CASE (e.g., `SILENCE_THRESHOLD_DB`)

**Error Handling**
- Use `Result<T, E>` for all functions returning results
- Define custom error types with `thiserror` or `anyhow`
- Never unwrap() or panic() in production code
- Return descriptive error messages to frontend
- Handle Tauri specific errors properly (`tauri::Error`)

**API Design**
- Use `#[tauri::command]` for all exposed functions
- Pass Tauri state using `app: AppHandle` or inject into commands
- Use Channels (`Sender`, `Receiver`) for progress streaming
- Define clear types for input/output structures

**Type System**
- Use generics where appropriate (`Result<T, E>`)
- Prefer `String` over `&str` for cross-thread communication
- Use `Arc<Mutex<T>>` for shared state
- Define clear type boundaries between modules

**Testing**
- Write unit tests for pure functions
- Write integration tests for Tauri commands
- Mock external dependencies in tests
- Test error paths and edge cases
- Keep tests deterministic and fast

**Code Organization**
```rust
// src-tauri/src/main.rs — Entry point
// src-tauri/src/lib.rs — Public API
// src-tauri/src/pipeline.rs — Pipeline orchestration
// src-tauri/src/transcribe.rs — Whisper integration
// src-tauri/src/ffmpeg.rs — FFmpeg wrapper
// src-tauri/src/models.rs — Shared types
```

### Cross-Project Consistency

**Shared Types**
- Define shared data structures in Rust
- Use `serde` for serialization/deserialization
- Keep Rust types in `models.rs` and corresponding TypeScript interfaces in frontend
- Maintain backward compatibility when changing shared types

**Configuration**
- Use environment variables for configuration
- Define defaults in Rust constants
- Keep frontend configuration minimal
- Validate all inputs before processing

**Logging**
- Frontend: Console logging for development
- Backend: Structured logging (or `println!` for simplicity in MVP)
- Log important events: start, completion, errors, performance metrics
- Never log sensitive information (paths, tokens, etc.)

**Performance Considerations**
- Use async/await for I/O operations
- Stream large data instead of loading into memory
- Reuse resources (FFmpeg processes, audio buffers)
- Profile before optimizing
- Consider background processing for long tasks

---

## Testing Strategy

### Unit Tests
- Test pure functions in isolation
- Mock external dependencies
- Test error handling paths
- Keep tests fast (< 1s)

### Integration Tests
- Test Tauri commands end-to-end
- Use test fixtures (sample videos, audio files)
- Verify IPC communication
- Check file system interactions

### Manual Testing Checklist
- [ ] Drop video files work (.mov, .mp4, .mkv)
- [ ] Progress indicator shows correct stage
- [ ] Toggles enable/disable features correctly
- [ ] Exported file has correct name and location
- [ ] Transcription is accurate (Spanish language)
- [ ] Silences are removed
- [ ] Audio is enhanced
- [ ] Error handling works (missing files, etc.)
- [ ] macOS notifications appear
- [ ] Multiple file types handled correctly
- [ ] Very short files ( < 5 seconds) work
- [ ] Long files ( > 10 minutes) work
- [ ] Edge cases (all silence, no silence) handled gracefully

---

## Common Tasks

### Adding a New Tauri Command
1. Define input/output types in `models.rs`
2. Implement function in appropriate module (e.g., `pipeline.rs`)
3. Add `#[tauri::command]` attribute
4. Register command in `lib.rs` invoke handler
5. Create TypeScript wrapper in `tauri.ts`

### Adding a New Frontend Component
1. Create component file in `src/lib/components/`
2. Define TypeScript interfaces for props
3. Add component to appropriate page
4. Test in development environment
5. Verify responsive design

### Adding a New Pipeline Stage
1. Define new `PipelineEvent` enum variant
2. Implement stage logic in `pipeline.rs`
3. Update progress streaming in orchestrator
4. Add frontend handling in `pipeline-progress.svelte`
5. Test with real video files

---

## Project Structure Reference

```
autovideo/
├── src/                          # SvelteKit frontend
│   ├── routes/
│   │   ├── +layout.ts           # Export ssr: false
│   │   └── +page.svelte         # Main app
│   ├── lib/
│   │   ├── components/
│   │   │   └── pipeline-progress.svelte
│   │   ├── stores/
│   │   │   └── pipeline.ts
│   │   └── tauri.ts             # Tauri wrappers
│   ├── app.html
│   └── app.css
│
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── pipeline.rs
│   │   ├── transcribe.rs
│   │   ├── ffmpeg.rs
│   │   └── models.rs
│   ├── binaries/
│   │   └── ffmpeg-aarch64-apple-darwin
│   ├── capabilities/
│   │   └── default.json
│   ├── Cargo.toml
│   └── tauri.conf.json
│
└── models/                       # Whisper models (gitignored)
    └── ggml-base.bin
```

---

## Security Considerations

### File System
- Never modify original files
- Always validate file paths before processing
- Clean up temporary files after processing
- Check file extensions before processing
- Validate file sizes (max ~2GB for MVP)

### Permissions
- Use Tauri capabilities for fine-grained permissions
- Only request necessary permissions (file system, shell)
- Never grant more permissions than needed
- Document all permissions in capabilities files

### Code Signing
- Sign FFmpeg binary before bundling
- Use proper code signing for distribution
- Include notarization for macOS
- Follow Apple security guidelines

---

## Development Workflow

1. **Feature Development**
   - Create feature branch from main
   - Implement feature with tests
   - Update documentation if needed
   - Run lint, typecheck, tests
   - Commit with descriptive message

2. **Code Review**
   - Ensure all tests pass
   - Check code follows style guidelines
   - Verify security considerations
   - Test locally before requesting review

3. **Deployment**
   - Build production versions
   - Sign and notarize for macOS
   - Create DMG package
   - Test installation process
   - Release to users

---

**Note**: This is a brand new project (0% implementation). Refer to `project.md` for detailed specifications and `plan.md` for implementation roadmap.
