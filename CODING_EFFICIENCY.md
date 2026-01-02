# Coding Efficiency Guide

This document provides guidelines, tools, and best practices for enhancing coding efficiency when working on the Proxy-Desktop-Browser project.

## 🚀 Quick Start for Developers

### Essential Commands

| Task | Command |
|------|---------|
| Build all | `make build` |
| Run tests | `make test` |
| Format code | `make fmt` |
| Lint check | `make lint` |
| Clean build | `make clean` |
| Dev mode | `cargo tauri dev` |

### IDE Setup

#### VS Code Extensions
- **rust-analyzer**: Rust language support
- **Svelte for VS Code**: Svelte syntax highlighting
- **Tauri**: Tauri development support
- **Error Lens**: Inline error display
- **GitLens**: Git integration

## 📁 Code Organization

### Module Structure

The project follows a modular structure:
- `src/lib.rs` - Public exports only
- `src/prelude.rs` - Common imports
- `src/module/mod.rs` - Module interface
- `src/module/types.rs` - Type definitions

### Import Convention

Use the prelude module for common imports in Rust.

## 🔧 Utility Functions

### Prelude Module Utilities

The `prelude.rs` module provides several utilities:

- **Error Handling**: OptionExt and ResultExt traits for better error context
- **Retry Logic**: RetryConfig for automatic retries with exponential backoff
- **Rate Limiting**: RateLimiter to control operation frequency
- **Circuit Breaker**: CircuitBreaker to prevent cascade failures

### TypeScript Utilities

Use the logger module instead of console:
- `logInfo()` - Information messages
- `logError()` - Error messages
- `logDebug()` - Debug messages
- `logWarn()` - Warning messages

## ✅ Code Quality Checklist

### Before Committing

- Run `cargo fmt` to format Rust code
- Run `cargo clippy` and fix warnings
- Run `cargo test` to ensure tests pass
- Run `npm run lint` for TypeScript/Svelte
- Update documentation if needed
- Add tests for new functionality

### Code Review Standards

1. **No `any` types** - Use proper TypeScript types
2. **No `unwrap()` in production** - Use `?` or proper error handling
3. **No `console.log`** - Use structured logging
4. **Max function complexity < 15** - Refactor complex functions
5. **Max line length 100** - Break long lines

## 🧪 Testing Best Practices

### Unit Tests (Rust)
- Place tests in the same file or `tests.rs`
- Use descriptive test names
- Follow Arrange-Act-Assert pattern
- Test edge cases and error conditions

### Integration Tests
- Test component interactions
- Use test fixtures for setup
- Clean up after tests

## 🔄 Common Patterns

### Error Handling Pattern

Use the `?` operator with `.context()` for meaningful error messages.

### Builder Pattern

Use builders for complex object construction with many optional parameters.

### State Management (Svelte)

Use Svelte stores for shared state across components.

## 📊 Performance Tips

### Rust
- Use `&str` instead of `String` for function parameters
- Prefer `Vec::with_capacity()` when size is known
- Use `Arc<T>` for shared ownership across threads
- Avoid unnecessary cloning - use references

### TypeScript/Svelte
- Use reactive statements sparingly
- Implement virtual scrolling for long lists
- Lazy load components with dynamic imports
- Debounce expensive operations

## 🛠️ Debugging

### Rust
- Enable debug logging: `RUST_LOG=debug cargo run`
- Use `dbg!()` macro for quick debugging
- Use `tracing` spans for performance analysis

### Browser DevTools
- Use Tauri devtools: `cargo tauri dev --devtools`
- Check console for frontend logs
- Use Network tab for API debugging

## 📚 Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Tauri Documentation](https://tauri.app/v1/guides/)
- [Svelte Documentation](https://svelte.dev/docs)
- [TypeScript Handbook](https://www.typescriptlang.org/docs/handbook/)

---

*Last updated: 2025-01-02*
