import { deleteAccount, getConfig, listAccounts, listLogs, listReports, saveAccount } from './api';
import { type Account, appStore, draftToPayload, resetDraft, setLoading } from './store';

export async function loadDashboard() {
  setLoading(true);
  try {
    const [config, accounts, logs, reports] = await Promise.all([
      getConfig(),
      listAccounts(),
      listLogs(),
      listReports(),
    ]);
    appStore({ config, accounts, logs, reports });
  } finally {
    setLoading(false);
  }
}

export async function submitAccount() {
  setLoading(true);
  try {
    await saveAccount(draftToPayload(), appStore.editingId || undefined);
    resetDraft();
    await loadDashboard();
  } finally {
    setLoading(false);
  }
}

export async function removeAccount(account: Account) {
  if (!globalThis.confirm(`Delete account "${account.name}"?`)) return;
  setLoading(true);
  try {
    await deleteAccount(account.id);
    await loadDashboard();
  } finally {
    setLoading(false);
  }
}
