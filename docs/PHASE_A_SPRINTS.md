<!-- Copyright (c) 2026 Eleftherios Notas and XIOM Foundation -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

# Ecosystem -- Phase A: Clean Architecture

**Goal**: Decouple windowing from rendering. Every package has ONE responsibility.
**Principle**: "If it compiles, it won't crash." All public functions have contracts.
**Status**: Sprint A1 -> A4 planned. Implementation starting now.

---

## Architecture Target

```
+-----------------------------------------------------+
|                  XIOM Application                    |
|-----------------------------------------------------|
|  xiom-imgui (GUI widgets, Dear ImGui)               |
|  depends: xiom-glfw (input) + xiom-vulkan (render)  |
|-----------------------------------------------------|
|  xiom-vulkan (GPU rendering, compute)                |
|  depends: xiom-glfw (window surface handle only)     |
|-----------------------------------------------------|
|  xiom-glfw (window, input, DPI, monitors, events)    |
|  depends: nothing (standalone C bridge)              |
|-----------------------------------------------------|
|  xiom.ffi (stdlib: malloc/free/memcpy, SafePtr,      |
|            FFIBuffer, FFIError, struct marshal)      |
`-----------------------------------------------------+

Future packages (Phase B): xiom-SDL3, xiom-ffmpeg, xiom-assimp
All follow the same pattern: thin C bridge .obj + XIOM safe wrappers.
```

## Dependency philosophy

| Library | How shipped | Why |
|---------|-----------|-----|
| Vulkan SDK | User installs separately | 400MB+. Every GPU dev has it. Document `VULKAN_SDK` env var. |
| GLFW | User installs or system package | 3MB DLL. `winget install glfw`, `apt install libglfw3-dev`. Document `GLFW_DIR`. |
| Dear ImGui | Bundled as .obj (7 files, ~500KB) | Tiny. No package manager. Perfect bundling candidate. |
| All future libs | Same as GLFW | System-installed or package manager. Never bundled. |

Message to users:
> **XIOM ecosystem packages are thin, safe wrappers. They do not ship library binaries.** Install dependencies once with your system package manager, then `use` the XIOM package. The XIOM compiler links against what you already have. No vendored DLLs. No hidden installs. No surprises.

---

## Sprint A1: Create xiom-glfw

**Goal**: Standalone GLFW package. Window creation, input, DPI, events. Zero Vulkan dependency.

### Files to create:
```
packages/xiom-glfw/
|-- package.xi          # name: "xiom-glfw", version: "0.1.0"
|-- README.md           # User docs: install GLFW, env vars, usage
|-- glfw.xi             # XIOM FFI + safe wrappers with contracts
`-- bridge/
    |-- glfw_bridge.h   # C ABI: 15-20 functions (window, input, monitor)
    |-- glfw_bridge.c   # Implementation
    `-- build.ps1       # Compile glfw_bridge.obj
```

### C bridge functions (extracted from xvk_app.c):
```c
glfw_bridge_init();                    // glfwInit
glfw_bridge_terminate();              // glfwTerminate
glfw_bridge_create_window(w, h, title); // returns GLFWwindow*
glfw_bridge_destroy_window(win);       // glfwDestroyWindow
glfw_bridge_should_close(win);         // glfwWindowShouldClose
glfw_bridge_poll_events();            // glfwPollEvents
glfw_bridge_get_key(win, key);         // glfwGetKey
glfw_bridge_get_mouse_button(win, b);  // glfwGetMouseButton
glfw_bridge_get_cursor_pos(win, &x, &y);
glfw_bridge_get_framebuffer_size(win, &w, &h);
glfw_bridge_get_window_size(win, &w, &h);
glfw_bridge_set_window_title(win, title);
glfw_bridge_get_monitors(&count);      // returns GLFWmonitor**
glfw_bridge_get_primary_monitor();
glfw_bridge_get_monitor_name(monitor);
glfw_bridge_get_video_mode(monitor, &w, &h, &refresh);
glfw_bridge_set_window_monitor(win, mon, x, y, w, h, refresh); // fullscreen
glfw_bridge_get_window_monitor(win);   // returns monitor or NULL
```

### XIOM wrappers:
```xiom
pub type Window = Int;  // opaque handle (GLFWwindow*)
pub type Monitor = Int; // opaque handle (GLFWmonitor*)

pub fn create_window(title: Str, w: Int, h: Int) -> Result[Window, Str]
pub fn destroy_window(win: Window)
pub fn should_close(win: Window) -> Bool
pub fn poll_events()
pub fn get_key(win: Window, key: Int) -> Bool
pub fn get_framebuffer_size(win: Window) -> (Int, Int)
pub fn toggle_fullscreen(win: Window) -> Result[Unit, Str]
```

---

## Sprint A2: Refactor xiom-vulkan

**Goal**: xiom-vulkan takes a GLFW window handle, doesn't create one.

### Key change:
```xiom
// OLD: xvk_app_create creates window internally
let app = create_app("My App", 1280, 800);

// NEW: user creates window with xiom-glfw, passes it to vulkan
let win = xiom_glfw.create_window("My App", 1280, 800)?;
let app = xiom_vulkan.create_app(win)?;
```

### C bridge changes (xvk_app.c):
- Remove `glfwCreateWindow` call
- `xvk_app_create` takes `GLFWwindow*` instead of (title, w, h)
- `xvk_app_create_surface` uses the provided window
- Remove `glfwPollEvents` from `xvk_app_poll` (moved to xiom-glfw)
- Remove key/mouse/input functions (moved to xiom-glfw)
- Keep `xvk_app_poll` only for internal Vulkan state refresh

### XIOM changes (vulkan.xi):
- `create_app` takes `win: Window` from `xiom.glwf` package
- Keep `begin_frame`, `end_frame`, `set_clear_color` unchanged
- Keep all draw/camera/shader/texture functions unchanged

### Files affected:
```
packages/xiom-vulkan/
|-- vulkan.xi          # create_app signature change
|-- src/wrapper.xi     # VulkanApp.new takes Window
|-- bridge/xvk_app.c   # Accept GLFWwindow*, remove window creation
|-- bridge/xvk_frame.c # Remove glfwPollEvents, glfwGetKey
`-- examples/          # Update all ~12 demos
```

---

## Sprint A3: Refactor xiom-imgui

**Goal**: xiom-imgui uses xiom-glfw for input. No bundled GLFW backend.

### Key change:
```xiom
// OLD: imgui_impl_glfw.cpp bundled, init takes GLFWwindow*
unsafe { imgui_bridge_init(win); }

// NEW: uses xiom-glfw for input events
let win = xiom_glfw.create_window("ImGui App", 1280, 800)?;
xiom_imgui.init(win)?;  // registers callbacks via GLFW bridge
```

### Bridge changes:
- Remove `imgui_impl_glfw.cpp` / `imgui_impl_glfw.obj` from the package
- `imgui_bridge_init` takes a callback mechanism or raw window handle
- Keyboard/mouse/scroll events forwarded through xiom-glfw callbacks

### Files affected:
```
packages/xiom-imgui/
|-- imgui.xi           # init takes xiom_glfw.Window
|-- bridge/            # Remove imgui_impl_glfw.*
|-- build.ps1          # Remove imgui_impl_glfw.obj from link
`-- tests/demo_imgui.xi
```

---

## Sprint A4: Integration & Verification

**Goal**: All demos compile and run with new architecture.

### Verification:
- [ ] xiom-vulkan ALL 12 demos compile and render correctly
- [ ] xiom-imgui demo window opens, widgets work, F11 fullscreen works
- [ ] xiom-glfw standalone test: window opens, key/mouse input detected
- [ ] 0 C compiler errors, 0 C compiler warnings
- [ ] User message documented in each package README

### Files affected by dependency chain:
```
xiom-glfw (new)           -> 0 deps
xiom-vulkan (update)      -> dep: xiom-glfw
xiom-imgui (update)       -> dep: xiom-glfw + xiom-vulkan
```

---

## Phase B (deferred): New packages

```
xiom-SDL3     -> window/input alternative to GLFW
xiom-ffmpeg   -> media decode/encode
xiom-assimp   -> 3D asset import
```

All follow the Phase A pattern: thin C bridge + XIOM safe wrappers + system-installed deps.
