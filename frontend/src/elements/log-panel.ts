import type { ContextMenuItem } from 'duoyun-ui/elements/contextmenu';
import { Modal } from 'duoyun-ui/elements/modal';
import { theme } from 'duoyun-ui/lib/theme';
import { Time } from 'duoyun-ui/lib/time';
import { polling } from 'duoyun-ui/lib/timer';
import type { PatTableColumn } from 'duoyun-ui/patterns/table';
import { listLogs } from '../api';
import { apiTypeMap, logLevelList, riskLevelList } from '../enums';
import { appStore, type LogEntry, LogLevel } from '../store';
import { toneForRisk } from '../utils';

@customElement('ai-guard-log-panel')
@connectStore(appStore)
class LogPanelElement extends GemElement {
  @mounted()
  #mounted = () => {
    return polling(async () => {
      appStore({ logs: await listLogs() });
    }, 10_000);
  };

  #columns: PatTableColumn<LogEntry>[] = [
    {
      title: 'Time',
      width: '12em',
      render: (log) => new Time(log.created_at).format(),
      sortable: true,
      filterOptions: {
        field: 'created_at',
        type: 'date-time',
        getCompareValue: (log) => new Date(log.created_at).getTime(),
      },
    },
    {
      title: 'Route',
      dataIndex: 'path',
      width: '18em',
      ellipsis: true,
      render: (log) => html`
        <strong>${log.method} ${log.path}</strong>
        ${muted(`${apiTypeMap[log.api_type]} / ${log.model || 'no model'}`)}
      `,
      filterOptions: {
        getSearchText: (log) => `${log.method} ${log.path} ${log.model || ''}`,
      },
    },
    {
      title: 'Account',
      width: '12em',
      ellipsis: true,
      dataIndex: 'account_name',
      filterOptions: {},
    },
    {
      title: 'Status',
      width: '9em',
      render: ({ level, status_code }) => html`
        <dy-tag small color=${level === LogLevel.Info ? 'ok' : level === LogLevel.Warn ? 'warn' : 'danger'}>${status_code || '-'}</dy-tag>
        ${muted(level)}
      `,
      filterOptions: {
        field: 'level',
        type: 'enum',
        getOptions: () => logLevelList,
      },
    },
    {
      title: 'Risk',
      width: '10em',
      render: (log) => html`
        <dy-tag small color=${tagColor(toneForRisk(log.risk_level))}>${log.risk_level}</dy-tag>
        ${muted(log.audit_status)}
      `,
      filterOptions: {
        field: 'risk_level',
        type: 'enum',
        getOptions: () => riskLevelList,
      },
    },
    {
      title: 'Latency',
      width: '8em',
      render: (log) => `${log.latency_ms ?? '-'} ms`,
      sortable: true,
      filterOptions: {
        field: 'latency_ms',
        type: 'number',
      },
    },
  ];

  #getActions = (log: LogEntry): ContextMenuItem[] => [
    {
      text: 'View details',
      handle: () => this.#openDetails(log),
    },
  ];

  #openDetails = (log: LogEntry) => {
    Modal.open({
      header: `${log.method} ${log.path}`,
      body: html`<ai-guard-log-detail .log=${log}></ai-guard-log-detail>`,
      disableDefaultOKBtn: true,
    }).catch(() => undefined);
  };

  @template()
  #template = () => html`
    <ai-guard-header .subTitle=${`Recent OpenAI and Anthropic relay traffic.`}></ai-guard-header>
    <dy-pat-table
      filterable
      .rowKey=${'id'}
      .data=${appStore.logs}
      .columns=${this.#columns}
      .getActions=${this.#getActions}
    ></dy-pat-table>
  `;
}

function tagColor(tone: string) {
  if (tone === 'danger') return 'negative';
  if (tone === 'warn') return 'notice';
  if (tone === 'ok') return 'positive';
  return 'informative';
}

function muted(text: string) {
  return html`<div style=${styleMap({ color: theme.describeColor, marginTop: '0.2em', overflowWrap: 'anywhere' })}>${text}</div>`;
}
