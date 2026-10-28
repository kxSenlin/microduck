# WSL 本地补丁与脚本

不属于官方仓库，是 kxSenlin 在 WSL 上使用 Microduck 的本地补丁与辅助脚本。

## 补丁

| 仓库 | 文件 | 内容 |
|---|---|---|
| microduck | `scripts/duck-sim` | 1) Debian rootfs 下载改用阿里云镜像；2) `DUCK_SIM_VIEWER=viser` 时给 `body_server` 传 `--viewer viser --viewer-port`；3) 启动与结尾打印浏览器地址 |
| microduck_rl | `src/mjlab_microduck/sim/body_server.py` | 新增 `--viewer native|viser` 与 `--viewer-port`；viser 模式用 mjviser 把场景同步到浏览器，由 Windows GPU 渲染 |

## 为什么用浏览器渲染

WSLg 下 MuJoCo 原生 GLFW 窗口的渲染由 CPU 端的 Mesa→D3D12 翻译承担：

| 场景 | 原生窗口 | 浏览器（mjviser） |
|---|---|---|
| 公寓（233 geom） | 165 ms/帧，body_server 264% CPU，GPU 5% | 场景同步 2 ms/帧，GPU 由浏览器使用 |
| 地面（82 geom） | 26 ms/帧 | 同上 |

**已知问题**：4 只鸭子 + 浏览器时，首次场景推流会阻塞身体循环数秒，`boot` 的站立判定可能误报。
身体循环恢复（`realtime 1.0`）后逐只执行 `robotctl robot enable` 即可站立。

## 使用流程

```bash
# 终端 A：启动（boot 模式，浏览器查看器）
cd ~/Pollen/microduck
scripts/duck-sim boot 4

# 浏览器
http://localhost:8091

# 终端 B：监控
duck-console
```

`duck-console` 只监控、不启动不停止。四个窗口：

| 窗口 | 内容 |
|---|---|
| shell | 普通 shell，打印浏览器地址 |
| status | `duck-health`：容器列表 + 每只鸭子健康 |
| logs | 所有鸭子的 robotd 日志合并（`journalctl -M … -f`） |
| monitor | duck-a 的实时面板（`robotctl monitor`，走容器绑出的 socket） |

收工：

```bash
scripts/duck-sim down
```

## 依赖

```bash
cd ~/Pollen/microduck_rl && uv pip install mjviser
```

WSL 环境：WSLg + `GALLIUM_DRIVER=d3d12`；`systemd-container`、`mmdebstrap`；免密 sudo；`~/.bashrc` 里 `export DUCK_SIM_VIEWER=viser`。

## 脚本安装

```bash
mkdir -p ~/bin ~/.local/bin
cp local/wsl-scripts/duck-console ~/bin/
cp local/wsl-scripts/duck-health ~/.local/bin/
chmod 755 ~/bin/duck-console ~/.local/bin/duck-health
```