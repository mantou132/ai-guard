import { theme } from 'duoyun-ui/lib/theme';
import { Time } from 'duoyun-ui/lib/time';
import { apiTypeMap } from '../enums';
import type { LogEntry } from '../store';

const style = css`
  .detail-grid {
    display: grid;
    gap: 1em;
    min-width: min(48em, 60vw);
  }
  .meta-grid {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
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
  pre {
    max-height: 18em;
    overflow: auto;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
    border: 1px solid ${theme.borderColor};
    border-radius: ${theme.normalRound};
    background: ${theme.lightBackgroundColor};
    padding: 0.8em;
    margin: 0.4em 0 0;
    font: 0.875em ui-monospace, SFMono-Regular, Consolas, monospace;
  }
`;

@customElement('ai-guard-log-detail')
@adoptedStyle(style)
class LogDetailElement extends GemElement {
  @property log: LogEntry;

  render = () => html`
    <div class="detail-grid">
      <div class="meta-grid">
        ${field('Time', new Time(this.log.created_at).format())}
        ${field('API Type', apiTypeMap[this.log.api_type])}
        ${field('Account', this.log.account_name || '-')}
        ${field('Model', this.log.model || '-')}
        ${field('Status', String(this.log.status_code || '-'))}
        ${field('Latency', `${this.log.latency_ms ?? '-'} ms`)}
        ${field('Risk', this.log.risk_level)}
        ${field('Audit', this.log.audit_status)}
      </div>
      ${this.log.error ? field('Error', this.log.error) : null}
      <section>
        <strong>Extracted Request</strong>
        <pre>${formatPayload(this.log.request_payload)}</pre>
      </section>
      <section>
        <strong>Extracted Response</strong>
        <pre>${formatPayload(this.log.response_payload)}</pre>
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

function formatPayload(payload: unknown) {
  if (payload === undefined || payload === null) return '-';
  if (typeof payload === 'string') return payload || '-';
  return JSON.stringify(payload, null, 2);
}
