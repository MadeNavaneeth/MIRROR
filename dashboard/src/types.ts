export interface HealthStatus {
  status: string
  paired: boolean
  runtime: {
    components: Record<string, string>
    uptime_secs: number
    version: string
  }
}

export interface ChatMessage {
  role: 'user' | 'assistant'
  content: string
  timestamp: string
}

export interface CostSummary {
  session_cost_usd: number
  daily_cost_usd: number
  monthly_cost_usd: number
  total_tokens: number
  request_count: number
}

export interface SkillInfo {
  version: string
  installedAt: number
}

export interface SkillsData {
  skills: Record<string, SkillInfo>
}

export interface AuthStatus {
  authenticated_providers: string[]
}

export interface ObservabilityConfig {
  backend: string
  otel_endpoint: string | null
  otel_service_name: string | null
}

export interface AutonomyConfig {
  level: string
  workspace_only: boolean
  allowed_commands: string[]
  forbidden_paths: string[]
  max_actions_per_hour: number
  max_cost_per_day_cents: number
  require_approval_for_medium_risk: boolean
  block_high_risk_commands: boolean
}

export interface DockerRuntimeConfig {
  image: string
  network: string
  memory_limit_mb: number | null
  cpu_limit: number | null
  read_only_rootfs: boolean
  mount_workspace: boolean
  allowed_workspace_roots: string[]
}

export interface RuntimeConfig {
  kind: string
  docker: DockerRuntimeConfig
}

export interface ModelFallbackEntry {
  model: string
  fallbacks: string[]
}

export interface ReliabilityConfig {
  provider_retries: number
  provider_backoff_ms: number
  fallback_providers: string[]
  api_keys: string[]
  model_fallbacks: ModelFallbackEntry[]
  channel_initial_backoff_secs: number
  channel_max_backoff_secs: number
  scheduler_poll_secs: number
  scheduler_retries: number
}

export interface SchedulerConfig {
  enabled: boolean
  max_tasks: number
  max_concurrent: number
}

export interface AgentConfig {
  compact_context: boolean
  max_tool_iterations: number
  max_history_messages: number
  parallel_tools: boolean
  tool_dispatcher: string
}

export interface ModelRouteConfig {
  hint: string
  provider: string
  model: string
  api_key: string | null
}

export interface HeartbeatConfig {
  enabled: boolean
  interval_minutes: number
}

export interface CloudflareTunnelConfig {
  token: string
}

export interface TailscaleTunnelConfig {
  funnel: boolean
  hostname: string | null
}

export interface NgrokTunnelConfig {
  auth_token: string
  domain: string | null
}

export interface CustomTunnelConfig {
  start_command: string
  health_url: string | null
  url_pattern: string | null
}

export interface TunnelConfig {
  provider: string
  cloudflare: CloudflareTunnelConfig | null
  tailscale: TailscaleTunnelConfig | null
  ngrok: NgrokTunnelConfig | null
  custom: CustomTunnelConfig | null
}

export interface TelegramChannelConfig {
  bot_token: string
  allowed_users: string[]
}

export interface DiscordChannelConfig {
  bot_token: string
  guild_id: string | null
  allowed_users: string[]
  listen_to_bots: boolean
}

export interface SlackChannelConfig {
  bot_token: string
  app_token: string | null
  channel_id: string | null
  allowed_users: string[]
}

export interface WebhookChannelConfig {
  port: number
  secret: string | null
}

export interface IMessageChannelConfig {
  allowed_contacts: string[]
}

export interface MatrixChannelConfig {
  homeserver: string
  access_token: string
  room_id: string
  allowed_users: string[]
}

export interface WhatsAppChannelConfig {
  access_token: string
  phone_number_id: string
  verify_token: string
  app_secret: string | null
  allowed_numbers: string[]
}

export interface EmailChannelConfig {
  imap_host: string
  imap_port: number
  imap_folder: string
  smtp_host: string
  smtp_port: number
  smtp_tls: boolean
  username: string
  password: string
  from_address: string
  poll_interval_secs: number
  allowed_senders: string[]
}

export interface IrcChannelConfig {
  server: string
  port: number
  nickname: string
  username: string | null
  channels: string[]
  allowed_users: string[]
  server_password: string | null
  nickserv_password: string | null
  sasl_password: string | null
  verify_tls: boolean | null
}

export interface LarkChannelConfig {
  app_id: string
  app_secret: string
  encrypt_key: string | null
  verification_token: string | null
  allowed_users: string[]
  use_feishu: boolean
}

export interface DingTalkChannelConfig {
  client_id: string
  client_secret: string
  allowed_users: string[]
}

export interface ChannelsConfig {
  cli: boolean
  telegram: TelegramChannelConfig | null
  discord: DiscordChannelConfig | null
  slack: SlackChannelConfig | null
  webhook: WebhookChannelConfig | null
  imessage: IMessageChannelConfig | null
  matrix: MatrixChannelConfig | null
  whatsapp: WhatsAppChannelConfig | null
  email: EmailChannelConfig | null
  irc: IrcChannelConfig | null
  lark: LarkChannelConfig | null
  dingtalk: DingTalkChannelConfig | null
}

export interface MemoryConfig {
  backend: string
  auto_save: boolean
  hygiene_enabled: boolean
  archive_after_days: number
  purge_after_days: number
  conversation_retention_days: number
  embedding_provider: string
  embedding_model: string
  embedding_dimensions: number
  vector_weight: number
  keyword_weight: number
  embedding_cache_size: number
  chunk_max_tokens: number
  response_cache_enabled: boolean
  response_cache_ttl_minutes: number
  response_cache_max_entries: number
  snapshot_enabled: boolean
  snapshot_on_hygiene: boolean
  auto_hydrate: boolean
}

export interface GatewayConfig {
  port: number
  host: string
  require_pairing: boolean
  allow_public_bind: boolean
  paired_tokens: string[]
  pair_rate_limit_per_minute: number
  webhook_rate_limit_per_minute: number
  idempotency_ttl_secs: number
}

export interface ComposioConfig {
  enabled: boolean
  api_key: string | null
  entity_id: string
}

export interface SecretsConfig {
  encrypt: boolean
}

export interface BrowserComputerUseConfig {
  endpoint: string
  api_key: string | null
  timeout_ms: number
  allow_remote_endpoint: boolean
  window_allowlist: string[]
  max_coordinate_x: number | null
  max_coordinate_y: number | null
}

export interface BrowserConfig {
  enabled: boolean
  allowed_domains: string[]
  session_name: string | null
  backend: string
  native_headless: boolean
  native_webdriver_url: string
  native_chrome_path: string | null
  computer_use: BrowserComputerUseConfig
}

export interface HttpRequestConfig {
  enabled: boolean
  allowed_domains: string[]
  max_response_size: number
  timeout_secs: number
}

export interface IdentityConfig {
  format: string
  aieos_path: string | null
  aieos_inline: string | null
}

export interface PricingEntry {
  model: string
  input: number
  output: number
}

export interface CostConfig {
  enabled: boolean
  daily_limit_usd: number
  monthly_limit_usd: number
  warn_at_percent: number
  allow_override: boolean
  prices: PricingEntry[]
}

export interface ReasoningConfig {
  cot_enabled: boolean
  self_consistency_enabled: boolean
  num_samples: number
  sampling_temperature: number
  consensus_threshold: number
}

export interface PeripheralBoardConfig {
  board: string
  transport: string
  path: string | null
  baud: number
}

export interface PeripheralsConfig {
  enabled: boolean
  boards: PeripheralBoardConfig[]
  datasheet_dir: string | null
}

export interface DelegateAgentConfig {
  name: string
  provider: string
  model: string
  system_prompt: string | null
  api_key: string | null
  temperature: number | null
  max_depth: number
}

export interface HardwareConfig {
  enabled: boolean
  transport: string
  serial_port: string | null
  baud_rate: number
  probe_target: string | null
  workspace_datasheets: boolean
}

export interface DashboardConfig {
  workspace_dir?: string
  config_path?: string
  api_key: string | null
  default_provider: string | null
  default_model: string | null
  default_temperature: number
  observability: ObservabilityConfig
  autonomy: AutonomyConfig
  runtime: RuntimeConfig
  reliability: ReliabilityConfig
  scheduler: SchedulerConfig
  agent: AgentConfig
  model_routes: ModelRouteConfig[]
  heartbeat: HeartbeatConfig
  channels_config: ChannelsConfig
  memory: MemoryConfig
  tunnel: TunnelConfig
  gateway: GatewayConfig
  composio: ComposioConfig
  secrets: SecretsConfig
  browser: BrowserConfig
  http_request: HttpRequestConfig
  identity: IdentityConfig
  cost: CostConfig
  reasoning: ReasoningConfig
  peripherals: PeripheralsConfig
  agents: DelegateAgentConfig[]
  hardware: HardwareConfig
}
