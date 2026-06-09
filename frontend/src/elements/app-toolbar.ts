import { loadDashboard } from '../actions';
import { appStore, type Tab } from '../store';
import { subtitleForTab, titleForTab } from '../utils';

@customElement('app-toolbar')
@connectStore(appStore)
class AppToolbarElement extends GemElement {
  @template()
  #template = () => html`
    <section
      class="min-h-16 rounded-lg border border-line bg-panel px-4 py-3 flex items-center justify-between gap-4"
    >
      <div>
        <h1 class="m-0 text-xl font-semibold">${titleForTab(appStore.tab as Tab)}</h1>
        <p class="mt-1 mb-0 text-sm text-slate-500">${subtitleForTab(appStore.tab as Tab)}</p>
      </div>
      <div class="flex items-center gap-2">
        <dy-button @click=${loadDashboard} ?disabled=${appStore.loading}>Refresh</dy-button>
      </div>
    </section>
  `;
}
