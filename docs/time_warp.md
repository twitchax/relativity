# Time Warp Level

## Overview

Time Warp is a challenging level that demonstrates the effects of gravitational time dilation in a dynamic environment. The level features a moving gravity well that orbits around the center of the playing field, creating a constantly shifting gravitational landscape and time-dilation zone.

## Level Mechanics

### Player Starting Position
- **Position**: (100.0, 300.0) - approximately 12.5% from the left edge, centered vertically
- **Initial Velocity**: (260.0, 0.0) km/s - a strong horizontal boost to the right
- **Purpose**: The starting velocity creates a "slingshot" effect, launching the player towards the goal

### Moving Gravity Well ("time_well")
- **Type**: DynamicPlanetBundle (moving gravity source)
- **Orbit Center**: (400, 300) - center of the screen
- **Initial Position**: (0.65, 0.5) - right side of the orbit path
- **Orbital Radius**: ~120 pixels (0.15 screen width)
- **Mass**: 18,000 × Earth's mass - extremely strong gravitational influence
- **Physical Radius**: ~40 pixels (0.67 × UNIT_RADIUS) - small visual size
- **Initial Velocity**: (0.0, 280.0) km/s - upward for counterclockwise orbital motion
- **Visual**: planet07.png sprite

The gravity well creates a strong gravitational field that will pull on the player as they pass by. The well is designed to orbit around the center, creating a dynamic challenge where timing is crucial.

### Destination/Exit Gate
- **Position**: (760.0, 300.0) - far right edge, centered vertically (95% across screen)
- **Radius**: ~48 pixels (0.8 × UNIT_RADIUS) - medium-sized target
- **Mass**: 0.1 × Sun's mass - weak gravitational influence to avoid disrupting player trajectory too much
- **Visual**: noise00.png sprite (portal/wormhole appearance)

## Strategy and Gameplay

### Basic Strategy
1. **Launch Phase**: The player starts with significant horizontal velocity, aimed roughly towards the destination
2. **Gravitational Assist**: Use the moving gravity well's gravitational field to adjust trajectory
3. **Timing**: The position of the orbiting well when you launch is critical - it may help or hinder your path
4. **Precision**: The combination of initial velocity and gravitational interactions requires precise timing

### Difficulty Factors
- **Dynamic Environment**: The gravity well is constantly moving, so each attempt can feel different
- **Strong Gravity**: The 18,000× Earth mass creates powerful gravitational forces that can significantly alter trajectories
- **Precision Required**: The destination is relatively small and positioned at the far edge

## Time Dilation (Future Enhancement)

**Note**: The current implementation includes a TODO for time-dilation effects. The planned feature would:

1. Calculate the distance from the player to the moving gravity well
2. Apply time scaling to the player based on proximity:
   - **Close to well** (< threshold distance): time_scale = 0.5 (50% speed - slower movement and projectiles)
   - **Far from well** (> threshold distance): time_scale = 1.0 (normal speed)
   - **In between**: linear interpolation between 0.5 and 1.0
3. Affect player movement speed and responsiveness when inside the dilation zone

This would add an additional layer of complexity, as players would need to account for their slowed movement when near the well.

## Tuning Notes

### For Game Designers

Key tunable parameters and their effects:

1. **Player Initial Velocity** (currently 260.0 km/s):
   - Increase: Makes the level easier, faster to complete
   - Decrease: Makes the level harder, requires more precise gravitational assist

2. **Gravity Well Mass** (currently 18,000 × Earth mass):
   - Increase: Stronger gravitational pull, more dramatic trajectory changes
   - Decrease: Weaker pull, requires closer approaches for gravitational assists

3. **Gravity Well Orbital Velocity** (currently 280.0 km/s):
   - Increase: Faster orbital motion, more unpredictable
   - Decrease: Slower motion, easier to predict and time launches

4. **Destination Size** (currently 0.8 × UNIT_RADIUS):
   - Increase: Easier target to hit
   - Decrease: Requires more precision

5. **Destination Mass** (currently 0.1 × Sun mass):
   - Increase: Pulls player towards goal (can help or hinder)
   - Decrease: Less gravitational interference, more predictable trajectories

### Testing Tips

- To test the level, temporarily set it as the default level by adding `#[default]` attribute to the `TimeWarp` variant in the `CurrentLevel` enum
- Watch the orbital motion of the gravity well to ensure it creates interesting patterns
- Try multiple launch timings to ensure the level is solvable regardless of when the player launches
- Adjust velocities and masses if the level feels too easy or impossibly difficult

## Implementation Details

The level is implemented in `src/game/levels/mod.rs` as the `level_time_warp()` function. It uses the standard Bevy ECS architecture with the following bundles:

- `PlayerSpriteBundle` - The player's rocket
- `DynamicPlanetBundle` - The moving gravity well
- `DestinationBundle` - The exit gate/goal

The level leverages the existing physics system in `src/game/shared/systems.rs` which handles:
- Gravitational forces (velocity_update)
- Position updates based on velocity
- Collision detection with planets and destination
- Relativistic effects already built into the game engine
