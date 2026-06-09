import { removeAccount, submitAccount } from '../actions';
import { type Account, type ApiType, appStore, editAccount, resetDraft, setDraft } from '../store';
import { checkedValue, inputValue, labelApiType } from '../utils';

@customElement('account-panel')
@connectStore(appStore)
class AccountPanelElement extends GemElement {
  @template()
  #template = () => html`
    <section
      class="grid grid-cols-[minmax(280px,360px)_minmax(0,1fr)] overflow-hidden rounded-lg border border-line bg-panel max-[900px]:grid-cols-1"
    >
      ${this.#renderForm()}
      ${this.#renderTable()}
    </section>
  `;

  #renderForm = () => html`
    <div class="p-4 border-r border-line grid gap-3 content-start max-[900px]:border-r-0 max-[900px]:border-b">
      <h2 class="m-0 text-base font-semibold">${appStore.editingId ? 'Edit account' : 'New account'}</h2>
      <label class="grid gap-1.5 text-xs font-semibold text-slate-700">
        Name
        <input class=${inputClass} .value=${appStore.draft.name} @input=${(evt: Event) => setDraft({ name: inputValue(evt) })} />
      </label>
      <label class="grid gap-1.5 text-xs font-semibold text-slate-700">
        Base URL
        <input class=${inputClass} .value=${appStore.draft.base_url} @input=${(evt: Event) => setDraft({ base_url: inputValue(evt) })} />
      </label>
      <label class="grid gap-1.5 text-xs font-semibold text-slate-700">
        API Key
        <input
          class=${inputClass}
          type="password"
          placeholder=${appStore.editingId ? 'Leave empty to keep current key' : ''}
          .value=${appStore.draft.api_key}
          @input=${(evt: Event) => setDraft({ api_key: inputValue(evt) })}
        />
      </label>
      <label class="grid gap-1.5 text-xs font-semibold text-slate-700">
        API Type
        <select class=${inputClass} .value=${appStore.draft.api_type} @change=${(evt: Event) => setDraft({ api_type: inputValue(evt) as ApiType })}>
          <option value="open_ai">OpenAI compatible</option>
          <option value="anthropic">Anthropic compatible</option>
        </select>
      </label>
      <label class="grid gap-1.5 text-xs font-semibold text-slate-700">
        Supported Models
        <textarea
          class="${inputClass} min-h-20 resize-y"
          placeholder="model-a, model-b"
          .value=${appStore.draft.supported_models}
          @input=${(evt: Event) => setDraft({ supported_models: inputValue(evt) })}
        ></textarea>
      </label>
      <label class="grid gap-1.5 text-xs font-semibold text-slate-700">
        Priority
        <input class=${inputClass} type="number" .value=${String(appStore.draft.priority)} @input=${(evt: Event) => setDraft({ priority: Number(inputValue(evt)) })} />
      </label>
      <label class="flex items-center gap-2 text-xs font-semibold text-slate-700">
        <input class="size-4" type="checkbox" .checked=${appStore.draft.enabled} @change=${(evt: Event) => setDraft({ enabled: checkedValue(evt) })} />
        Enabled
      </label>
      <div class="flex items-center gap-2">
        <dy-button @click=${submitAccount} ?disabled=${appStore.loading}>Save</dy-button>
        <dy-button @click=${resetDraft}>Reset</dy-button>
      </div>
    </div>
  `;

  #renderTable = () => html`
    <div class="overflow-auto">
      <table class="w-full min-w-[780px] border-collapse text-sm">
        <thead>
          <tr class="bg-slate-50 text-slate-500">
            <th class=${thClass}>Name</th>
            <th class=${thClass}>Type</th>
            <th class=${thClass}>Priority</th>
            <th class=${thClass}>Status</th>
            <th class=${thClass}>Models</th>
            <th class=${thClass}>Key</th>
            <th class=${thClass}></th>
          </tr>
        </thead>
        <tbody>
          ${appStore.accounts.map((account) => this.#renderAccount(account))}
        </tbody>
      </table>
    </div>
  `;

  #renderAccount = (account: Account) => html`
    <tr>
      <td class=${tdClass}>
        <strong>${account.name}</strong>
        <div class="text-slate-500">${account.base_url}</div>
      </td>
      <td class=${tdClass}>${labelApiType(account.api_type)}</td>
      <td class=${tdClass}>${account.priority}</td>
      <td class=${tdClass}>
        <span class=${badgeClass(account.available ? 'ok' : 'warn')}>${account.available ? 'available' : 'paused'}</span>
        ${account.status.last_error ? html`<div class="text-red-700">${account.status.last_error}</div>` : null}
      </td>
      <td class=${tdClass}>${account.supported_models.length ? account.supported_models.join(', ') : 'any'}</td>
      <td class=${tdClass}>${account.api_key_preview || '-'}</td>
      <td class=${tdClass}>
        <div class="flex items-center gap-2">
          <dy-button @click=${() => editAccount(account)}>Edit</dy-button>
          <dy-button @click=${() => removeAccount(account)}>Delete</dy-button>
        </div>
      </td>
    </tr>
  `;
}

const inputClass = 'w-full min-h-9 rounded-md border border-slate-300 bg-white px-2.5 py-2 text-sm text-ink';
const thClass = 'p-3 text-left align-top border-b border-slate-100 font-semibold';
const tdClass = 'p-3 text-left align-top border-b border-slate-100';

function badgeClass(tone: 'ok' | 'warn') {
  return `inline-flex min-h-5 items-center rounded-full px-2 text-xs font-semibold ${
    tone === 'ok' ? 'bg-green-100 text-green-800' : 'bg-orange-100 text-orange-800'
  }`;
}
