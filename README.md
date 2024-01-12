<div align="center">

# Health Check

Run healthchecks in your terminal

</div>

Usage

```
cargo run -- ./config.json
```

An example config file

```
[
  {
    "domain": "localhost",
    "path": "/health_check",
    "port": "3000",

    "timeout_sec": 5,
    "check_interval_sec": 6,
    "healthy_threshold": 2,
    "unhealthy_threshold": 3
  }
]
```
