import type {
  BrowserConfig,
  ChannelsConfig,
  CloudflareTunnelConfig,
  ComposioConfig,
  CostConfig,
  CustomTunnelConfig,
  DashboardConfig,
  DelegateAgentConfig,
  DingTalkChannelConfig,
  DiscordChannelConfig,
  DockerRuntimeConfig,
  EmailChannelConfig,
  GatewayConfig,
  HardwareConfig,
  HeartbeatConfig,
  HttpRequestConfig,
  IMessageChannelConfig,
  IdentityConfig,
  IrcChannelConfig,
  LarkChannelConfig,
  MatrixChannelConfig,
  MemoryConfig,
  ModelFallbackEntry,
  ModelRouteConfig,
  NgrokTunnelConfig,
  ObservabilityConfig,
  PeripheralBoardConfig,
  PeripheralsConfig,
  PricingEntry,
  ReasoningConfig,
  ReliabilityConfig,
  RuntimeConfig,
  SchedulerConfig,
  SecretsConfig,
  SlackChannelConfig,
  TailscaleTunnelConfig,
  TelegramChannelConfig,
  TunnelConfig,
  WebhookChannelConfig,
  WhatsAppChannelConfig,
  AgentConfig,
  AutonomyConfig,
} from './types'

const SECRET_STRING_KEYS = new Set([
  'access_token',
  'api_key',
  'app_secret',
  'app_token',
  'auth_token',
  'bot_token',
  'client_secret',
  'encrypt_key',
  'nickserv_password',
  'password',
  'sasl_password',
  'secret',
  'server_password',
  'token',
  'verification_token',
  'verify_token',
])
const SECRET_ARRAY_KEYS = new Set(['api_keys', 'paired_tokens'])

const MASKED_SECRET_VALUE = '***'

type RawPricingValue = { input?: number; output?: number }
type RawModelFallbacks = Record<string, string[]> | ModelFallbackEntry[] | undefined
type RawPrices = Record<string, RawPricingValue> | PricingEntry[] | undefined
type RawAgents =
  | Record<string, Omit<DelegateAgentConfig, 'name'>>
  | DelegateAgentConfig[]
  | undefined

export interface RawDashboardConfig
  extends Partial<Omit<DashboardConfig, 'agents' | 'cost' | 'reliability'>> {
  reliability?: Partial<Omit<ReliabilityConfig, 'model_fallbacks'>> & {
    model_fallbacks?: RawModelFallbacks
  }
  cost?: Partial<Omit<CostConfig, 'prices'>> & {
    prices?: RawPrices
  }
  agents?: RawAgents
}

export function isMaskedSecret(value: string | null | undefined): boolean {
  return value === MASKED_SECRET_VALUE
}

export function cloneConfig<T>(value: T): T {
  return JSON.parse(JSON.stringify(value)) as T
}

export function parseList(value: string): string[] {
  return value
    .split('\n')
    .map((entry) => entry.trim())
    .filter(Boolean)
}

export function formatList(value: string[]): string {
  return value.join('\n')
}

function normalizeOptionalText(value: string | null | undefined): string | null {
  const trimmed = value?.trim()
  return trimmed ? trimmed : null
}

function normalizeOptionalNumber(value: number | null | undefined): number | null {
  return Number.isFinite(value) ? Number(value) : null
}

function normalizeFallbackEntries(value: RawModelFallbacks): ModelFallbackEntry[] {
  if (!value) {
    return []
  }

  if (Array.isArray(value)) {
    return value.map((entry) => ({
      model: entry.model ?? '',
      fallbacks: entry.fallbacks ?? [],
    }))
  }

  return Object.entries(value).map(([model, fallbacks]) => ({
    model,
    fallbacks,
  }))
}

function normalizePricingEntries(value: RawPrices): PricingEntry[] {
  if (!value) {
    return []
  }

  if (Array.isArray(value)) {
    return value.map((entry) => ({
      model: entry.model ?? '',
      input: entry.input ?? 0,
      output: entry.output ?? 0,
    }))
  }

  return Object.entries(value).map(([model, pricing]) => ({
    model,
    input: pricing.input ?? 0,
    output: pricing.output ?? 0,
  }))
}

function normalizeAgents(value: RawAgents): DelegateAgentConfig[] {
  if (!value) {
    return []
  }

  if (Array.isArray(value)) {
    return value.map((entry) => ({
      name: entry.name ?? '',
      provider: entry.provider ?? '',
      model: entry.model ?? '',
      system_prompt: entry.system_prompt ?? null,
      api_key: entry.api_key ?? null,
      temperature: normalizeOptionalNumber(entry.temperature),
      max_depth: entry.max_depth ?? 3,
    }))
  }

  return Object.entries(value).map(([name, agent]) => ({
    name,
    provider: agent.provider ?? '',
    model: agent.model ?? '',
    system_prompt: agent.system_prompt ?? null,
    api_key: agent.api_key ?? null,
    temperature: normalizeOptionalNumber(agent.temperature),
    max_depth: agent.max_depth ?? 3,
  }))
}

export function createObservabilityConfig(): ObservabilityConfig {
  return {
    backend: 'none',
    otel_endpoint: null,
    otel_service_name: null,
  }
}

export function createAutonomyConfig(): AutonomyConfig {
  return {
    level: 'supervised',
    workspace_only: true,
    allowed_commands: [
      'git',
      'npm',
      'cargo',
      'ls',
      'cat',
      'grep',
      'find',
      'echo',
      'pwd',
      'wc',
      'head',
      'tail',
    ],
    forbidden_paths: ['/etc', '/root', '/sys', '/proc', '~/.ssh', '~/.gnupg', '~/.aws', '~/.config'],
    max_actions_per_hour: 100,
    max_cost_per_day_cents: 500,
    require_approval_for_medium_risk: true,
    block_high_risk_commands: true,
  }
}

export function createDockerRuntimeConfig(): DockerRuntimeConfig {
  return {
    image: 'alpine:3.20',
    network: 'none',
    memory_limit_mb: 512,
    cpu_limit: 1,
    read_only_rootfs: true,
    mount_workspace: true,
    allowed_workspace_roots: [],
  }
}

export function createRuntimeConfig(): RuntimeConfig {
  return {
    kind: 'native',
    docker: createDockerRuntimeConfig(),
  }
}

export function createReliabilityConfig(): ReliabilityConfig {
  return {
    provider_retries: 2,
    provider_backoff_ms: 500,
    fallback_providers: [],
    api_keys: [],
    model_fallbacks: [],
    channel_initial_backoff_secs: 2,
    channel_max_backoff_secs: 60,
    scheduler_poll_secs: 15,
    scheduler_retries: 2,
  }
}

export function createSchedulerConfig(): SchedulerConfig {
  return {
    enabled: true,
    max_tasks: 64,
    max_concurrent: 4,
  }
}

export function createAgentConfig(): AgentConfig {
  return {
    compact_context: false,
    max_tool_iterations: 10,
    max_history_messages: 50,
    parallel_tools: false,
    tool_dispatcher: 'auto',
  }
}

export function createModelRoute(): ModelRouteConfig {
  return {
    hint: '',
    provider: '',
    model: '',
    api_key: null,
  }
}

export function createModelFallbackEntry(): ModelFallbackEntry {
  return {
    model: '',
    fallbacks: [],
  }
}

export function createHeartbeatConfig(): HeartbeatConfig {
  return {
    enabled: false,
    interval_minutes: 30,
  }
}

export function createCloudflareTunnelConfig(): CloudflareTunnelConfig {
  return {
    token: '',
  }
}

export function createTailscaleTunnelConfig(): TailscaleTunnelConfig {
  return {
    funnel: false,
    hostname: null,
  }
}

export function createNgrokTunnelConfig(): NgrokTunnelConfig {
  return {
    auth_token: '',
    domain: null,
  }
}

export function createCustomTunnelConfig(): CustomTunnelConfig {
  return {
    start_command: '',
    health_url: null,
    url_pattern: null,
  }
}

export function createTunnelConfig(): TunnelConfig {
  return {
    provider: 'none',
    cloudflare: null,
    tailscale: null,
    ngrok: null,
    custom: null,
  }
}

export function createTelegramChannelConfig(): TelegramChannelConfig {
  return {
    bot_token: '',
    allowed_users: [],
  }
}

export function createDiscordChannelConfig(): DiscordChannelConfig {
  return {
    bot_token: '',
    guild_id: null,
    allowed_users: [],
    listen_to_bots: false,
  }
}

export function createSlackChannelConfig(): SlackChannelConfig {
  return {
    bot_token: '',
    app_token: null,
    channel_id: null,
    allowed_users: [],
  }
}

export function createWebhookChannelConfig(): WebhookChannelConfig {
  return {
    port: 8080,
    secret: null,
  }
}

export function createIMessageChannelConfig(): IMessageChannelConfig {
  return {
    allowed_contacts: [],
  }
}

export function createMatrixChannelConfig(): MatrixChannelConfig {
  return {
    homeserver: '',
    access_token: '',
    room_id: '',
    allowed_users: [],
  }
}

export function createWhatsAppChannelConfig(): WhatsAppChannelConfig {
  return {
    access_token: '',
    phone_number_id: '',
    verify_token: '',
    app_secret: null,
    allowed_numbers: [],
  }
}

export function createEmailChannelConfig(): EmailChannelConfig {
  return {
    imap_host: '',
    imap_port: 993,
    imap_folder: 'INBOX',
    smtp_host: '',
    smtp_port: 587,
    smtp_tls: true,
    username: '',
    password: '',
    from_address: '',
    poll_interval_secs: 60,
    allowed_senders: [],
  }
}

export function createIrcChannelConfig(): IrcChannelConfig {
  return {
    server: '',
    port: 6697,
    nickname: '',
    username: null,
    channels: [],
    allowed_users: [],
    server_password: null,
    nickserv_password: null,
    sasl_password: null,
    verify_tls: true,
  }
}

export function createLarkChannelConfig(): LarkChannelConfig {
  return {
    app_id: '',
    app_secret: '',
    encrypt_key: null,
    verification_token: null,
    allowed_users: [],
    use_feishu: false,
  }
}

export function createDingTalkChannelConfig(): DingTalkChannelConfig {
  return {
    client_id: '',
    client_secret: '',
    allowed_users: [],
  }
}

export function createChannelsConfig(): ChannelsConfig {
  return {
    cli: true,
    telegram: null,
    discord: null,
    slack: null,
    webhook: null,
    imessage: null,
    matrix: null,
    whatsapp: null,
    email: null,
    irc: null,
    lark: null,
    dingtalk: null,
  }
}

export function createMemoryConfig(): MemoryConfig {
  return {
    backend: 'sqlite',
    auto_save: true,
    hygiene_enabled: true,
    archive_after_days: 7,
    purge_after_days: 30,
    conversation_retention_days: 30,
    embedding_provider: 'openai',
    embedding_model: 'text-embedding-3-small',
    embedding_dimensions: 1536,
    vector_weight: 0.7,
    keyword_weight: 0.3,
    embedding_cache_size: 10000,
    chunk_max_tokens: 512,
    response_cache_enabled: false,
    response_cache_ttl_minutes: 60,
    response_cache_max_entries: 5000,
    snapshot_enabled: false,
    snapshot_on_hygiene: false,
    auto_hydrate: true,
  }
}

export function createGatewayConfig(): GatewayConfig {
  return {
    port: 3000,
    host: '127.0.0.1',
    require_pairing: true,
    allow_public_bind: false,
    paired_tokens: [],
    pair_rate_limit_per_minute: 10,
    webhook_rate_limit_per_minute: 60,
    idempotency_ttl_secs: 300,
  }
}

export function createComposioConfig(): ComposioConfig {
  return {
    enabled: false,
    api_key: null,
    entity_id: 'default',
  }
}

export function createSecretsConfig(): SecretsConfig {
  return {
    encrypt: true,
  }
}

export function createBrowserConfig(): BrowserConfig {
  return {
    enabled: false,
    allowed_domains: [],
    session_name: null,
    backend: 'agent_browser',
    native_headless: true,
    native_webdriver_url: 'http://127.0.0.1:9515',
    native_chrome_path: null,
    computer_use: {
      endpoint: 'http://127.0.0.1:8787/v1/actions',
      api_key: null,
      timeout_ms: 15000,
      allow_remote_endpoint: false,
      window_allowlist: [],
      max_coordinate_x: null,
      max_coordinate_y: null,
    },
  }
}

export function createHttpRequestConfig(): HttpRequestConfig {
  return {
    enabled: false,
    allowed_domains: [],
    max_response_size: 1000000,
    timeout_secs: 30,
  }
}

export function createIdentityConfig(): IdentityConfig {
  return {
    format: 'openclaw',
    aieos_path: null,
    aieos_inline: null,
  }
}

export function createPricingEntry(): PricingEntry {
  return {
    model: '',
    input: 0,
    output: 0,
  }
}

export function createCostConfig(): CostConfig {
  return {
    enabled: true,
    daily_limit_usd: 10,
    monthly_limit_usd: 100,
    warn_at_percent: 80,
    allow_override: false,
    prices: [
      { model: 'anthropic/claude-sonnet-4-20250514', input: 3, output: 15 },
      { model: 'openai/gpt-4o', input: 5, output: 15 },
      { model: 'google/gemini-2.0-flash', input: 0.1, output: 0.4 },
    ],
  }
}

export function createReasoningConfig(): ReasoningConfig {
  return {
    cot_enabled: true,
    self_consistency_enabled: false,
    num_samples: 3,
    sampling_temperature: 0.8,
    consensus_threshold: 0.6,
  }
}

export function createPeripheralBoardConfig(): PeripheralBoardConfig {
  return {
    board: '',
    transport: 'serial',
    path: null,
    baud: 115200,
  }
}

export function createPeripheralsConfig(): PeripheralsConfig {
  return {
    enabled: false,
    boards: [],
    datasheet_dir: null,
  }
}

export function createDelegateAgentConfig(): DelegateAgentConfig {
  return {
    name: '',
    provider: '',
    model: '',
    system_prompt: null,
    api_key: null,
    temperature: null,
    max_depth: 3,
  }
}

export function createHardwareConfig(): HardwareConfig {
  return {
    enabled: false,
    transport: 'none',
    serial_port: null,
    baud_rate: 115200,
    probe_target: null,
    workspace_datasheets: false,
  }
}

export function createDefaultConfig(): DashboardConfig {
  return {
    api_key: null,
    default_provider: 'openrouter',
    default_model: 'anthropic/claude-sonnet-4',
    default_temperature: 0.7,
    observability: createObservabilityConfig(),
    autonomy: createAutonomyConfig(),
    runtime: createRuntimeConfig(),
    reliability: createReliabilityConfig(),
    scheduler: createSchedulerConfig(),
    agent: createAgentConfig(),
    model_routes: [],
    heartbeat: createHeartbeatConfig(),
    channels_config: createChannelsConfig(),
    memory: createMemoryConfig(),
    tunnel: createTunnelConfig(),
    gateway: createGatewayConfig(),
    composio: createComposioConfig(),
    secrets: createSecretsConfig(),
    browser: createBrowserConfig(),
    http_request: createHttpRequestConfig(),
    identity: createIdentityConfig(),
    cost: createCostConfig(),
    reasoning: createReasoningConfig(),
    peripherals: createPeripheralsConfig(),
    agents: [],
    hardware: createHardwareConfig(),
  }
}

export function normalizeConfig(raw: RawDashboardConfig | null | undefined): DashboardConfig {
  const defaults = createDefaultConfig()
  const source = raw ?? {}

  return {
    ...defaults,
    workspace_dir: source.workspace_dir,
    config_path: source.config_path,
    api_key: source.api_key ?? defaults.api_key,
    default_provider: source.default_provider ?? defaults.default_provider,
    default_model: source.default_model ?? defaults.default_model,
    default_temperature: source.default_temperature ?? defaults.default_temperature,
    observability: {
      ...defaults.observability,
      ...source.observability,
    },
    autonomy: {
      ...defaults.autonomy,
      ...source.autonomy,
      allowed_commands: source.autonomy?.allowed_commands ?? defaults.autonomy.allowed_commands,
      forbidden_paths: source.autonomy?.forbidden_paths ?? defaults.autonomy.forbidden_paths,
    },
    runtime: {
      ...defaults.runtime,
      ...source.runtime,
      docker: {
        ...defaults.runtime.docker,
        ...source.runtime?.docker,
        allowed_workspace_roots:
          source.runtime?.docker?.allowed_workspace_roots ??
          defaults.runtime.docker.allowed_workspace_roots,
      },
    },
    reliability: {
      ...defaults.reliability,
      ...source.reliability,
      fallback_providers:
        source.reliability?.fallback_providers ?? defaults.reliability.fallback_providers,
      api_keys: source.reliability?.api_keys ?? defaults.reliability.api_keys,
      model_fallbacks: normalizeFallbackEntries(source.reliability?.model_fallbacks),
    },
    scheduler: {
      ...defaults.scheduler,
      ...source.scheduler,
    },
    agent: {
      ...defaults.agent,
      ...source.agent,
    },
    model_routes: (source.model_routes ?? []).map((route) => ({
      ...createModelRoute(),
      ...route,
      api_key: route.api_key ?? null,
    })),
    heartbeat: {
      ...defaults.heartbeat,
      ...source.heartbeat,
    },
    channels_config: {
      ...defaults.channels_config,
      ...source.channels_config,
    },
    memory: {
      ...defaults.memory,
      ...source.memory,
    },
    tunnel: {
      ...defaults.tunnel,
      ...source.tunnel,
      cloudflare: source.tunnel?.cloudflare ?? defaults.tunnel.cloudflare,
      tailscale: source.tunnel?.tailscale ?? defaults.tunnel.tailscale,
      ngrok: source.tunnel?.ngrok ?? defaults.tunnel.ngrok,
      custom: source.tunnel?.custom ?? defaults.tunnel.custom,
    },
    gateway: {
      ...defaults.gateway,
      ...source.gateway,
      paired_tokens: source.gateway?.paired_tokens ?? defaults.gateway.paired_tokens,
    },
    composio: {
      ...defaults.composio,
      ...source.composio,
    },
    secrets: {
      ...defaults.secrets,
      ...source.secrets,
    },
    browser: {
      ...defaults.browser,
      ...source.browser,
      allowed_domains: source.browser?.allowed_domains ?? defaults.browser.allowed_domains,
      computer_use: {
        ...defaults.browser.computer_use,
        ...source.browser?.computer_use,
        window_allowlist:
          source.browser?.computer_use?.window_allowlist ??
          defaults.browser.computer_use.window_allowlist,
      },
    },
    http_request: {
      ...defaults.http_request,
      ...source.http_request,
      allowed_domains:
        source.http_request?.allowed_domains ?? defaults.http_request.allowed_domains,
    },
    identity: {
      ...defaults.identity,
      ...source.identity,
    },
    cost: {
      ...defaults.cost,
      ...source.cost,
      prices: normalizePricingEntries(source.cost?.prices),
    },
    reasoning: {
      ...defaults.reasoning,
      ...source.reasoning,
    },
    peripherals: {
      ...defaults.peripherals,
      ...source.peripherals,
      boards: source.peripherals?.boards ?? defaults.peripherals.boards,
    },
    agents: normalizeAgents(source.agents),
    hardware: {
      ...defaults.hardware,
      ...source.hardware,
    },
  }
}

function trimOptionalStringFields(value: unknown): unknown {
  if (Array.isArray(value)) {
    return value.map((item) => trimOptionalStringFields(item))
  }

  if (value && typeof value === 'object') {
    const nextEntries = Object.entries(value as Record<string, unknown>).map(([key, child]) => {
      if (typeof child === 'string' && !SECRET_STRING_KEYS.has(key)) {
        return [key, child.trim()]
      }

      return [key, trimOptionalStringFields(child)]
    })

    return Object.fromEntries(nextEntries)
  }

  return value
}

function stripSecretPlaceholders(value: unknown): unknown {
  if (Array.isArray(value)) {
    return value.map((item) => stripSecretPlaceholders(item))
  }

  if (value && typeof value === 'object') {
    const next: Record<string, unknown> = {}

    for (const [key, child] of Object.entries(value as Record<string, unknown>)) {
      if (SECRET_ARRAY_KEYS.has(key) && Array.isArray(child)) {
        const items = child
          .map((item) => (typeof item === 'string' ? item.trim() : item))
          .filter((item) => item !== '' && item !== MASKED_SECRET_VALUE)

        if (items.length === 0) {
          continue
        }

        next[key] = items
        continue
      }

      if (
        SECRET_STRING_KEYS.has(key) &&
        (child === '' || child === null || child === MASKED_SECRET_VALUE)
      ) {
        continue
      }

      next[key] = stripSecretPlaceholders(child)
    }

    return next
  }

  return value
}

export function buildConfigPayload(config: DashboardConfig): Record<string, unknown> {
  const payload: Record<string, unknown> = {
    api_key: normalizeOptionalText(config.api_key),
    default_provider: normalizeOptionalText(config.default_provider),
    default_model: normalizeOptionalText(config.default_model),
    default_temperature: config.default_temperature,
    observability: {
      ...config.observability,
      otel_endpoint: normalizeOptionalText(config.observability.otel_endpoint),
      otel_service_name: normalizeOptionalText(config.observability.otel_service_name),
    },
    autonomy: config.autonomy,
    runtime: {
      ...config.runtime,
      docker: {
        ...config.runtime.docker,
        memory_limit_mb: normalizeOptionalNumber(config.runtime.docker.memory_limit_mb),
        cpu_limit: normalizeOptionalNumber(config.runtime.docker.cpu_limit),
      },
    },
    reliability: {
      ...config.reliability,
      model_fallbacks: Object.fromEntries(
        config.reliability.model_fallbacks
          .filter((entry) => entry.model.trim())
          .map((entry) => [entry.model.trim(), entry.fallbacks.filter(Boolean)]),
      ),
    },
    scheduler: config.scheduler,
    agent: config.agent,
    model_routes: config.model_routes.map((route) => ({
      ...route,
      hint: route.hint.trim(),
      provider: route.provider.trim(),
      model: route.model.trim(),
      api_key: normalizeOptionalText(route.api_key),
    })),
    heartbeat: config.heartbeat,
    channels_config: config.channels_config,
    memory: config.memory,
    tunnel: {
      ...config.tunnel,
      cloudflare: config.tunnel.cloudflare
        ? {
            token: config.tunnel.cloudflare.token,
          }
        : null,
      tailscale: config.tunnel.tailscale
        ? {
            funnel: config.tunnel.tailscale.funnel,
            hostname: normalizeOptionalText(config.tunnel.tailscale.hostname),
          }
        : null,
      ngrok: config.tunnel.ngrok
        ? {
            auth_token: config.tunnel.ngrok.auth_token,
            domain: normalizeOptionalText(config.tunnel.ngrok.domain),
          }
        : null,
      custom: config.tunnel.custom
        ? {
            start_command: config.tunnel.custom.start_command,
            health_url: normalizeOptionalText(config.tunnel.custom.health_url),
            url_pattern: normalizeOptionalText(config.tunnel.custom.url_pattern),
          }
        : null,
    },
    gateway: config.gateway,
    composio: {
      ...config.composio,
      api_key: normalizeOptionalText(config.composio.api_key),
    },
    secrets: config.secrets,
    browser: {
      ...config.browser,
      session_name: normalizeOptionalText(config.browser.session_name),
      native_chrome_path: normalizeOptionalText(config.browser.native_chrome_path),
      computer_use: {
        ...config.browser.computer_use,
        api_key: normalizeOptionalText(config.browser.computer_use.api_key),
        max_coordinate_x: normalizeOptionalNumber(config.browser.computer_use.max_coordinate_x),
        max_coordinate_y: normalizeOptionalNumber(config.browser.computer_use.max_coordinate_y),
      },
    },
    http_request: config.http_request,
    identity: {
      ...config.identity,
      aieos_path: normalizeOptionalText(config.identity.aieos_path),
      aieos_inline: normalizeOptionalText(config.identity.aieos_inline),
    },
    cost: {
      ...config.cost,
      prices: Object.fromEntries(
        config.cost.prices
          .filter((entry) => entry.model.trim())
          .map((entry) => [
            entry.model.trim(),
            {
              input: entry.input,
              output: entry.output,
            },
          ]),
      ),
    },
    reasoning: config.reasoning,
    peripherals: config.peripherals,
    agents: Object.fromEntries(
      config.agents
        .filter((entry) => entry.name.trim())
        .map((entry) => [
          entry.name.trim(),
          {
            provider: entry.provider.trim(),
            model: entry.model.trim(),
            system_prompt: normalizeOptionalText(entry.system_prompt),
            api_key: normalizeOptionalText(entry.api_key),
            temperature: normalizeOptionalNumber(entry.temperature),
            max_depth: entry.max_depth,
          },
        ]),
    ),
    hardware: {
      ...config.hardware,
      serial_port: normalizeOptionalText(config.hardware.serial_port),
      probe_target: normalizeOptionalText(config.hardware.probe_target),
    },
  }

  return stripSecretPlaceholders(trimOptionalStringFields(payload)) as Record<string, unknown>
}
