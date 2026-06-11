import type { ContextMenuItem } from 'duoyun-ui/elements/contextmenu';
import { Modal } from 'duoyun-ui/elements/modal';
import { theme } from 'duoyun-ui/lib/theme';
import { Time } from 'duoyun-ui/lib/time';
import { polling } from 'duoyun-ui/lib/timer';
import type { PatTableColumn } from 'duoyun-ui/patterns/table';
import { listReports } from '../api';
import { auditStatusList, riskLevelList } from '../enums';
import { type AuditReport, appStore } from '../store';
import { toneForRisk } from '../utils';

@customElement('ai-guard-report-panel')
@connectStore(appStore)
class ReportPanelElement extends GemElement {
  @mounted()
  #mounted = () => {
    return polling(async () => {
      appStore({ reports: await listReports() });
    }, 10_000);
  };

  #columns: PatTableColumn<AuditReport>[] = [
    {
      title: 'Time',
      width: '12em',
      render: (report) => new Time(report.created_at).format(),
      sortable: true,
      filterOptions: {
        field: 'created_at',
        type: 'date-time',
        getCompareValue: (report) => new Date(report.created_at).getTime(),
      },
    },
    {
      title: 'Risk',
      width: '9em',
      render: (report) => html`
        <dy-tag small color=${tagColor(toneForRisk(report.risk_level))}>${report.risk_level}</dy-tag>
      `,
      filterOptions: {
        field: 'risk_level',
        type: 'enum',
        getOptions: () => riskLevelList,
      },
    },
    {
      title: 'Status',
      dataIndex: 'status',
      width: '9em',
      filterOptions: {
        type: 'enum',
        getOptions: () => auditStatusList,
      },
    },
    {
      title: 'Title',
      dataIndex: 'title',
      width: '24em',
      ellipsis: true,
      render: (report) => html`
        <strong>${report.title}</strong>
        ${report.error ? muted(report.error, theme.negativeColor) : null}
      `,
      filterOptions: {
        getSearchText: (report) => `${report.title} ${report.error || ''}`,
      },
    },
    {
      title: 'Findings',
      dataIndex: 'findings',
      width: '10em',
      visibleWidth: 'auto',
      render: (report) => String(report.findings.length),
      filterOptions: {
        getSearchText: (report) => report.findings.join(' '),
      },
    },
  ];

  #getActions = (report: AuditReport): ContextMenuItem[] => [
    {
      text: 'View report',
      handle: () => this.#openReport(report),
    },
  ];

  #openReport = (report: AuditReport) => {
    Modal.open({
      header: report.title || 'Security Report',
      body: html`<ai-guard-report-detail .report=${report}></ai-guard-report-detail>`,
      disableDefaultOKBtn: true,
    }).catch(() => undefined);
  };

  @template()
  #template = () => html`
    <ai-guard-header .subTitle=${`Async review results from the configured OpenRouter audit model.`}></ai-guard-header>
    <dy-pat-table
      filterable
      .rowKey=${'id'}
      .data=${appStore.reports}
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

function muted(text: string, color = theme.describeColor) {
  return html`<div style=${styleMap({ color, marginTop: '0.2em', overflowWrap: 'anywhere' })}>${text}</div>`;
}
