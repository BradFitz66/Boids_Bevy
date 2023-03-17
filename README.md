# Boids_Bevy
Simple Boid simulation in Rust w/ Bevy.

Getting familiar with parallelization in Bevy and Rust in general. Some stuff are paralellized (such as neighbor gathering and updating the transforms and velocity of the boids)

But other parts, like flocking behaviours, haven't been. Even so, this is able to run 10k boids at a pretty stable 144FPS on my  laptop(i7-10875H with 32GB of RAM)

The (somewhat insane) stretch goal is 100k boids at a stable a 60FPS or higher though from my testing, 100k is when rendering starts becoming a bottleneck,
and I don't really know how to fix that since as far as I know, all boids should be rendered in one draw call (if I'm incorrect, make an issue with a fix).

If you see any performance improvements that can be made, either make an issue or a pull request with the performance improvement.
