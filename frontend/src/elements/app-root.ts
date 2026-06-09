import './account-panel';
import './app-toolbar';
import './log-panel';
import './report-panel';
import { loadDashboard } from '../actions';
import { appStore, setTab, type Tab } from '../store';

@customElement('app-root')
@connectStore(appStore)
class AppRootElement extends GemElement {
  @mounted()
  #mounted = () => {
    void loadDashboard();
  };

  @template()
  #template = () => html`
    <div class="min-h-screen grid grid-cols-[260px_minmax(0,1fr)] max-[900px]:grid-cols-1">
      <aside class="bg-sidebar text-slate-100 p-6 max-[900px]:p-4">
        <h1 class="mt-0 mb-6 text-[22px] font-bold">ai-guard</h1>
        <nav class="grid gap-2 max-[900px]:grid-cols-3">
          ${(['logs', 'accounts', 'reports'] as Tab[]).map(
            (tab) => html`
              <button
                class="h-10 rounded-md border-0 px-3 text-left cursor-pointer text-slate-200 bg-transparent data-[active=true]:bg-accent data-[active=true]:text-white"
                data-active=${appStore.tab === tab}
                @click=${() => setTab(tab)}
              >
                ${tabLabel(tab)}
              </button>
            `,
          )}
        </nav>
        <div class="mt-7 pt-5 border-t border-slate-600 grid gap-2 text-sm text-slate-300">
          <span>Audit: ${appStore.config?.audit_enabled ? 'enabled' : 'not configured'}</span>
          <span>Model: ${appStore.config?.audit_model || '-'}</span>
          <span>Bind: ${appStore.config?.bind || '-'}</span>
        </div>
      </aside>
      <main class="p-6 grid gap-5 content-start max-[900px]:p-4">
        <app-toolbar></app-toolbar>
        ${appStore.tab === 'logs' ? html`<log-panel></log-panel>` : null}
        ${appStore.tab === 'accounts' ? html`<account-panel></account-panel>` : null}
        ${appStore.tab === 'reports' ? html`<report-panel></report-panel>` : null}
      </main>
    </div>
  `;
}

function tabLabel(tab: Tab) {
  return {
    logs: 'Logs',
    accounts: 'Accounts',
    reports: 'Reports',
  }[tab];
}
