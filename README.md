# Delta ($\Delta$)

Made for benchmarking similar programs based on time it takes to execute and the memory used during the programs execution.

> [!CAUTION]
>
> $\Delta$ is currently work in progress and may change dramatically

## Usage

To use $\Delta$ please create a [benchmark config](#config) and execute $\Delta$ like this:
```sh
delta -f <your_config_file> -o <your_output_directory>
```

Its that easy 🚀.


## Benchmark config example

<a name="config" />

```toml
threads = 2                        # override how many threads the program uses (defaults to systems suggestion) (only for memory measurment)
iterations = 10                    # The gloabl amount of iterations (defaults to 200)

max_execution_time = "60s"         # The global time before the process is supposed to get killed (defaults to 10s)

memory_sampling_interval = "10ms"  # Set the memory sampling rate (defaults to 100us)
measure_mem_once = true            # Only measure the memory once for each cmd (default: false)
memory_measuring_mode = "timeline" # Set the measuring mode (defaults to timeline)

warmup_count = 3                   # Set the global warmup count/iterations (defaults to 10)
warmup_mode = "interval"           # Set the global warmup mode (defaults to global)

[[command]]
cmd = "./rust-fibonacci"           # The cmd to execute
args = ["20000"]                   # List of arguments for the cmd

[[command]]
cmd = "./go-fibonacci"
args = ["20000"]
iterations = 3                     # Override the gloabl iteration limit for this cmd

max_execution_time = "300s"        # Override the global max_time for this cmd

memory_sampling_interval = "10ms"  # Override the gloabel memory_sampling_interval
measure_mem_once = true            # Override the global flag
memory_measuring_mode = "maximum"  # Override the global measuring mode

warmup_count = 3                   # Override the gloabel warmup count/intervals
warmup_mode = "interval"           # Override the gloabel warmup mode
```


### Memory measuring modes

> [!NOTE]
>
> $\Delta$ measures both physical and virtual memory.

| Mode                   | Description                                                                                                                                                   |
| ---------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `"timeline"` (default) | Track the memory usage of the process over time via polling the process info from the system                                                                  |
| `"maximum"`            | Get the [rss](https://en.wikipedia.org/wiki/Resident_set_size) from the unix system for the physical memory. Currently it does not measure the virtual memory |


### Warmup modes

> [!NOTE]
>
> Currently the warmup mode is not implemented yet.
> One can however add the config options as it will be supported in the next few weeks.

| Mode                 | Description                                                                                                                               |
| -------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- |
| `"global"` (default) | Runs the warmup iterations for all commands one time before executing them. This is a more realistic scenario than the `"interval"` mode. |
| `"interval"`         | Runs the warmup of a command right before it executes it, so the cache already has all the relevant data. This is a best case scenario.   |
