# Client Logger

The client logger fetches weather data (nowcasts) and lightning data from the WICTK service and stores them in the HEM (Home Energy Management) system.

## Features

- Fetch nowcast data from MET and OpenWeatherMap
- Store weather measurements in HEM system  
- Optional lightning data storage
- Structured JSON logging (matching backend format)
- Configurable log levels
- Automatic device and sensor setup

## Usage

### Basic Usage

```bash
# Run with default settings (Trondheim, info-level logging)
cargo run

# Run with custom location
cargo run -- --location "Oslo"

# Include lightning data storage
cargo run -- --store-lightning

# Use custom service URLs
cargo run -- --service-url "http://localhost:8080/" --hemrs-url "http://localhost:8081/"

# Set log level (matches backend behavior)
cargo run -- --log-level debug
```

### Logging Configuration

The client logger uses structured JSON logging identical to the backend's format. Log levels can be set using the `--log-level` command line option.

#### Log Levels

- `error`: Only critical errors
- `warn`: Warnings and errors
- `info`: General information, warnings, and errors (default)
- `debug`: Detailed debugging information
- `trace`: Very verbose tracing information

#### Examples

```bash
# Info level logging (default)
cargo run

# Debug level logging for detailed information
cargo run -- --log-level debug

# Only errors
cargo run -- --log-level error

# Maximum verbosity
cargo run -- --log-level trace
```

#### Log Output Format (JSON)

All log output is in structured JSON format, consistent with the backend:

**Info Level:**
```json
{"timestamp":"2025-08-11T19:56:59.109169Z","level":"INFO","fields":{"message":"Starting client logger with configuration: Opts { location: \"Trondheim\", service_url: \"http://wictk.frikk.io/\", hemrs_url: \"http://hemrs.frikk.io/\", store_lightning: false, log_level: Info }"},"target":"client_logger"}
{"timestamp":"2025-08-11T19:56:59.128286Z","level":"INFO","fields":{"message":"HTTP client initialized successfully"},"target":"client_logger"}
{"timestamp":"2025-08-11T19:56:59.128347Z","level":"INFO","fields":{"message":"=== FETCHING NOWCAST DATA ==="},"target":"client_logger"}
```

**Debug Level:**
```json
{"timestamp":"2025-08-11T19:57:45.123456Z","level":"DEBUG","fields":{"message":"Fetching nowcast data","url":"http://wictk.frikk.io/","location":"Trondheim"},"target":"client_logger","spans":[{"name":"get_nowcast"}]}
{"timestamp":"2025-08-11T19:57:45.234567Z","level":"DEBUG","fields":{"message":"Response status: 200 OK"},"target":"client_logger"}
```

### Logging Structure

The application provides structured logging at different stages:

1. **Initialization**: Configuration logging, HTTP client setup
2. **Data Fetching**: API requests, response handling, error reporting
3. **Device/Sensor Setup**: HEM service communication, device creation/retrieval
4. **Data Storage**: Measurement creation and storage, success/error reporting
5. **Lightning Processing**: Filtering logic, storage results, error tracking

### Integration with Backend

The client logger's logging format is designed to be consistent with the backend service:

- Same JSON structure for logs
- Identical log level naming and behavior
- Compatible with the same log aggregation and monitoring tools
- Consistent error reporting format

## Configuration Options

| Option | Short | Default | Description |
|--------|--------|---------|-------------|
| `--location` | `-l` | `Trondheim` | Location for weather data |
| `--service-url` | `-s` | `http://wictk.frikk.io/` | WICTK API base URL |
| `--hemrs-url` | `-r` | `http://hemrs.frikk.io/` | HEM service base URL |
| `--store-lightning` | | `false` | Enable lightning data storage |
| `--log-level` | | `info` | Set logging level (trace, debug, info, warn, error) |

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with debug logging
cargo test -- --log-level debug

# Run specific test module
cargo test hem::tests
```

### Log Processing

Since logs are in JSON format, they can be easily processed with tools like `jq`:

```bash
# Extract only error messages
cargo run 2>&1 | jq 'select(.level == "ERROR") | .fields.message'

# Show timestamps and messages
cargo run 2>&1 | jq '{timestamp: .timestamp, level: .level, message: .fields.message}'

# Filter by target
cargo run 2>&1 | jq 'select(.target == "client_logger")'
```

### Production Deployment

For production use, consider:

1. **Log Level**: Set to `info` or `warn` to reduce log volume
2. **Log Aggregation**: JSON format works well with ELK stack, Splunk, or similar
3. **Monitoring**: Set up alerts on ERROR level messages
4. **Storage**: Logs can be large at debug level, plan storage accordingly

Example production command:
```bash
cargo run --release -- --log-level warn --location "Production" >> /var/log/wictk/client_logger.json 2>&1
```
