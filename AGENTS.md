# Agent Rules

1. 遇到没有明确的事情不要自由发挥，应该询问确定
2. 始终根据目的考虑代码，如果有更简洁的方案应该提出来
3. 项目结构或入口有变化时，要同步更新本文件，避免后续 Agent 重新摸索

# 项目结构

## 运行链路

- 开发时启动两个服务：
  - Rust 后端：`cargo watch -x run`，默认监听 `127.0.0.1:8787`
  - 前端：`pnpm --dir frontend run dev`，Rspack dev server 默认监听 `127.0.0.1:5173`，并代理 `/api`、`/v1`、`/openai`、`/anthropic` 到 Rust 后端
- 生产构建：`cargo build --release` 会通过 `build.rs` 调用 `frontend/pnpm run build`，再用 `rust-embed` 将 `frontend/dist` 内嵌进单二进制
- Docker 镜像发布：`.github/workflows/publish-docker.yml` 在 `v*` tag、release published 或手动触发时，用 GitHub Actions 构建并推送 `linux/amd64` 镜像到 Docker Hub `594mantou/ai-guard`
- 运行时路由：
  - `/` 和前端静态资源由 Rust 内嵌资源服务
  - `/api/*` 是前端访问后端数据的管理接口
  - `/v1/*` 和 `/openai/*` 转发 OpenAI 兼容请求
  - `/anthropic/*` 转发 Anthropic 兼容请求

## 目录职责

- `src/`：Rust 后端，包含管理 API、账号路由、代理、审查、临时持久化和内嵌静态资源服务
- `frontend/`：Gem + duoyun-ui 前端，Rspack 构建
- `frontend/src/main.ts`：前端入口，导入根元素；HTML 根标签为 `<ai-guard-app>`
- `frontend/src/elements/`：Gem 元素模块，本项目自定义元素统一使用 `ai-guard-*` 前缀
- `frontend/src/api.ts`：前端请求封装
- `frontend/src/store.ts`：前端全局状态
- `frontend/src/utils.ts`：格式化、输入读取等纯工具
- `frontend/src/main.css`：Tailwind CSS v4 全局样式入口
- `frontend/public/`：Rspack HTML 模板和公开静态资源；图标放在 `frontend/public/icons/`
- `frontend/dist/`：前端构建产物，同时是 `rust-embed` 的内嵌目录

## 关键文件

- `Cargo.toml`：Rust 依赖和 `build.rs` 入口声明
- `build.rs`：生产构建前端；debug 默认跳过，可用 `AI_GUARD_BUILD_FRONTEND=1` 强制构建
- `.github/workflows/publish-docker.yml`：Docker Hub 镜像发布，依赖 `DOCKER_USERNAME` 和 `DOCKER_PASSWORD` secrets，目标平台为 `linux/amd64`
- `src/main.rs`：Axum 服务入口和路由装配
- `src/api.rs`：管理接口 `/api/*`
- `src/proxy.rs`：OpenAI/Anthropic 入口转发、上游选择、代理日志
- `src/audit.rs`：异步 OpenRouter 审查
- `src/payload.rs`：请求/响应结构化内容提取，用于日志和审计 payload
- `src/store.rs`：JSON 文件状态存储，默认 `~/.ai-guard/state.json`
- `src/assets.rs`：前端资源内嵌和 SPA fallback
- `frontend/rspack.config.mjs`：Rspack dev/build 配置、dev server 代理、`swc-plugin-gem` 自动导入
- `frontend/auto-import.d.ts`：`swc-plugin-gem` 生成的自动导入类型声明
- `frontend/src/elements/app.ts`：应用外壳，基于 `duoyun-ui/patterns/console` 定义路由、侧边栏
- `frontend/src/elements/toolbar.ts`：当前路由标题栏
- `frontend/src/elements/account-panel.ts`：账号管理面板，使用 `dy-pat-table` 和 `createForm` 弹窗
- `frontend/src/elements/log-panel.ts`：日志面板，使用 `dy-pat-table` 和详情弹窗
- `frontend/src/elements/report-panel.ts`：审查报告面板，使用 `dy-pat-table` 和报告详情弹窗
- `frontend/src/elements/actions.ts`：前端面板共享异步动作

## 常用命令

- 安装前端依赖：`pnpm --dir frontend install`
- 前端开发服务：`pnpm --dir frontend run dev`
- 前端类型检查：`pnpm --dir frontend run lint`
- 前端生产构建：`pnpm --dir frontend run build`
- Rust 检查：`cargo check`
- Rust 格式化检查：`cargo fmt -- --check`
- 后端开发运行：`cargo run`
- 生产构建：`cargo build --release`

常用环境变量：

- `AI_GUARD_BIND`：监听地址，默认 `127.0.0.1:8787`
- `AI_GUARD_DATA_DIR`：状态目录，默认 `~/.ai-guard`
- `AI_GUARD_OPENROUTER_KEY`：审查用 OpenRouter key；不设置时审查报告标记为 skipped
- `AI_GUARD_AUDIT_MODEL`：审查模型，默认 `qwen/qwen3-4b:free`
- `AI_GUARD_RESEND_KEY`：Resend 邮件 key；和 `AI_GUARD_RESEND_EMAIL` 同时设置后，high/critical 审查报告会发邮件
- `AI_GUARD_RESEND_EMAIL`：high/critical 审查邮件收件人，支持逗号分隔多个邮箱
- `AI_GUARD_RESEND_FROM`：high/critical 审查邮件发件人，默认 `AI Guard <onboarding@resend.dev>`
- `AI_GUARD_BUILD_FRONTEND=1`：debug 构建时也执行前端构建
- `AI_GUARD_SKIP_FRONTEND_BUILD=1`：跳过 build.rs 前端构建
- 支持从当前工作目录的 `.env` 文件读取运行时环境变量，真实环境变量优先级更高

## 维护要求

- 改入口、目录职责、运行链路、构建方式时，优先同步更新这里
- 如果新增页面、模块或目录，先判断是否需要补到“目录职责”和“关键文件”
- 这里不写细节实现，只写后续 Agent 需要的导航信息

# 前端开发

使用 [`@mantou/gem`](https://gemjs.org/) 框架，[`duoyun-ui`](https://duoyun-ui.gemjs.org/) UI 库。页面外壳参考 gem examples console，优先使用 `dy-pat-console`、`dy-pat-table`、`createForm` 等 duoyun-ui pattern，不手写基础表格和弹窗。Rspack 通过 `swc-plugin-gem` 自动导入 Gem 成员、模板内使用到的 `duoyun-ui` 元素和 `ai-guard-*` 本地元素；本地元素文件名按去掉 `ai-guard-` 前缀后的元素名命名，例如 `ai-guard-log-panel` 对应 `frontend/src/elements/log-panel.ts`。需要让 `auto-import.d.ts` 保持存在，删除后会在下一次构建时重新生成。样式使用 Tailwind CSS v4，从 `src/main.css` 入口导入；如果以后确实需要 Shadow DOM，再把该元素私有样式写在相应元素文件开头。请求封装优先使用 `@mantou/gem/helper/request`，全站未捕获错误由 `duoyun-ui/helper/error` 处理。

## Gem Element Development

Files in `elements` folder are for Gem elements. One file contains one or more elements. Filename is the prefix-less element name. Gem elements extend GemElement or its derived classes.

### Gem Syntax Example

```ts
// 如果需要全局状态，就可以创建一个 Store
// 也许是从其他模块中导入的
const store = createStore({
  globalCount: 1,
  text: '',
});

// 一个更新 Store 的函数，Store 即是个数据对象，也可以用来更新内容
// 一般和 Store 的定义写在模块中，也可能没有这样的函数，因为可以直接调用 `store({})` 更新
const addCount = () => store({ globalCount: store.globalCount + 1 });

// 创建一个给元素实例用的主题
// 当元素的样式基于元素的属性时使用这种方法
// 这是个特殊的主题，在应用到元素时他也是个装饰器，作用是用来反应元素属性的变化来更改主题值
const elementTheme = createDecoratorTheme({ color: 'red' });

// 用 `css` 创建 Gem 元素可挂载的样式表，可以使用 CSS 嵌套语法
// 只有元素通过 `@shadow` 定义成了 Shadow DOM，CSS 中才能使用 `:host`
// 否则使用 `:scope`，请注意区分它们的使用方法而不是简单的替换
// 不要在模板内写内联样式，以这种方式定义的样式可以共享，而且和 DOM 分离
// 如果项目定义了主题，CSS 规则值可以从主题读取
const style = css`
  :scope {
    display: block;
    color: ${theme.textColor};
  }
`;

// 复杂的元素，可以使用这个方案编写样式表，在模板中用 `style1.header` 来引用类名
const style1 = css({
  // `$` 表示 `:host` 或 `:scope`
  $: `
    font-size: small;
  `,
  content: `
    font-size: 24px;
    color: ${elementTheme.color};
  `,
});

// 自定义元素标签名，使用统一的 `dy` 命名空间
@customElement('dy-test')
// 将创建的样式表挂载到元素上，使用多次就可以挂载多个样式表
@adoptedStyle(style)
@adoptedStyle(style1)
// 将全局 store 链接到元素上，store 更新时驱动元素更新，使用多次就可以链接多个 store
@connectStore(store)
// 默认是 Light DOM，只有使用了 `@shadow()` 才是 Shadow DOM，参数是 `ShadowRootInit`
@shadow()
// 一般不需要使用，只有该元素的内容需要能被外部样式化时才使用
@light({ penetrable: true })
// 指定元素渲染不会阻塞主线程，如果这个元素需要一次渲染很多个实例，可以使用
@async()
// 用来指定元素的 ARIA 属性，加强元素的可访问性
@aria({ role: 'region' })
// 这里的元素类名，`Duoyun` 是 `dy` 的全称，后面要加 `Element`，类似原生 HTML 元素类名
class DuoyunTestElement extends GemElement {
  // 定义元素的 part，使用静态字段可以让外部引用 part 名称，不需要设置初始值，状态器会提供一个同名初始值
  static @part img: string;
  // 定义元素的 slot，和 `@part` 一样的原则
  static @slot content: string;
  // 指定一个称为 `src` 的 Attribute，当没有赋值时默认解析成空字符串
  @attribute src: string;
  // 指定一个称为 `count` 的 Attribute，但解析成数字，当没有赋值时默认解析成 `0`
  @numattribute count: number;
  // 指定一个称为 `show` 的 Attribute，但解析成布尔值，当没有赋值时默认解析成 `false`
  @boolattribute show: boolean;
  // 当 Attribute 不能表示的属性时用 Property 表示，由于用户可以不传递属性，所以总要处理为空的情况，更改时会触发元素重新渲染
  @property data?: {};
  // 定义了一个 `display-content` 事件，直接调用触发，参数是自定义事件的 `detail` 属性
  // 只需要指定类型，类型中的参数是自定义事件的 `detail` 属性，`this.displayContent(true)` 触发
  // 很多时候传递数据，就使用 `null` 占位
  // `@globalemitter` 可以穿透 ShadowDOM 进行冒泡
  @emitter displayContent: Emitter<boolean>;
  // 定义 CSS 状态，仅仅是用来供外部 CSS 选择器使用，例如 `dy-test:state(open)`
  // 修改方法：`this.open = true`，没有特别的限制
  @state open: boolean;

  // 创建一个 { value?: HTMLImageElement } 对象，用来访问 DOM
  #imgRef = createRef<HTMLImageElement>();
  // 创建一个内部状态对象，`this.#state({ ... })` 来更新状态
  // 元素内部不应该更新元素的 Attribute/Property，就像原生元素一样
  // 注意和 CSS 状态 `@state` 无关
  #state = createState({ internalCount: 1 });

  // Attribute 不要赋初始值，因为 DOM 序列化会多出以内容，如果需要默认值，可以定义一个 `getter`
  // Property 可以赋初始值，但也可以同样用 `getter`
  get #src() {
    return this.src || 'test';
  }

  // 一些复杂计算可以使用 `@memo`，他的参数是一个函数，参数是当前实例，返回一个依赖数组
  // 在元素每次渲染前执行，只有依赖数组有更改时才会执行函数内容
  // 基于 `@memo` 实现了 `@willMount`
  @memo((i) => [i.src])
  get #text() {
    return i.src.repeat(10);
  }

  // 每次渲染后的副作用，参数和 `@memo` 一样，没有参数时每次都执行
  // 返回的函数会作为清理函数，在下次调用前执行
  // 类似 React 的 `useLayoutEffect`
  // 基于 `@effect` 实现了 `@mounted` `@unmounted`
  @effect()
  #print = () => {
    console.log('updated');
    return () => console.log('clear');
  }

  // `@template` 指定模板渲染函数，参数是一个条件函数，可以为不同条件指定不同渲染内容
  // 不提供条件函数时直接认为满足条件
  @template()
  #content = () => {
    const imgProps = { dataTest: 1 };
    // 模板语法基于 lit-html，添加了 Vue 的 `v-if` 语法、Ref 语法和剩余属性语法
    // 必要时候使用 `classMap` `styleMap` `partMap` `exportPartsMap`
    return html`
      <img ${this.#imgRef} ${imgProps} src=${this.#src} part=${DuoyunTestElement.part} />
      <div class=${classMap({ div: true })} v-if=${this.show}>Show</div>
      <div v-else class=${style1.content} style=${styleMap({ fontSize: '10px' })}>None</div>
    `;
  }

  // 当元素更新后，会根据依赖是否变化重新计算主题，不提供依赖函数则每次更新都更新主题
  @elementTheme((i) => [i.show])
  #updateTheme = () => ({ color: this.show ? 'red' : 'blue' });

  // 渲染出错时的后备内容，只有可能会渲染出错时才需要提供后备模板内容
  @fallback()
  #errorContent = (err) => {
    return html`Error: ${err}`;
  }

  // Gem 元素使用 ES 装饰器定义特性，装饰器本身就完整的表示了意义，所以不需要额外写自定义元素声明
  // Gem 元素不要使用生命周期函数，应该使用各种装饰器装饰普通函数，生命周期已经弃用了!!!
  // 应该尽量使用 ES 私有字段（`#aaa`）来替代类方法，这样没有 `this` 指向的问题
}

```

### Gem Best Practices

- [other](packages/gem/docs/en/004-blog/001-create-standard-element.md)
