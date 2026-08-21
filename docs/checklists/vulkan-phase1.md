# Vulkan Phase 1 -- Production Bridge Implementation Plan

## Status: IN PROGRESS

## Overview

Expand `packages/xiom-vulkan/` from 22 toy-bridge functions to ~80 production-grade functions covering:
- Buffer creation (vertex, index, uniform, storage)
- Image/texture creation and management
- Sampler configuration
- Descriptor sets and layouts
- Compute pipelines
- Multi-pass rendering
- Custom render passes
- Full XIOM type layer with contracts

## Bug Fixes (Blocking Demos)

### Bug 1: CCW Triangle/Quad Culling
- **Root cause**: `VK_FRONT_FACE_COUNTER_CLOCKWISE` + `VK_CULL_MODE_BACK_BIT` culls CCW faces
- **Affected**: `triangle.vert` (CCW), `quad.vert` (CCW)
- **Unaffected**: `cube.vert` (CW), `particle.vert` (no culling)
- **Fix**: Reorder triangle.vert and quad.vert vertices to CW winding
- [ ] Fix `bridge/shaders/triangle.vert` -- reorder vertex positions for CW
- [ ] Fix `bridge/shaders/quad.vert` -- change vertex generation for CW

### Bug 2: set_clear_color Called After begin_frame
- **Root cause**: `set_clear_color` updates `a->clear_r/g/b` but `begin_frame` already read them
- **Fix**: Move `set_clear_color` calls BEFORE `begin_frame` in all demos
- [ ] Fix `examples/demo_2d.xi`
- [ ] Fix `examples/demo_3d.xi`
- [ ] Fix `examples/demo_cubes.xi`
- [ ] Fix `examples/demo_particles.xi`
- [ ] Fix `examples/demo_shapes.xi`

## Phase 1 Bridge API Expansion (22 -> ~80 functions)

### A. Buffer Management (`xvk_buffer_*`)
| Function | Purpose |
|----------|---------|
| `xvk_buffer_create(size, usage, memory_type)` | Create VkBuffer + allocate + bind memory |
| `xvk_buffer_destroy(buffer)` | Free buffer + memory |
| `xvk_buffer_map(buffer, offset, size)` | Map to host-visible pointer |
| `xvk_buffer_unmap(buffer)` | Unmap |
| `xvk_buffer_write(buffer, data, size, offset)` | Write data to mapped buffer |
| `xvk_buffer_read(buffer, offset, size, out)` | Read from mapped buffer |
| `xvk_buffer_size(buffer)` | Get buffer byte size |

### B. Image/Texture Management (`xvk_image_*`)
| `xvk_image_create_2d(w, h, format, usage, mips)` | Create VkImage + allocate + bind |
| `xvk_image_destroy(image)` | Free image + memory |
| `xvk_image_view_create(image, format, aspect)` | Create VkImageView |
| `xvk_image_view_destroy(view)` | Destroy view |
| `xvk_image_transition_layout(cb, image, old, new, mips)` | Pipeline barrier for layout transition |

### C. Sampler Management (`xvk_sampler_*`)
| `xvk_sampler_create(filter, address_mode, mip_mode, max_lod)` | Create VkSampler |
| `xvk_sampler_destroy(sampler)` | Destroy |

### D. Descriptor Sets (`xvk_descriptor_*`)
| `xvk_descriptor_set_layout_create(bindings)` | Create layout |
| `xvk_descriptor_set_layout_destroy(layout)` | Destroy |
| `xvk_descriptor_pool_create(max_sets, pool_sizes)` | Create pool |
| `xvk_descriptor_pool_destroy(pool)` | Destroy |
| `xvk_descriptor_set_allocate(pool, layout)` | Allocate set |
| `xvk_descriptor_set_write_buffer(set, binding, buffer, offset, range)` | Write buffer descriptor |
| `xvk_descriptor_set_write_image(set, binding, sampler, view, image_layout)` | Write image descriptor |

### E. Pipeline Management (`xvk_pipeline_*`)
| `xvk_pipeline_create_graphics(app, config)` | Create custom graphics pipeline |
| `xvk_pipeline_create_compute(app, shader)` | Create compute pipeline |
| `xvk_pipeline_destroy(pipeline)` | Destroy pipeline |
| `xvk_pipeline_layout_create(push_constant_ranges, descriptor_layouts)` | Create layout |
| `xvk_pipeline_layout_destroy(layout)` | Destroy layout |

### F. Compute Pipeline (`xvk_compute_*`)
| `xvk_compute_begin(cb)` | Begin compute pass |
| `xvk_compute_bind_pipeline(cb, pipeline)` | Bind compute pipeline |
| `xvk_compute_bind_descriptor_sets(cb, layout, sets)` | Bind descriptor sets |
| `xvk_compute_dispatch(cb, x, y, z)` | Dispatch compute |
| `xvk_compute_end(cb)` | End compute pass |

### G. Multi-Pass & Render Targets
| `xvk_render_pass_create(app, color_formats, depth_format)` | Create custom render pass |
| `xvk_render_pass_destroy(rp)` | Destroy |
| `xvk_framebuffer_create(rp, attachments, w, h)` | Create framebuffer |
| `xvk_framebuffer_destroy(fb)` | Destroy |
| `xvk_begin_render_pass_custom(cb, rp, fb, w, h, clears)` | Begin custom render pass |
| `xvk_cmd_bind_vertex_buffers(cb, binding, buffer)` | Bind vertex buffer |
| `xvk_cmd_bind_index_buffer(cb, buffer, index_type)` | Bind index buffer |
| `xvk_cmd_draw_indexed(cb, index_count, instance_count, first_index, vertex_offset, first_instance)` | Indexed draw |
| `xvk_cmd_bind_pipeline(cb, pipeline)` | Bind pipeline (general) |

### H. Uniform/Storage Buffers
| `xvk_cmd_push_constants(cb, layout, stage, offset, size, data)` | Push constants (general) |
| `xvk_cmd_bind_descriptor_sets(cb, bind_point, layout, first_set, sets)` | Bind descriptor sets |

### I. Vertex Input
| `xvk_vertex_input_state_create(bindings, attributes)` | Create vertex input state (for pipeline config) |
| `xvk_vertex_input_state_destroy(vi)` | Destroy |

## XIOM Type Layer (`xiom.vulkan`)

### New Types
| Type | Fields | Purpose |
|------|--------|---------|
| `Buffer` | `handle: Int; size: Int;` | GPU buffer handle |
| `Image` | `handle: Int; width: Int; height: Int;` | GPU image handle |
| `ImageView` | `handle: Int;` | Image view |
| `Sampler` | `handle: Int;` | Sampler |
| `DescriptorSetLayout` | `handle: Int;` | Descriptor set layout |
| `DescriptorPool` | `handle: Int;` | Descriptor pool |
| `DescriptorSet` | `handle: Int;` | Allocated descriptor set |
| `Pipeline` | `handle: Int;` | Graphics or compute pipeline |
| `PipelineLayout` | `handle: Int;` | Pipeline layout |
| `RenderPass` | `handle: Int;` | Render pass |
| `Framebuffer` | `handle: Int;` | Framebuffer |

### Contracts
- All handles validated with `requires: handle != 0`
- Buffer writes validated with `requires: offset + size <= buffer_size`
- Image operations validated with `requires: width > 0; requires: height > 0`

## New Demos (~15)

1. [x] `demo_2d` -- Fixed triangle (exists)
2. [x] `demo_3d` -- Fixed cube (exists)
3. [x] `demo_shapes` -- Fixed quads + triangle (exists)
4. [x] `demo_cubes` -- Fixed cube grid (exists)
5. [x] `demo_particles` -- Fixed particles (exists)
6. [ ] `demo_vertex_buffer` -- Custom geometry via vertex buffer
7. [ ] `demo_indexed_draw` -- Indexed geometry
8. [ ] `demo_texture` -- Textured quad
9. [ ] `demo_multipass` -- Render to texture, then sample
10. [ ] `demo_compute` -- Compute shader (particle simulation on GPU)
11. [ ] `demo_uniform_buffer` -- Per-frame uniforms
12. [ ] `demo_descriptor_sets` -- Multiple textures + samplers
13. [ ] `demo_lighting` -- Normal mapping with descriptor sets
14. [ ] `demo_instancing` -- Instanced rendering
15. [ ] `demo_post_process` -- Bloom / blur post-processing
16. [ ] `demo_game_objects` -- Many objects with individual uniforms
17. [ ] `demo_phong` -- Phong lighting with uniform buffers

## AI_CONTEXT.md Update

Add Section 8.28: `xiom.vulkan` -- Complete production API reference with all types, functions, demos, and build instructions.

## Build Script Updates

- Add all new demos to `build.ps1` and `build.sh` target maps
- Add new shader compilation targets
- Add compute shader support to shader compilation step
