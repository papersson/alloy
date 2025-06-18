# AAA Vegetation System Implementation Plan

## Overview
Transform the current basic grass and tree systems into AAA-quality vegetation with:
- Multi-level LOD system
- GPU-driven culling
- Subsurface scattering
- Natural distribution patterns
- High-quality textures and shading

## Phase 5.2 Tasks

### 1. LOD System Architecture
- [ ] Design LOD data structure (4 levels: full, reduced, billboard, fade)
- [ ] Create LOD mesh generation for grass blades
- [ ] Create LOD mesh generation for trees
- [ ] Implement distance-based LOD selection
- [ ] Add smooth LOD cross-fading

### 2. GPU-Driven Culling
- [ ] Create compute shader for frustum culling
- [ ] Implement distance-based culling
- [ ] Add Hi-Z occlusion culling
- [ ] Create indirect rendering buffers
- [ ] Implement GPU instance streaming

### 3. Advanced Shading
- [ ] Two-sided lighting for vegetation
- [ ] Subsurface scattering approximation
- [ ] Translucency effects for leaves
- [ ] Wind animation improvements
- [ ] Ambient occlusion integration

### 4. Texture System
- [ ] Implement texture arrays for grass variations
- [ ] Create texture atlas for tree leaves
- [ ] Add normal maps for grass blades
- [ ] Implement translucency maps
- [ ] Add detail textures for close-up views

### 5. Natural Distribution
- [ ] Create density maps for grass placement
- [ ] Implement Poisson disk sampling
- [ ] Add biome-based variation
- [ ] Create natural clustering patterns
- [ ] Path/road avoidance system

### 6. Performance Optimization
- [ ] Implement GPU indirect rendering
- [ ] Add instance data streaming
- [ ] Optimize shader complexity by LOD
- [ ] Memory pooling for instances
- [ ] Benchmark and profile

## Implementation Order
1. ✅ Start with LOD system for existing grass
2. Modify renderer to support multiple LOD levels:
   - Update GrassBuffers to store multiple meshes/instances
   - Render each LOD level separately
   - Add fade alpha support in shader
3. Add GPU culling infrastructure
4. Upgrade shading quality
5. Implement texture arrays
6. Improve distribution patterns
7. Apply same improvements to trees
8. Final optimization pass

## Key Files to Modify
- src/core/grass.rs - LOD generation, distribution
- src/core/tree.rs - LOD generation, billboards
- src/renderer/scene_renderer.rs - GPU culling, indirect rendering
- src/shaders/grass.metal - Advanced shading
- src/shaders/tree.metal - Billboard rendering
- New: src/core/vegetation_lod.rs
- New: src/shaders/vegetation_culling.metal

## Reference Quality Targets
- Horizon Zero Dawn vegetation
- Red Dead Redemption 2 grass
- Ghost of Tsushima foliage
- Unreal Engine 5 Nanite vegetation