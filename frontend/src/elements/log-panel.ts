import { appStore, type LogEntry } from '../store';
import { formatTime, labelApiType, toneForLevel, toneForRisk } from '../utils';

@customElement('log-panel')
@connectStore(appStore)
class LogPanelElement extends GemElement {
  #state = createState({ expandedLogId: '' });

  @template()
  #template = () => html`
    <section class="rounded-lg border border-line bg-panel p-4 overflow-auto">
      <table class="w-full min-w-[780px] border-collapse text-sm">
        <thead>
          <tr class="bg-slate-50 text-slate-500">
            <th class=${thClass}>Time</th>
            <th class=${thClass}>Route</th>
            <th class=${thClass}>Account</th>
            <th class=${thClass}>Status</th>
            <th class=${thClass}>Risk</th>
            <th class=${thClass}>Latency</th>
          </tr>
        </thead>
        <tbody>
          ${appStore.logs.map((log) => this.#renderLog(log))}
        </tbody>
      </table>
    </section>
  `;

  #renderLog = (log: LogEntry) => html`
    <tr class="cursor-pointer" @click=${() => this.#toggleLog(log.id)}>
      <td class=${tdClass}>${formatTime(log.created_at)}</td>
      <td class=${tdClass}>
        <strong>${log.method} ${log.path}</strong>
        <div class="text-slate-500">${labelApiType(log.api_type)} / ${log.model || 'no model'}</div>
      </td>
      <td class=${tdClass}>${log.account_name || '-'}</td>
      <td class=${tdClass}>
        <span class=${badgeClass(toneForLevel(log.level))}>${log.status_code || '-'}</span>
        ${log.error ? html`<div class="text-red-700">${log.error}</div>` : null}
      </td>
      <td class=${tdClass}>
        <span class=${badgeClass(toneForRisk(log.risk_level))}>${log.risk_level}</span>
        <div class="text-slate-500">${log.audit_status}</div>
      </td>
      <td class=${tdClass}>${log.latency_ms ?? '-'} ms</td>
    </tr>
    ${
      this.#state.expandedLogId === log.id
        ? html`
          <tr>
            <td class=${tdClass} colspan="6">
              <strong>Extracted Request</strong>
              <pre class=${preClass}>${formatPayload(log.request_payload)}</pre>
              <strong>Extracted Response</strong>
              <pre class=${preClass}>${formatPayload(log.response_payload)}</pre>
            </td>
          </tr>
        `
        : null
    }
  `;

  #toggleLog = (id: string) => {
    this.#state({ expandedLogId: this.#state.expandedLogId === id ? '' : id });
  };
}

const thClass = 'p-3 text-left align-top border-b border-slate-100 font-semibold';
const tdClass = 'p-3 text-left align-top border-b border-slate-100';
const preClass =
  'my-2 max-h-56 overflow-auto whitespace-pre-wrap break-words rounded-md bg-slate-50 p-3 font-mono text-sm text-slate-800';

function badgeClass(tone: string) {
  return `inline-flex min-h-5 items-center rounded-full px-2 text-xs font-semibold ${toneClass(tone)}`;
}

function toneClass(tone: string) {
  if (tone === 'danger') return 'bg-red-100 text-red-800';
  if (tone === 'warn') return 'bg-orange-100 text-orange-800';
  if (tone === 'ok') return 'bg-green-100 text-green-800';
  return 'bg-blue-100 text-blue-800';
}

function formatPayload(payload: unknown) {
  if (payload === undefined || payload === null) return '-';
  if (typeof payload === 'string') return payload || '-';
  return JSON.stringify(payload, null, 2);
}
