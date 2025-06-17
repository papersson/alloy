# Movement Polish & Sky Implementation TODO

## Movement Feel Improvements
- [ ] Reduce movement speed from 5.0 to 2.0 (more appropriate for 100m sphere)
- [ ] Add smooth up vector interpolation to prevent jarring transitions
- [ ] Ensure movement vectors are properly tangent to sphere surface
- [ ] Add camera orientation smoothing/interpolation
- [ ] Test movement thoroughly at poles and edges

## Sky System
- [ ] Create basic sky rendering system
- [ ] Add simple gradient sky color (dark blue to light blue)
- [ ] Implement basic atmospheric fog effect
- [ ] Prepare infrastructure for future skybox

## Technical Tasks
- [ ] Update InputState movement speed
- [ ] Add up vector interpolation in Camera struct
- [ ] Fix forward/right vector calculations for spherical movement
- [ ] Create sky rendering pass in renderer
- [ ] Run verification loops after each change