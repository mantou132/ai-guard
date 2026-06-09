import type { ApiType, LogLevel, RiskLevel, Tab } from './store';

export function inputValue(evt: Event) {
  return (evt.target as HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement).value;
}

export function checkedValue(evt: Event) {
  return (evt.target as HTMLInputElement).checked;
}

export function formatTime(value: string) {
  return new Date(value).toLocaleString();
}

export function titleForTab(tab: Tab) {
  return {
    logs: 'Traffic Logs',
    accounts: 'Upstream Accounts',
    reports: 'Security Reports',
  }[tab];
}

export function subtitleForTab(tab: Tab) {
  return {
    logs: 'Recent OpenAI and Anthropic relay traffic.',
    accounts: 'Route requests by priority, API type, model support, and current health.',
    reports: 'Async review results from the configured OpenRouter audit model.',
  }[tab];
}

export function labelApiType(apiType: ApiType) {
  return apiType === 'open_ai' ? 'OpenAI' : 'Anthropic';
}

export function toneForLevel(level: LogLevel) {
  return level === 'info' ? 'ok' : level === 'warn' ? 'warn' : 'danger';
}

export function toneForRisk(risk: RiskLevel) {
  if (risk === 'critical' || risk === 'high') return 'danger';
  if (risk === 'medium') return 'warn';
  if (risk === 'low') return 'ok';
  return 'info';
}
