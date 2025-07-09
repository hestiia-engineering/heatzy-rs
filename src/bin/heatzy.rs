use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use flexi_logger::{Logger, WriteMode};
use heatzy::{Client, DeviceMode};
use log::{debug, error};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Authentication token (can also use login subcommand)
    #[arg(short, long, global = true)]
    token: Option<String>,
    
    /// Log level (error, warn, info, debug, trace)
    #[arg(long, default_value = "warn", global = true)]
    log_level: String,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Login and get authentication token
    Login {
        /// Username (email)
        #[arg(short, long)]
        username: String,
        
        /// Password
        #[arg(short, long)]
        password: String,
    },
    
    /// List all devices
    Devices,
    
    /// Get device information
    Device {
        /// Device name
        #[arg(long = "name", group = "device")]
        device_name: Option<String>,
        
        /// Device ID
        #[arg(long = "id", group = "device")]
        device_id: Option<String>,
    },
    
    /// Get current device mode
    GetMode {
        /// Device name
        #[arg(long = "name", group = "device")]
        device_name: Option<String>,
        
        /// Device ID
        #[arg(long = "id", group = "device")]
        device_id: Option<String>,
    },
    
    /// Set device mode
    SetMode {
        /// Device name
        #[arg(long = "name", group = "device")]
        device_name: Option<String>,
        
        /// Device ID
        #[arg(long = "id", group = "device")]
        device_id: Option<String>,
        
        /// Mode (comfort, eco, frost-protection, stop, comfort-1, comfort-2)
        mode: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize logger - logs go to stderr
    Logger::try_with_str(&cli.log_level)
        .context("Failed to parse log level")?
        .write_mode(WriteMode::Direct)
        .start()
        .context("Failed to initialize logger")?;
    
    match cli.command {
        Commands::Login { username, password } => {
            debug!("Performing login");
            let client = Client::new().context("Failed to create client")?;
            
            match client.login(&username, &password).await {
                Ok(auth_response) => {
                    // Output only the token to stdout
                    println!("{}", auth_response.token);
                }
                Err(e) => {
                    error!("Login failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        
        _ => {
            // All other commands require authentication
            let mut client = Client::new().context("Failed to create client")?;
            
            if let Some(token) = cli.token {
                client.set_token(token);
            } else {
                error!("No authentication token provided. Use --token or login first");
                std::process::exit(1);
            }
            
            match cli.command {
                Commands::Devices => {
                    let devices = client.list_devices().await.context("Failed to list devices")?;
                    
                    for device in devices {
                        println!("{:<30} {} {} ({})", 
                            device.dev_alias.as_deref().unwrap_or("(no name)"), 
                            device.did, 
                            if device.is_online { "✓" } else { "✗" },
                            device.product_name
                        );
                    }
                }
                
                Commands::Device { device_name, device_id } => {
                    let device = match (device_name, device_id) {
                        (Some(name), None) => {
                            client.get_device_by_name(&name).await
                                .context("Failed to get device by name")?
                        }
                        (None, Some(id)) => {
                            client.get_device(&id).await
                                .context("Failed to get device by ID")?
                        }
                        _ => {
                            error!("Must specify either --name or --id");
                            std::process::exit(1);
                        }
                    };
                    
                    // Device name is not returned by this endpoint
                    if let Some(alias) = &device.dev_alias {
                        println!("Name:    {}", alias);
                    }
                    println!("ID:      {}", device.did);
                    println!("Product: {}", device.product_name);
                    println!("MAC:     {}", device.mac);
                    println!("Online:  {}", if device.is_online { "Yes" } else { "No" });
                }
                
                Commands::GetMode { device_name, device_id } => {
                    let device_id = match (device_name, device_id) {
                        (Some(name), None) => {
                            let device = client.get_device_by_name(&name).await
                                .context("Failed to get device by name")?;
                            device.did
                        }
                        (None, Some(id)) => id,
                        _ => {
                            error!("Must specify either --device-name or --device-id");
                            std::process::exit(1);
                        }
                    };
                    
                    let mode = client.get_device_mode(&device_id).await
                        .context("Failed to get device mode")?;
                    
                    println!("{}", mode);
                }
                
                Commands::SetMode { device_name, device_id, mode } => {
                    let device_id = match (device_name, device_id) {
                        (Some(name), None) => {
                            let device = client.get_device_by_name(&name).await
                                .context("Failed to get device by name")?;
                            device.did
                        }
                        (None, Some(id)) => id,
                        _ => {
                            error!("Must specify either --device-name or --device-id");
                            std::process::exit(1);
                        }
                    };
                    
                    let mode = DeviceMode::from_cli_str(&mode)
                        .context("Invalid mode")?;
                    
                    client.set_device_mode(&device_id, mode).await
                        .context("Failed to set device mode")?;
                    
                    println!("Device mode set to: {}", mode);
                }
                
                _ => unreachable!(),
            }
        }
    }
    
    Ok(())
}