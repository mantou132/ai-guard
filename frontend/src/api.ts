import { del, get, post, put } from '@mantou/gem/helper/request';

import type { Account, AuditReport, ConfigView, LogEntry, SaveAccountPayload } from './store';

export const getConfig = () => get<ConfigView>('/api/config');

export const listAccounts = () => get<Account[]>('/api/accounts');

export const saveAccount = (payload: SaveAccountPayload, id?: string) => {
  if (id) return put<Account>(`/api/accounts/${id}`, payload);
  return post<Account>('/api/accounts', payload);
};

export const deleteAccount = (id: string) => del<void>(`/api/accounts/${id}`);

export const listLogs = () => get<LogEntry[]>('/api/logs', { limit: 100 });

export const listReports = () => get<AuditReport[]>('/api/reports', { limit: 100 });
