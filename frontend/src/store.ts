export enum ApiType {
  OpenAI = 'open_ai',
  Anthropic = 'anthropic',
}

export enum LogLevel {
  Info = 'info',
  Warn = 'warn',
  Error = 'error',
}

export enum RiskLevel {
  Unknown = 'unknown',
  Low = 'low',
  Medium = 'medium',
  High = 'high',
  Critical = 'critical',
}

export enum AuditStatus {
  Queued = 'queued',
  Completed = 'completed',
  Failed = 'failed',
  Skipped = 'skipped',
}

export type JsonValue = null | boolean | number | string | JsonValue[] | { [key: string]: JsonValue };

export type Account = {
  id: string;
  name: string;
  base_url: string;
  api_key_preview: string;
  has_api_key: boolean;
  api_type: ApiType;
  supported_models: string[];
  priority: number;
  enabled: boolean;
  available: boolean;
  status: {
    disabled_until?: string;
    failures: number;
    last_error?: string;
    last_success_at?: string;
    last_failure_at?: string;
  };
};

export type LogEntry = {
  id: string;
  created_at: string;
  level: LogLevel;
  api_type: ApiType;
  method: string;
  path: string;
  model?: string;
  account_name?: string;
  status_code?: number;
  latency_ms?: number;
  request_payload: JsonValue;
  response_payload: JsonValue;
  risk_level: RiskLevel;
  audit_status: AuditStatus;
  error?: string;
};

export type AuditReport = {
  id: string;
  log_id: string;
  created_at: string;
  status: AuditStatus;
  risk_level: RiskLevel;
  title: string;
  findings: string[];
  error?: string;
};

export type ConfigView = {
  bind: string;
  data_dir: string;
  audit_enabled: boolean;
  audit_model: string;
};

export type Draft = {
  name: string;
  base_url: string;
  api_key: string;
  api_type: ApiType;
  supported_models: string;
  priority: number;
  enabled: boolean;
};

export type SaveAccountPayload = Omit<Draft, 'supported_models'> & {
  supported_models: string[];
};

export const appStore = createStore({
  accounts: [] as Account[],
  logs: [] as LogEntry[],
  reports: [] as AuditReport[],
  config: undefined as ConfigView | undefined,
});
