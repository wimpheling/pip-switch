use std::path::PathBuf;

use clap::{Parser, Subcommand};
use pip_switch_core::{
    decode_setting_value, encode_setting_value, Config, HidapiTransport, MonitorClient, Result,
    Setting,
};

#[derive(Debug, Parser)]
#[command(
    author,
    version,
    about = "Keyboard-friendly MSI MD342CQPW PIP/PBP switch utility"
)]
struct Cli {
    #[arg(short, long, global = true)]
    config: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    List,
    Identify,
    Probe {
        #[arg(long)]
        pip: bool,
    },
    Read {
        setting: String,
    },
    Swap,
    PipOn,
    PipOff,
    PipToggle,
    RawRead {
        command: String,
    },
    RawWrite {
        #[arg(long)]
        i_understand_risk: bool,
        command: String,
        value: String,
    },
    WriteExampleConfig {
        path: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    if let Command::WriteExampleConfig { path } = &cli.command {
        let path = path.clone().unwrap_or_else(Config::default_path);
        Config::write_example(&path)?;
        println!("wrote {}", path.display());
        return Ok(());
    }

    let config_path = cli.config.unwrap_or_else(Config::default_path);
    let config = Config::load_or_default(&config_path)?;
    let transport = HidapiTransport::new()?;
    let client = MonitorClient::new(transport, config);

    match cli.command {
        Command::List => list(&client),
        Command::Identify => identify(&client),
        Command::Probe { pip } => probe(&client, pip),
        Command::Read { setting } => read(&client, &setting),
        Command::Swap => {
            client.swap()?;
            println!("swap command sent");
            Ok(())
        }
        Command::PipOn => {
            client.pip_on()?;
            println!("PIP enabled");
            Ok(())
        }
        Command::PipOff => {
            client.pip_off()?;
            println!("PIP disabled");
            Ok(())
        }
        Command::PipToggle => {
            client.pip_toggle()?;
            println!("PIP toggled");
            Ok(())
        }
        Command::RawRead { command } => {
            println!("{}", client.raw_read(&command)?);
            Ok(())
        }
        Command::RawWrite {
            i_understand_risk,
            command,
            value,
        } => {
            if !i_understand_risk {
                return Err(pip_switch_core::Error::Message(
                    "raw-write requires --i-understand-risk".to_string(),
                ));
            }
            println!("{}", client.raw_write(&command, &value)?);
            Ok(())
        }
        Command::WriteExampleConfig { .. } => unreachable!("handled before HID init"),
    }
}

fn list(client: &MonitorClient<HidapiTransport>) -> Result<()> {
    let devices = client.list()?;
    if devices.is_empty() {
        println!("no matching MSI HID monitors found");
        return Ok(());
    }

    for (index, device) in devices.iter().enumerate() {
        println!(
            "{}: {:04x}:{:04x} product={} serial={} path={}",
            index,
            device.vendor_id,
            device.product_id,
            device.product.as_deref().unwrap_or("<unknown>"),
            device.serial.as_deref().unwrap_or("<none>"),
            device.path
        );
    }
    Ok(())
}

fn identify(client: &MonitorClient<HidapiTransport>) -> Result<()> {
    let identity = client.identify()?;
    println!("vendor_id={:04x}", identity.device.vendor_id);
    println!("product_id={:04x}", identity.device.product_id);
    println!(
        "manufacturer={}",
        identity
            .device
            .manufacturer
            .as_deref()
            .unwrap_or("<unknown>")
    );
    println!(
        "product={}",
        identity.device.product.as_deref().unwrap_or("<unknown>")
    );
    println!(
        "serial={}",
        identity.device.serial.as_deref().unwrap_or("<none>")
    );
    if let Some(model) = identity.model_raw {
        println!("model_raw={model}");
    }
    if let Some(firmware) = identity.firmware_raw {
        println!("firmware_raw={firmware}");
    }
    Ok(())
}

fn probe(client: &MonitorClient<HidapiTransport>, pip: bool) -> Result<()> {
    if !pip {
        identify(client)?;
        return Ok(());
    }

    for (setting, raw, decoded) in client.probe_pip()? {
        println!(
            "{} {} raw={} decoded={}",
            setting,
            setting.command(),
            raw,
            decoded
        );
    }
    Ok(())
}

fn read(client: &MonitorClient<HidapiTransport>, setting: &str) -> Result<()> {
    let setting = setting.parse::<Setting>()?;
    let raw = client.read_setting(setting)?;
    println!("raw={raw}");
    println!("decoded={}", decode_setting_value(setting, &raw));
    println!(
        "encoded_decoded={}",
        encode_setting_value(setting, &decode_setting_value(setting, &raw)).unwrap_or(raw)
    );
    Ok(())
}
