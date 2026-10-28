# WSL 本地补丁与脚本

不属于官方仓库，是 kxSenlin 在 WSL 上使用 Microduck 的本地补丁与辅助脚本。

## 补丁

| 仓库 | 文件 | 内容 |
|---|---|---|
| microduck | `scripts/duck-sim` | Debian rootfs 下载改用阿里云镜像；`DUCK_SIM_VIEWER=viser` 时给 `body_server` 传 `--viewer viser --viewer-port 8091` |
| microduck_rl | `src/mjlab_microduck/sim/body_server.py` | 新增 `--viewer native|viser` 与 `--viewer-port`；viser 模式用 mjviser 把场景同步到浏览器 |

## 为什么用浏览器渲染

WSLg 下 MuJoCo 原生 GLFW 窗口的渲染由 CPU 端的 Mesa→D3D12 翻译承担：

| 场景 | 原生窗口 | 浏览器（mjviser） |
|---|---|---|
| 公寓（233 geom） | 165 ms/帧，body_server 264% CPU，GPU 5% | 场景同步 2 ms/帧，GPU 由浏览器使用 |
| 地面（82 geom） | 26 ms/帧 | 同上 |

## 使用流程

```bash
# 终端 A：启动鸭子（boot 模式，浏览器查看器）
cd ~/Pollen/microduck
DUCK_SIM_VIEWER=viser scripts/duck-sim boot 2

# 浏览器打开场景
http://localhost:8091

# 终端 B：打开监控窗口
duck-console
```

`duck-console` 不启动、不停止鸭子，只打开 tmux 监控窗口：
shell / status / 每只鸭子的 robotd 日志 / body 日志 / health。
鸭子未运行时，`duck-console` 打印上面的启动命令后退出。

收工：

```bash
scripts/duck-sim down
```

## 依赖

`mjviser` 需要装在 `microduck_rl/.venv`：

```bash
cd ~/Pollen/microduck_rl && uv pip install mjviser
```

WSL 环境：

- WSLg + `GALLIUM_DRIVER=d3d12`
- `systemd-container`、`mmdebstrap`（boot 模式）
- 免密 sudo（`/etc/sudoers.d/90-wsl-nopasswd`）

## 脚本安装

```bash
mkdir -p ~/bin
cp local/wsl-scripts/duck-console ~/bin/
chmod 755 ~/bin/duck-console
```

## 说明

- `duck-view`、`duck-hide`、`duck-show` 只对原生 MuJoCo 窗口（`DUCK_SIM_VIEWER=1`）有效；浏览器模式下不需要，本目录不再包含。
- 单机模式（`scripts/duck-sim` 不带参数）仍可用，但 `duck-console` 只监控 boot 模式。