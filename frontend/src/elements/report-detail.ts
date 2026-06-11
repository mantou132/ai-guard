import { adoptedStyle } from '@mantou/gem/lib/decorators';
import { css } from '@mantou/gem/lib/reactive';
import { theme } from 'duoyun-ui/lib/theme';
import { Time } from 'duoyun-ui/lib/time';
import type { AuditReport } from '../store';

const style = css`
  .detail-grid {
    display: grid;
    gap: 1em;
    min-width: min(48em, 60vw);
  }
  .meta-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 0.7em 1em;
    font-size: 0.875em;
  }
  .field {
    min-width: 0;
  }
  .field-label {
    color: ${theme.describeColor};
    margin-block-end: 0.2em;
  }
  .field-value {
    overflow-wrap: anywhere;
  }
  ul {
    margin: 0.5em 0 0;
    padding-inline-start: 1.3em;
  }
  li + li {
    margin-block-start: 0.45em;
  }
`;

@customElement('ai-guard-report-detail')
@adoptedStyle(style)
class ReportDetailElement extends GemElement {
  @property report: AuditReport;

  render = () => html`
    <div class="detail-grid">
      <div class="meta-grid">
        ${field('Time', new Time(this.report.created_at).format())}
        ${field('Status', this.report.status)}
        ${field('Risk', this.report.risk_level)}
        ${field('Log ID', this.report.log_id)}
      </div>
      ${this.report.error ? field('Error', this.report.error) : null}
      <section>
        <strong>Findings</strong>
        ${
          this.report.findings.length
            ? html`<ul>${this.report.findings.map((finding) => html`<li>${finding}</li>`)}</ul>`
            : html`<p>-</p>`
        }
      </section>
    </div>
  `;
}

function field(label: string, value: unknown) {
  return html`
    <div class="field">
      <div class="field-label">${label}</div>
      <div class="field-value">${String(value)}</div>
    </div>
  `;
}
