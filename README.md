# Heatzy Rust Client

A Rust client library and CLI tool for the Heatzy REST API.

## Features

- Async API client using `reqwest` with rustls
- Command-line interface for easy device control
- Support for all Heatzy Pilote heating modes
- Flexible device identification by name or ID
- Comprehensive error handling and logging

## Installation

### As a library

Add this to your `Cargo.toml`:

```toml
[dependencies]
heatzy = "0.1"
```

### As a CLI tool

```bash
cargo install heatzy
```

## Usage

### Library Usage

```rust
use heatzy::{Client, DeviceMode};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client and authenticate
    let mut client = Client::new()?;
    client.connect("user@example.com", "password").await?;
    
    // List all devices
    let devices = client.list_devices().await?;
    for device in devices {
        println!("{}: {}", device.dev_alias, device.did);
    }
    
    // Control a device by name
    let device = client.get_device_by_name("Bedroom").await?;
    client.set_device_mode(&device.did, DeviceMode::Eco).await?;
    
    // Get current mode
    let mode = client.get_device_mode(&device.did).await?;
    println!("Current mode: {}", mode);
    
    Ok(())
}
```

### CLI Usage

#### Authentication

Login and save token:
```bash
# Login returns only the token to stdout
TOKEN=$(heatzy login --username user@example.com --password secret)

# Use the token for subsequent commands
heatzy --token $TOKEN devices
```

Or provide credentials directly:
```bash
heatzy --token YOUR_TOKEN_HERE devices
```

#### Device Management

List all devices:
```bash
heatzy --token $TOKEN devices
```

Get device information:
```bash
# By name
heatzy --token $TOKEN device --name "Bedroom"

# By ID
heatzy --token $TOKEN device --id "iYgWgYcmCLh6q06aTur7ha"
```

#### Mode Control

Get current mode:
```bash
# By name
heatzy --token $TOKEN get-mode --name "Bedroom"

# By ID
heatzy --token $TOKEN get-mode --id "iYgWgYcmCLh6q06aTur7ha"
```

Set mode:
```bash
# By name
heatzy --token $TOKEN set-mode --name "Bedroom" eco

# By ID
heatzy --token $TOKEN set-mode --id "iYgWgYcmCLh6q06aTur7ha" comfort
```

Available modes:
- `comfort` - Comfort mode
- `eco` - Economy mode
- `frost-protection` (or `frost`) - Frost protection mode
- `stop` - Off
- `comfort-1` - Comfort minus 1°C
- `comfort-2` - Comfort minus 2°C

#### Logging

Control log verbosity with `--log-level`:
```bash
heatzy --log-level debug --token $TOKEN devices
```

Levels: `error`, `warn`, `info`, `debug`, `trace`

## API Coverage

- ✅ Authentication (`POST /login`)
- ✅ List devices (`GET /bindings`)
- ✅ Get device info (`GET /devices/{id}`)
- ✅ Get device mode (`GET /devdata/{id}/latest`)
- ✅ Set device mode (`POST /control/{id}`)

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.