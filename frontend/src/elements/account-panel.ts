import type { ContextMenuItem } from 'duoyun-ui/elements/contextmenu';
import { ContextMenu } from 'duoyun-ui/elements/contextmenu';
import { theme } from 'duoyun-ui/lib/theme';
import { Time } from 'duoyun-ui/lib/time';
import { polling } from 'duoyun-ui/lib/timer';
import type { FormItem } from 'duoyun-ui/patterns/form';
import { createForm } from 'duoyun-ui/patterns/form';
import type { PatTableColumn } from 'duoyun-ui/patterns/table';
import { deleteAccount, listAccounts, saveAccount } from '../api';
import { apiTypeList, apiTypeMap } from '../enums';
import { type Account, ApiType, appStore, type Draft } from '../store';

const initialDraft = (): Draft => ({
  name: '',
  base_url: 'https://openrouter.ai/api',
  api_key: '',
  api_type: ApiType.OpenAI,
  supported_models: '',
  priority: 0,
  enabled: true,
});

const formItems = (isNew?: boolean): FormItem<Draft>[] => [
  [
    {
      type: 'text',
      field: 'name',
      label: 'Name',
      required: true,
      autofocus: true,
      style: { flexGrow: 1 },
    },
    {
      type: 'select',
      field: 'api_type',
      label: 'API Type',
      required: true,
      options: apiTypeList.map(({ value, label }) => ({ label: `${label} compatible`, value })),
    },
  ],
  {
    type: 'text',
    field: 'base_url',
    label: 'Base URL',
    required: true,
  },
  {
    type: 'password',
    field: 'api_key',
    label: 'API Key',
    required: isNew,
    placeholder: isNew ? '' : 'Leave empty when editing to keep current key',
  },
  [
    {
      type: 'number',
      field: 'priority',
      label: 'Priority',
      style: { flexGrow: 1 },
    },
    {
      type: 'select',
      field: 'enabled',
      label: 'Status',
      options: [
        { label: 'Enabled', value: true },
        { label: 'Disabled', value: false },
      ],
    },
  ],
  {
    type: 'textarea',
    rows: 3,
    field: 'supported_models',
    label: 'Supported Models',
    placeholder: 'model-a, model-b',
  },
];

@customElement('ai-guard-account-panel')
@connectStore(appStore)
class AccountPanelElement extends GemElement {
  @mounted()
  #mounted = () => {
    return polling(async () => {
      appStore({ accounts: await listAccounts() });
    }, 10_000);
  };

  #columns: PatTableColumn<Account>[] = [
    {
      title: 'Name',
      dataIndex: 'name',
      width: '16em',
      ellipsis: true,
      render: (account) => html`
        <strong>${account.name}</strong>
        ${muted(account.base_url)}
      `,
      filterOptions: {
        getSearchText: (account) => `${account.name} ${account.base_url}`,
      },
    },
    {
      title: 'Type',
      width: '9em',
      render: (account) => apiTypeMap[account.api_type],
      filterOptions: {
        field: 'api_type',
        type: 'enum',
        getOptions: () => apiTypeList,
      },
    },
    {
      title: 'Priority',
      dataIndex: 'priority',
      width: '7em',
      sortable: true,
      filterOptions: {
        type: 'number',
      },
    },
    {
      title: 'Status',
      width: '11em',
      render: (account) => html`
        <dy-tag small color=${account.available ? 'positive' : 'notice'}>
          ${account.available ? 'available' : 'paused'}
        </dy-tag>
        ${muted(account.enabled ? 'enabled' : 'disabled')}
      `,
      filterOptions: {
        field: 'enabled',
        type: 'enum',
        getOptions: () => [
          { label: 'Enabled', value: true },
          { label: 'Disabled', value: false },
        ],
      },
    },
    {
      title: 'Models',
      dataIndex: 'supported_models',
      width: '16em',
      visibleWidth: 'auto',
      ellipsis: true,
      render: (account) => (account.supported_models.length ? account.supported_models.join(', ') : 'any'),
      filterOptions: {
        getSearchText: (account) => account.supported_models.join(' '),
      },
    },
    {
      title: 'Key',
      dataIndex: 'api_key_preview',
      width: '10em',
      ellipsis: true,
      filterOptions: false,
    },
    {
      title: 'Last Success',
      width: '11em',
      visibleWidth: 'auto',
      render: (account) => (account.status.last_success_at ? new Time(account.status.last_success_at).format() : '-'),
      filterOptions: false,
    },
  ];

  #getActions = (account: Account, activeElement: HTMLElement): ContextMenuItem[] => [
    {
      text: 'Edit',
      handle: () => this.#openForm(account),
    },
    {
      text: 'Delete',
      danger: true,
      handle: async () => {
        await ContextMenu.confirm(`Delete account "${account.name}"?`, { activeElement, danger: true });
        await deleteAccount(account.id);
        appStore({ accounts: await listAccounts() });
      },
    },
  ];

  #openForm = (account?: Account) => {
    createForm<Draft>({
      type: 'modal',
      data: account
        ? { ...account, api_key: '', supported_models: account.supported_models.join(', ') }
        : initialDraft(),
      header: account ? `Edit ${account.name}` : 'Create Account',
      query: account ? ['account', account.id] : ['account', 'new'],
      formItems: formItems(!account),
      prepareOk: async (draft) => {
        await saveAccount(
          {
            ...draft,
            supported_models: draft.supported_models
              .split(',')
              .map((item) => item.trim())
              .filter(Boolean),
          },
          account?.id,
        );
        appStore({ accounts: await listAccounts() });
      },
    });
  };

  @template()
  #template = () => html`
    <ai-guard-header .subTitle=${`Route requests by priority, API type, model support, and current health.`}></ai-guard-header>
    <dy-pat-table
      filterable
      .rowKey=${'id'}
      .data=${appStore.accounts}
      .columns=${this.#columns}
      .getActions=${this.#getActions}
    >
      <dy-button @click=${() => this.#openForm()}>Add</dy-button>
    </dy-pat-table>
  `;
}

function muted(text: string) {
  return html`<div style=${styleMap({ color: theme.describeColor, marginTop: '0.2em', overflowWrap: 'anywhere' })}>${text}</div>`;
}
