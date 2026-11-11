# Synra 热键

Hotkey 是一个手机端快捷键面板：在手机上选择已配对的电脑，点击固定的快捷键方块，让电脑执行对应的系统组合键。

## 当前功能

- 打开插件后直接展示系统基础快捷键列表。
- 方块按“图标、中文动作名称、灰色小号快捷键组合”展示。
- 点击方块触发固定快捷键，并显示按下反馈。
- 编辑模式支持拖拽调整方块顺序。
- 支持每行 3、4、5 列，方块保持正方形并自动适配屏幕宽度。
- 列数和方块顺序保存在手机端偏好中。
- 快捷键组合由插件内置，不能修改或新增。
- 发送请求会携带目标电脑的平台，电脑端再转换为该平台的固定按键格式。

当前内置快捷键包括复制、剪切、粘贴、撤销、重做、全选、保存、查找和打开文件夹。

## 运行方式

1. 在 Synra 中配对手机和电脑，并允许本插件的设备发送和电脑端模拟按键权限。
2. 在手机端打开本插件，选择一台在线电脑。
3. 点击快捷键方块；编辑时先点击“编辑”，拖拽完成后点击“完成”。

电脑端还必须在 Synra 主机设置中开启系统级“允许插件模拟按键”策略；插件权限页的 `platform:simulate-keys` 授权与这项主机策略是两层独立开关。

电脑端只作为事件接收端执行固定快捷键。插件不会把任意按键字符串交给电脑端，电脑端只接受内置快捷键 ID，并在接收端映射为平台格式。

界面图标使用 UnoCSS 的 `presetIcons`，Carbon 图标在构建时内联到 `ui.zip`，运行时不依赖网络。

## 开发

```text
pnpm install
pnpm run check
pnpm test
pnpm run lint
cargo fmt --all --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```

构建真实插件组件需要 Synra Plugin CLI：

```text
synra-plugin build
synra-plugin verify --dir dist
```

设计文档位于 [ai-docs/design-docs](./ai-docs/design-docs)。

## 许可证

MIT
