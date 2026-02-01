# Delta ($\Delta$)

Made for benchmarking similar programs based on time it takes to execute and the memory used during the programs execution.

> [!CAUTION]
>
> $\Delta$ is currently work in progress and may change dramatically

## Benchmark config example

```toml
threads = 2                        # override how many threads the program uses (defaults to systems suggestion)
iterations = 10                    # The gloabl amount of iterations (defaults to 200)
max_execution_time = "60s"         # The global time before the process is supposed to get killed (defaults to 10s)
memory_sampling_interval = "10ms"  # Set the memory sampling rate (defaults to 100us)

[[command]]
cmd = "./rust-fibonacci"           # The cmd to execute
args = ["20000"]                   # List of arguments for the cmd

[[command]]
cmd = "./go-fibonacci"
args = ["20000"]

iterations = 3                     # Override the gloabl iteration limit for this cmd
max_execution_time = "300s"        # Override the global max_time for this cmd
memory_sampling_interval = "10ms"  # Override the gloabel memory_sampling_interval
```
