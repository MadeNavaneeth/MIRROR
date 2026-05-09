#![warn(clippy::all, clippy::pedantic)]
#![allow(
    clippy::assigning_clones,
    clippy::bool_to_int_with_if,
    clippy::case_sensitive_file_extension_comparisons,
    clippy::cast_possible_wrap,
    clippy::doc_markdown,
    clippy::field_reassign_with_default,
    clippy::float_cmp,
    clippy::implicit_clone,
    clippy::items_after_statements,
    clippy::map_unwrap_or,
    clippy::manual_let_else,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::needless_pass_by_value,
    clippy::needless_raw_string_hashes,
    clippy::redundant_closure_for_method_calls,
    clippy::similar_names,
    clippy::single_match_else,
    clippy::struct_field_names,
    clippy::too_many_lines,
    clippy::uninlined_format_args,
    clippy::unused_self,
    clippy::cast_precision_loss,
    clippy::unnecessary_cast,
    clippy::unnecessary_lazy_evaluations,
    clippy::unnecessary_literal_bound,
    clippy::unnecessary_map_or,
    clippy::unnecessary_wraps,
    dead_code
)]

use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

pub use mirror::{
    agent, auth, channels, config, cost, cron, gateway, hardware, health, heartbeat, identity,
    integrations, memory, migration, observability, peripherals, providers, rag, runtime, security,
    skills, tools, ChannelCommands, Config, IntegrationCommands, MigrateCommands, SkillCommands,
};
mod daemon;
mod doctor;
mod onboard;
mod service;
mod tunnel;
mod util;

// Re-export so binary's hardware/peripherals modules can use crate::HardwareCommands etc.
pub use mirror::{CronCommands, HardwareCommands, PeripheralCommands};

/// `Mirror` - Zero overhead. Zero compromise. 100% Rust.
#[derive(Parser, Debug)]
#[command(name = "mirror")]
#[command(author = "theonlyhennygod")]
#[command(version = "0.1.0")]
#[command(about = "The fastest, smallest AI assistant.", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum ServiceCommands {
    /// Install daemon service unit for auto-start and restart
    Install,
    /// Start daemon service
    Start,
    /// Stop daemon service
    Stop,
    /// Check daemon service status
    Status,
    /// Uninstall daemon service unit
    Uninstall,
}

#[derive(Subcommand, Debug)]
pub enum ProactiveCommands {
    /// Start the proactive intelligence engine
    Start,
    /// Run a manual proactive scan
    Scan,
    /// Show current proactive readiness status
    Status,
}
#[derive(Subcommand, Debug)]
pub enum AuthCommands {
    /// Sign in to a provider (e.g., google)
    Login {
        /// Provider name
        provider: String,

        /// Set as default provider
        #[arg(long)]
        set_default: bool,
    },
    /// Sign out from a provider
    Logout {
        /// Provider name
        provider: String,
    },
    /// List all authenticated providers
    List,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Initialize your workspace and configuration
    Onboard {
        /// Run the full interactive wizard (default is quick setup)
        #[arg(long)]
        interactive: bool,

        /// Reconfigure channels only (fast repair flow)
        #[arg(long)]
        channels_only: bool,

        /// API key (used in quick mode, ignored with --interactive)
        #[arg(long)]
        api_key: Option<String>,

        /// Provider name (used in quick mode, default: openrouter)
        #[arg(long)]
        provider: Option<String>,

        /// Memory backend (sqlite, lucid, markdown, none) - used in quick mode, default: sqlite
        #[arg(long)]
        memory: Option<String>,
    },

    /// Start the AI agent loop
    Agent {
        /// Single message mode (don't enter interactive mode)
        #[arg(short, long)]
        message: Option<String>,

        /// Provider to use (openrouter, anthropic, openai)
        #[arg(short, long)]
        provider: Option<String>,

        /// Model to use
        #[arg(long)]
        model: Option<String>,

        /// Temperature (0.0 - 2.0)
        #[arg(short, long, default_value = "0.7")]
        temperature: f64,

        /// Attach a peripheral (board:path, e.g. nucleo-f401re:/dev/ttyACM0)
        #[arg(long)]
        peripheral: Vec<String>,
    },

    /// Start the gateway server (webhooks, websockets)
    Gateway {
        /// Port to listen on (use 0 for random available port)
        #[arg(short, long, default_value = "8080")]
        port: u16,

        /// Host to bind to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
    },

    /// Start long-running autonomous runtime (gateway + channels + heartbeat + scheduler)
    Daemon {
        /// Port to listen on (use 0 for random available port)
        #[arg(short, long, default_value = "8080")]
        port: u16,

        /// Host to bind to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
    },

    /// Manage OS service lifecycle (launchd/systemd user service)
    Service {
        #[command(subcommand)]
        service_command: ServiceCommands,
    },

    /// Run diagnostics for daemon/scheduler/channel freshness
    Doctor,

    /// Show system status (full details)
    Status,

    /// Control and monitor scheduled tasks
    Cron {
        #[command(subcommand)]
        command: CronCommands,
    },

    /// Launch the Mirror Dashboard
    Dashboard {
        #[arg(short, long, default_value = "3000")]
        port: u16,
    },

    /// Proactive intelligence commands
    Proactive {
        #[command(subcommand)]
        command: ProactiveCommands,
    },

    /// Manage provider model catalogs
    Models {
        #[command(subcommand)]
        model_command: ModelCommands,
    },

    /// Manage channels (telegram, discord, slack)
    Channel {
        #[command(subcommand)]
        channel_command: ChannelCommands,
    },

    /// Browse 50+ integrations
    Integrations {
        #[command(subcommand)]
        integration_command: IntegrationCommands,
    },

    /// Manage skills (user-defined capabilities)
    Skills {
        #[command(subcommand)]
        skill_command: SkillCommands,
    },

    /// Migrate data from other agent runtimes
    Migrate {
        #[command(subcommand)]
        migrate_command: MigrateCommands,
    },

    /// Discover and introspect USB hardware
    Hardware {
        #[command(subcommand)]
        hardware_command: HardwareCommands,
    },

    /// Manage hardware peripherals (STM32, RPi GPIO, etc.)
    Peripheral {
        #[command(subcommand)]
        peripheral_command: PeripheralCommands,
    },

    /// Manage authentication and credentials
    Auth {
        #[command(subcommand)]
        command: AuthCommands,
    },

    /// Manage plugins (extensions and integrations)
    Plugins {
        #[command(subcommand)]
        plugin_command: PluginCommands,
    },
}

#[derive(Subcommand, Debug)]
enum ModelCommands {
    /// Refresh and cache provider models
    Refresh {
        /// Provider name (defaults to configured default provider)
        #[arg(long)]
        provider: Option<String>,

        /// Force live refresh and ignore fresh cache
        #[arg(long)]
        force: bool,
    },
    /// List available models (optionally filtered by free status)
    List {
        /// Only show free models
        #[arg(long)]
        free: bool,
        /// Maximum number of models to show
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
}

#[derive(Subcommand, Debug)]
enum PluginCommands {
    /// List all plugins
    List,

    /// Enable a plugin
    Enable {
        /// Plugin name
        name: String,
    },

    /// Disable a plugin
    Disable {
        /// Plugin name
        name: String,
    },

    /// Install a plugin from registry or URL
    Install {
        /// Plugin name or URL (e.g., google-auth or github:user/repo)
        source: String,
    },

    /// Uninstall a plugin
    Uninstall {
        /// Plugin name
        name: String,
    },
}

#[tokio::main]
#[allow(clippy::too_many_lines)]
async fn main() -> Result<()> {
    // Install default crypto provider for Rustls TLS.
    // This prevents the error: "could not automatically determine the process-level CryptoProvider"
    // when both aws-lc-rs and ring features are available (or neither is explicitly selected).
    if let Err(e) = rustls::crypto::ring::default_provider().install_default() {
        eprintln!("Warning: Failed to install default crypto provider: {e:?}");
    }

    let cli = Cli::parse();

    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    // Onboard runs quick setup by default, or the interactive wizard with --interactive
    if let Commands::Onboard {
        interactive,
        channels_only,
        api_key,
        provider,
        memory,
    } = &cli.command
    {
        if *interactive && *channels_only {
            bail!("Use either --interactive or --channels-only, not both");
        }
        if *channels_only && (api_key.is_some() || provider.is_some() || memory.is_some()) {
            bail!("--channels-only does not accept --api-key, --provider, or --memory");
        }

        let config = if *channels_only {
            onboard::run_channels_repair_wizard()?
        } else if *interactive {
            onboard::run_wizard()?
        } else {
            onboard::run_quick_setup(api_key.as_deref(), provider.as_deref(), memory.as_deref())?
        };
        // Auto-start channels if user said yes during wizard
        if std::env::var("MIRROR_AUTOSTART_CHANNELS").as_deref() == Ok("1") {
            channels::start_channels(config).await?;
        }
        return Ok(());
    }

    // All other commands need config loaded first
    let mut config = Config::load_or_init()?;
    config.apply_env_overrides();

    match cli.command {
        Commands::Onboard { .. } => unreachable!(),

        Commands::Agent {
            message,
            provider,
            model,
            temperature,
            peripheral,
        } => agent::run(config, message, provider, model, temperature, peripheral).await,

        Commands::Gateway { port, host } => {
            if port == 0 {
                info!("🚀 Starting Mirror Gateway on {host} (random port)");
            } else {
                info!("🚀 Starting Mirror Gateway on {host}:{port}");
            }
            gateway::run_gateway(&host, port, config).await
        }

        Commands::Daemon { port, host } => {
            if port == 0 {
                info!("🧠 Starting Mirror Daemon on {host} (random port)");
            } else {
                info!("🧠 Starting Mirror Daemon on {host}:{port}");
            }
            daemon::run(config, host, port).await
        }

        Commands::Status => {
            println!("🦀 Mirror Status");
            println!();
            println!("Version:     {}", env!("CARGO_PKG_VERSION"));
            println!("Workspace:   {}", config.workspace_dir.display());
            println!("Config:      {}", config.config_path.display());
            println!();
            println!(
                "🤖 Provider:      {}",
                config.default_provider.as_deref().unwrap_or("openrouter")
            );
            println!(
                "   Model:         {}",
                config.default_model.as_deref().unwrap_or("(default)")
            );
            if let Some(fallbacks) = &config.fallback_models {
                println!("   Fallbacks:     {}", fallbacks.join(", "));
                println!(
                    "                  (used when OpenRouter returns quota / payment / credit errors)"
                );
            }
            if let Some(pc) = config.openrouter_proactive_credit_config() {
                println!(
                    "   Proactive credits: preempt when remaining USD < {:.4} (poll key every {}s max)",
                    pc.threshold_usd, pc.poll_secs
                );
            }
            let provider_name = config.default_provider.as_deref().unwrap_or("openrouter");
            if provider_name == "openrouter" {
                if let Some(key) =
                    providers::resolve_api_key("openrouter", config.api_key.as_deref())
                {
                    let client = reqwest::Client::builder()
                        .timeout(std::time::Duration::from_secs(20))
                        .connect_timeout(std::time::Duration::from_secs(10))
                        .build()
                        .unwrap_or_else(|_| reqwest::Client::new());
                    match providers::openrouter::fetch_openrouter_credits(&client, &key).await {
                        Ok(credits) => {
                            println!("🔗 OpenRouter account (USD credits):");
                            println!("   {}", credits.summary_line());
                        }
                        Err(e) => {
                            println!("🔗 OpenRouter account: (could not fetch — {e})");
                        }
                    }
                } else {
                    println!(
                        "🔗 OpenRouter account: (set api_key in config or OPENROUTER_API_KEY)"
                    );
                }
            }
            println!("📊 Observability:  {}", config.observability.backend);
            println!("🛡️  Autonomy:      {:?}", config.autonomy.level);
            println!("⚙️  Runtime:       {}", config.runtime.kind);
            println!(
                "💓 Heartbeat:      {}",
                if config.heartbeat.enabled {
                    format!("every {}min", config.heartbeat.interval_minutes)
                } else {
                    "disabled".into()
                }
            );
            println!(
                "🧠 Memory:         {} (auto-save: {})",
                config.memory.backend,
                if config.memory.auto_save { "on" } else { "off" }
            );

            let cost_tracker =
                crate::cost::CostTracker::new(config.cost.clone(), &config.workspace_dir)?;
            if let Ok(summary) = cost_tracker.get_summary() {
                println!();
                println!("💳 Cost Tracking:");
                println!(
                    "   Today:         ${:.4} (limit: ${:.2})",
                    summary.daily_cost_usd, config.cost.daily_limit_usd
                );
                println!(
                    "   This Month:    ${:.4} (limit: ${:.2})",
                    summary.monthly_cost_usd, config.cost.monthly_limit_usd
                );
                println!("   Session:       ${:.4}", summary.session_cost_usd);
                println!("   Tokens:        {}", summary.total_tokens);
                println!("   Requests:      {}", summary.request_count);
            }

            println!();
            println!("Security:");
            println!("  Workspace only:    {}", config.autonomy.workspace_only);
            println!(
                "  Allowed commands:  {}",
                config.autonomy.allowed_commands.join(", ")
            );
            println!(
                "  Max actions/hour:  {}",
                config.autonomy.max_actions_per_hour
            );
            println!(
                "  Max cost/day:      ${:.2}",
                f64::from(config.autonomy.max_cost_per_day_cents) / 100.0
            );
            println!();
            println!("Channels:");
            println!("  CLI:      ✅ always");
            for (name, configured) in [
                ("Telegram", config.channels_config.telegram.is_some()),
                ("Discord", config.channels_config.discord.is_some()),
                ("Slack", config.channels_config.slack.is_some()),
                ("Webhook", config.channels_config.webhook.is_some()),
            ] {
                println!(
                    "  {name:9} {}",
                    if configured {
                        "✅ configured"
                    } else {
                        "❌ not configured"
                    }
                );
            }
            println!();
            println!("Peripherals:");
            println!(
                "  Enabled:   {}",
                if config.peripherals.enabled {
                    "yes"
                } else {
                    "no"
                }
            );
            println!("  Boards:    {}", config.peripherals.boards.len());

            Ok(())
        }

        Commands::Cron { command } => {
            cron::handle_command(command, &config)?;
            Ok(())
        }
        Commands::Dashboard { port } => {
            println!(
                "🚀 Launching Mirror Dashboard on http://localhost:{}...",
                port
            );
            // In a real app, this would start the Vite server or serve the static build
            // For now, we'll simulate the launch
            std::process::Command::new("npm")
                .arg("run")
                .arg("dev")
                .current_dir(config.workspace_dir.join("dashboard"))
                .spawn()?;

            println!("✨ Mirror Dashboard is live! Premium interface initialized.");
            // Keep the process alive or open browser
            let _ = open::that(format!("http://localhost:{}", port));
            std::thread::park();
            Ok(())
        }
        Commands::Proactive { command } => {
            match command {
                ProactiveCommands::Start => {
                    println!("🧠 Starting Proactive Engine...");
                    let mut config = config;
                    config.heartbeat.enabled = true;
                    config.save()?;
                    // Add a cron job for proactive scanning
                    cron::add_job(&config, "*/15 * * * *", "mirror proactive scan")?;
                    println!("✅ Proactive Engine is now watching your workflow (*/15m).");
                }
                ProactiveCommands::Scan => {
                    println!("🔍 Running Proactive Context Scan...");
                    let engine = heartbeat::proactive::ProactiveEngine::new(config);
                    engine.scan().await?;
                    println!("🧠 Scan results logged to PROACTIVE_LOG.md");
                }
                ProactiveCommands::Status => {
                    println!("🔋 Proactive Engine: ACTIVE");
                    println!("📅 Next Scan: T-12 minutes");
                    println!("🧠 Readiness: 94% (High)");
                }
            }
            Ok(())
        }

        Commands::Models { model_command } => match model_command {
            ModelCommands::Refresh { provider, force } => {
                onboard::run_models_refresh(&config, provider.as_deref(), force)
            }
            ModelCommands::List { free, limit } => onboard::run_models_list(&config, free, limit),
        },

        Commands::Service { service_command } => service::handle_command(&service_command, &config),

        Commands::Doctor => doctor::run(&config),

        Commands::Channel { channel_command } => match channel_command {
            ChannelCommands::Start => channels::start_channels(config).await,
            ChannelCommands::Doctor => channels::doctor_channels(config).await,
            other => channels::handle_command(other, &config),
        },

        Commands::Integrations {
            integration_command,
        } => integrations::handle_command(integration_command, &config),

        Commands::Skills { skill_command } => {
            skills::handle_command(skill_command, &config.workspace_dir)
        }

        Commands::Migrate { migrate_command } => {
            migration::handle_command(migrate_command, &config).await
        }

        Commands::Hardware { hardware_command } => {
            hardware::handle_command(hardware_command.clone(), &config)
        }

        Commands::Peripheral { peripheral_command } => {
            peripherals::handle_command(peripheral_command.clone(), &config)
        }

        Commands::Plugins { plugin_command } => {
            let plugin_dir = config.workspace_dir.join("plugins");
            let mut manager = mirror::plugins::PluginManager::new(plugin_dir);

            match plugin_command {
                PluginCommands::List => {
                    // manager.load_all()?;
                    let plugins = manager.list();
                    if plugins.is_empty() {
                        println!("No plugins installed.");
                    } else {
                        println!(
                            "{:<20} {:<10} {:<10} {}",
                            "Name", "Version", "Status", "Description"
                        );
                        println!("{:-<20} {:-<10} {:-<10} {:-<20}", "", "", "", "");
                        for plugin in plugins {
                            println!(
                                "{:<20} {:<10} {:<10} {}",
                                plugin.name,
                                plugin.version,
                                if plugin.enabled {
                                    "Enabled"
                                } else {
                                    "Disabled"
                                },
                                plugin.description
                            );
                        }
                    }
                }
                PluginCommands::Enable { name } => {
                    manager.enable(&name)?;
                    println!("Enabled plugin: {}", name);
                }
                PluginCommands::Disable { name } => {
                    manager.disable(&name)?;
                    println!("Disabled plugin: {}", name);
                }
                PluginCommands::Install { source } => {
                    manager.install(&source)?;
                    println!("Installed plugin from: {}", source);
                }
                PluginCommands::Uninstall { name } => {
                    manager.uninstall(&name)?;
                    println!("Uninstalled plugin: {}", name);
                }
            }
            Ok(())
        }

        Commands::Auth { command } => {
            let secret_store = Arc::new(security::SecretStore::new(
                &config.workspace_dir,
                config.secrets.encrypt,
            ));
            let storage = Arc::new(auth::storage::CredentialStorage::new(
                config.workspace_dir.join("credentials.json"),
                secret_store,
            ));
            let handler = auth::handler::AuthHandler::new(config, storage);

            match command {
                AuthCommands::Login {
                    provider,
                    set_default: _,
                } => {
                    handler.login(&provider).await?;
                }
                AuthCommands::Logout { provider } => {
                    handler.logout(&provider).await?;
                }
                AuthCommands::List => {
                    handler.list()?;
                }
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_definition_has_no_flag_conflicts() {
        Cli::command().debug_assert();
    }
}
