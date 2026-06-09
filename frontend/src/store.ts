export type ApiType = 'open_ai' | 'anthropic';
export type LogLevel = 'info' | 'warn' | 'error';
export type RiskLevel = 'unknown' | 'low' | 'medium' | 'high' | 'critical';
export type AuditStatus = 'queued' | 'completed' | 'failed' | 'skipped';
export type Tab = 'logs' | 'accounts' | 'reports';
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

export type SaveAccountPayload = {
  name: string;
  base_url: string;
  api_key: string;
  api_type: ApiType;
  supported_models: string[];
  priority: number;
  enabled: boolean;
};

export const emptyDraft = (): Draft => ({
  name: '',
  base_url: 'https://openrouter.ai/api',
  api_key: '',
  api_type: 'open_ai',
  supported_models: '',
  priority: 0,
  enabled: true,
});

export const appStore = createStore({
  tab: 'logs' as Tab,
  accounts: [] as Account[],
  logs: [] as LogEntry[],
  reports: [] as AuditReport[],
  config: undefined as ConfigView | undefined,
  draft: emptyDraft(),
  editingId: '',
  loading: false,
});

export function setTab(tab: Tab) {
  appStore({ tab });
}

export function setLoading(loading: boolean) {
  appStore({ loading });
}

export function setDraft(patch: Partial<Draft>) {
  appStore({ draft: { ...appStore.draft, ...patch } });
}

export function resetDraft() {
  appStore({ draft: emptyDraft(), editingId: '' });
}

export function editAccount(account: Account) {
  appStore({
    tab: 'accounts',
    editingId: account.id,
    draft: {
      name: account.name,
      base_url: account.base_url,
      api_key: '',
      api_type: account.api_type,
      supported_models: account.supported_models.join(', '),
      priority: account.priority,
      enabled: account.enabled,
    },
  });
}

export function draftToPayload(): SaveAccountPayload {
  const draft = appStore.draft;
  return {
    name: draft.name,
    base_url: draft.base_url,
    api_key: draft.api_key,
    api_type: draft.api_type,
    supported_models: draft.supported_models
      .split(',')
      .map((item) => item.trim())
      .filter(Boolean),
    priority: Number(draft.priority) || 0,
    enabled: draft.enabled,
  };
}
