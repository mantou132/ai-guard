import { type AuditReport, appStore } from '../store';
import { formatTime, toneForRisk } from '../utils';

@customElement('report-panel')
@connectStore(appStore)
class ReportPanelElement extends GemElement {
  @template()
  #template = () => html`
    <section class="rounded-lg border border-line bg-panel p-4 overflow-auto">
      <table class="w-full min-w-[780px] border-collapse text-sm">
        <thead>
          <tr class="bg-slate-50 text-slate-500">
            <th class=${thClass}>Time</th>
            <th class=${thClass}>Risk</th>
            <th class=${thClass}>Status</th>
            <th class=${thClass}>Title</th>
            <th class=${thClass}>Findings</th>
          </tr>
        </thead>
        <tbody>
          ${appStore.reports.map((report) => this.#renderReport(report))}
        </tbody>
      </table>
    </section>
  `;

  #renderReport = (report: AuditReport) => html`
    <tr>
      <td class=${tdClass}>${formatTime(report.created_at)}</td>
      <td class=${tdClass}><span class=${badgeClass(toneForRisk(report.risk_level))}>${report.risk_level}</span></td>
      <td class=${tdClass}>${report.status}</td>
      <td class=${tdClass}>
        <strong>${report.title}</strong>
        ${report.error ? html`<div class="text-red-700">${report.error}</div>` : null}
      </td>
      <td class=${tdClass}>${report.findings.length ? report.findings.join('; ') : '-'}</td>
    </tr>
  `;
}

const thClass = 'p-3 text-left align-top border-b border-slate-100 font-semibold';
const tdClass = 'p-3 text-left align-top border-b border-slate-100';

function badgeClass(tone: string) {
  return `inline-flex min-h-5 items-center rounded-full px-2 text-xs font-semibold ${toneClass(tone)}`;
}

function toneClass(tone: string) {
  if (tone === 'danger') return 'bg-red-100 text-red-800';
  if (tone === 'warn') return 'bg-orange-100 text-orange-800';
  if (tone === 'ok') return 'bg-green-100 text-green-800';
  return 'bg-blue-100 text-blue-800';
}
