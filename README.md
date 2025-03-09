# rbxsoap

A lightweight rust library for communicating with RCCService (Roblox Cloud Compute Service)

## Installing

```bash
cargo add rbxsoap
```

## Examples

Basic configuration:
```rust
use rbxsoap::RccSoapMessages;

let config = RccConfig {
    base_url: "boblox.com".to_string(), // your site url
    soap_namespace: "http://boblox.com".to_string(), // only needed if you replaced all urls in rccservice
};

let rcc = RccSoapMessages::new()::with_config(config);
```

Get all jobs:
```rust
use rbxsoap::RccSoapMessages;

let rcc = RccSoapMessages::new();
let endpoint = "http://localhost:64989";

match rcc.get_all_jobs(endpoint) {
    Ok(jobs) => println!("Jobs: {:?}", jobs),
    Err(e) => eprintln!("Error: {}", e),
}
```

Open job:
```rust
use rbxsoap::RccSoapMessages;
use uuid::Uuid;

let rcc = RccSoapMessages::new();

let endpoint = "http://localhost:64989";
let job_id = Uuid::new_v4().to_string();

let response = rcc.open_job(
    endpoint,
    &job_id,
    20,                       // expiration in seconds
    1,                        // cores
    "HelloWorld",             // script name
    "print('Hello, World!')", // script body
    &[]
)?;
```

Open JSON job:

> [!NOTE]
> JSON jobs can only be used on version 0.314.0.159032 and above

```rust
use rbxsoap::RccSoapMessages;

let rcc = RccSoapMessages::new();
let endpoint = "http://localhost:64989";

let game_json = rcc.format_game_open_json(
    1818,                                          // place_id
    1,                                             // creator_id
    &job_id,                                       // job_id
    "00000000-0000-0000-0000-000000000000",        // api_key
    10,                                            // max_players
    60,                                            // gsm_interval (heartbeat interval in seconds)
    53640,                                         // port_number (UDP/RakNet)
    "User",                                        // creator_type
    1,                                             // place_version
    "127.0.0.1",                                   // machine_address
    1,                                             // universe_id
);

let response = rcc.open_job(
    endpoint,
    &job_id,
    120,                   // expiration in seconds
    1,                     // cores
    "GameServer",          // script name
    &game_json,            // the JSON configuration as the script
    &[]                    // no additional arguments needed
)?;
```

## Contribution
Want to contribute? Simply create a pull request, or if you are experiencing problems please open an issue.