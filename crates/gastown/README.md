# Gas Town - Minimal GPUI Application

This is a minimal GPUI application created as a proof-of-concept for transforming Zed into Gas Town UI.

## What This Is

A bare-bones GPUI application that:
- Uses only the essential crates from Zed (gpui, ui, theme)
- Opens a simple window with the title "Gas Town"
- Displays basic text in the center
- Demonstrates the minimal skeleton for building on GPUI

## Structure

```
crates/gastown/
├── Cargo.toml          # Minimal dependencies
├── README.md           # This file
└── src/
    └── main.rs         # Entry point with GPUI window
```

## Dependencies

The crate depends on only:
- `gpui` - The UI framework
- `ui` - Common widgets
- `theme` - Styling
- `anyhow` - Error handling
- `env_logger` - Logging

This is in contrast to the full Zed binary which depends on 100+ crates.

## What Was Removed (Conceptually)

For this PoC, we chose NOT to remove crates from the workspace (too invasive), but instead
created a new minimal crate. In a full transformation, these would be removed:

**Removed:**
- `editor` - Text editing
- `lsp` - Language server support
- `terminal_view` - Terminal emulator
- `collab` - Multiplayer features
- `vim` - Vim mode
- `languages` - Language definitions
- All AI/assistant crates (agent, anthropic, bedrock, etc.)

**Kept:**
- `gpui` - UI framework
- `ui` - Common widgets
- `theme` - Styling
- `git` - Git integration (needed for gastown)
- `workspace` - Window management
- `project` - Project management (to adapt for rigs)
- `menu` - Context menus
- `picker` - Selection widgets

## Building

```bash
# Check compilation (lighter than full build)
cargo check -p gastown

# Build
cargo build -p gastown

# Run
cargo run -p gastown
```

Note: Building requires significant resources as GPUI depends on system libraries.

## Next Steps

After this PoC is validated:
1. Remove unnecessary crates from workspace
2. Adapt `workspace` for rig management
3. Adapt `project` for rig state
4. Create Gas Town panels (rig list, agent status, etc.)
5. Integrate with `gt` CLI commands
6. Add Git UI components for gastown tracking
