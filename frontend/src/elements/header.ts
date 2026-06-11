@customElement('ai-guard-header')
class AppHeaderElement extends GemElement {
  @attribute subTitle: string;

  render = () => html`
    <section class="min-h-16 flex items-center justify-between gap-4 mb-5">
      <div>
        <h1 class="m-0 text-xl font-semibold"><dy-title inert></dy-title></h1>
        <p class="mt-1 mb-0 text-sm text-slate-500">${this.subTitle}</p>
      </div>
    </section>
  `;
}
