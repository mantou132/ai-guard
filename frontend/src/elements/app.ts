import { polling } from 'duoyun-ui/lib/timer';
import type { NavItems, Routes, UserInfo } from 'duoyun-ui/patterns/console';
import { getConfig } from '../api';
import { appStore, RiskLevel } from '../store';

const routes = {
  logs: {
    pattern: '/',
    title: 'Traffic Logs',
    getContent: () => html`<ai-guard-log-panel></ai-guard-log-panel>`,
  },
  accounts: {
    pattern: '/accounts',
    title: 'Upstream Accounts',
    getContent: () => html`<ai-guard-account-panel></ai-guard-account-panel>`,
  },
  reports: {
    pattern: '/reports',
    title: 'Security Reports',
    getContent: () => html`<ai-guard-report-panel></ai-guard-report-panel>`,
  },
  fallback: {
    pattern: '*',
    redirect: '/',
  },
} satisfies Routes;

@customElement('ai-guard-app')
@connectStore(appStore)
class AppRootElement extends GemElement {
  get #navItems(): NavItems {
    const riskReportCount = appStore.reports.filter((report) => isRiskyRiskLevel(report.risk_level)).length;
    return [
      routes.logs,
      {
        title: 'Manage',
        group: [
          routes.accounts,
          {
            ...routes.reports,
            title: 'Security Reports',
            slot: riskReportCount ? html`<dy-badge small count=${String(riskReportCount)}></dy-badge>` : undefined,
          },
        ],
      },
    ];
  }

  get #userInfo(): UserInfo {
    return {
      username: 'ai-guard',
      org: appStore.config?.audit_enabled ? 'audit enabled' : 'audit not configured',
      profile: '/',
    };
  }

  @mounted()
  #mounted = () => {
    return polling(async () => {
      appStore({ config: await getConfig() });
    }, 10_000);
  };

  @template()
  #template = () => html`
    <dy-pat-console
      name="AI Guard"
      .logo=${'/icons/icon-fg.png'}
      .routes=${routes}
      .navItems=${this.#navItems}
      .userInfo=${this.#userInfo}
      .keyboardAccess=${true}
      .responsive=${true}
    ></dy-pat-console>
  `;
}

function isRiskyRiskLevel(risk: RiskLevel) {
  return risk === RiskLevel.Medium || risk === RiskLevel.High || risk === RiskLevel.Critical;
}
