# Delta ($\Delta$)

Made for benchmarking similar programs based on time it takes to execute and the memory used during the programs execution.

> [!CAUTION]
>
> $\Delta$ is currently work in progress and may change dramatically

## Benchmark config example

```toml
iterations = 10                # The gloabl amount of iterations (defaults to 200)
max_time = "60s"               # The global time before the process is supposed to get killed (defaults to 10)

[[command]]
cmd = "./rust-fibonacci 20000" # The cmd to execute

[[command]]
cmd = "./go-fibonacci 20000"
iterations = 3                 # Override the gloabl iteration limit for this cmd
max_time = "300s"              # Override the global max_time for this cmd
```
