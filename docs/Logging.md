# Logging

Log output is controlled via the `RUST_LOG` environment variable. Its format is documented here: https://docs.rs/tracing-subscriber/0.3/tracing_subscriber/struct.EnvFilter.html#directives

By default, pajbot3 outputs all types of log messages. You can use the `RUST_LOG` environment variable to limit what messages are output to the console.
