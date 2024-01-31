# Worklog

## Exploration strategy

We can "find farlands" by evaluating over multiple types in lockstep: at the
same number of iterations, have two different fractals escaped (or not)?

## Evaluation strategy

### Input coordinates
I'd like the input to be an array-of-BigRational, so each coordinate is
"as precise as it can be".

We could do that "for each fractal"; somewhat nicer if we can do it once for all
of them. It's not a _tiny_ amount of computation.

### Vectorization

I think if we can run along a Y-coordinate, then we'll get decent cache locality
etc. per-thread.

### Parallelism

I think we want something like a thread-pool. The native types (IEEE754,
fixed-point?) will probably be faster; we should have some way of scheduling
the work s.t. we can complete the "fast" ones quickly and then spin up to
full speed on the others.

(Something like - tokening? Each type starts with $N_threads / N_types$ tokens,
rounded up & down so the total equals $N_threads$. Spend one to dispatch,
get it back on completion, return each token forever when there's no work;
and those tokens come back evenly distributed to the others.)

### Refinement

I'd like to be able to present _progressively refined_ images, which can happen
in two ways:

-   Increased resolution, i.e. more pixels in the same window;
    view as sharpening
-   Increased iterations, i.e. more depth for the same pixels;
    view as a surface moving in

Increased resolution can increase from stateful evaluation if we can preserve
the "intermediate" values...and get them back.

Increased iterations can benefit from state if the cost of pulling e.g.

```rust
struct EvalCell<N> {
    iterations: u32,
    computed_value: N,
    coordinates: (N, N),
}
```

out of cache is less than the cost of however many iterations we've done so far.
For a large number of iterations - sure.



