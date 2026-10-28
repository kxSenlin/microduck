# WSL 本地补丁与脚本

这个目录里的内容不属于官方仓库，是 kxSenlin 在 WSL 上使用 Microduck 时的本地补丁与辅助脚本。

## 补丁

| 文件 | 内容 |
|---|---|
| `scripts/duck-sim` | Debian rootfs 下载改用阿里云镜像（`mirrors.aliyun.com`）；`DUCK_SIM_VIEWER=viser` 时给 `body_server` 传 `--viewer viser --viewer-port 8091` |
| `src/mjlab_microduck/sim/body_server.py`（microduck_rl） | 新增 `--viewer native|viser` 与 `--viewer-port`；viser 模式用 mjviser 把场景同步到浏览器，由 Windows GPU 渲染 |

## 为什么改用浏览器渲染

WSLg 下 MuJoCo 原生 GLFW 窗口的渲染由 CPU 端的 Mesa→D3D12 翻译承担：

| 场景 | 原生窗口 | 浏览器（mjviser） |
|---|---|---|
| 公寓（233 geom） | 165 ms/帧，body_server 264% CPU，GPU 5% | 场景同步 2 ms/帧，GPU 由浏览器使用 |
| 地面（82 geom） | 26 ms/帧 | 同上 |

## 用法

```bash
DUCK_SIM_VIEWER=viser scripts/duck-sim            # 单机，浏览器 http://localhost:8091
DUCK_SIM_VIEWER=viser scripts/duck-sim boot 2     # 容器模式
DUCK_SIM_VIEWER=1 scripts/duck-sim                # 原生窗口（默认）
DUCK_SIM_VIEWER=0 scripts/duck-sim                # 无窗口
```

`mjviser` 需要装在 `microduck_rl/.venv`：

```bash
cd ~/Pollen/microduck_rl && uv pip install mjviser
```

## 脚本

| 脚本 | 用途 |
|---|---|
| `duck-console` | 起鸭子并打开 tmux 监控 tab；`duck-console` 单机，`duck-console boot 2` 容器模式 |
| `duck-view` | 打开并定位 MuJoCo 原生窗口 |
| `duck-hide` / `duck-show` | 隐藏 / 恢复 MuJoCo 窗口（鸭子继续运行） |

安装：

```bash
mkdir -p ~/bin ~/.local/bin
cp local/wsl-scripts/duck-console ~/bin/
cp local/wsl-scripts/duck-view local/wsl-scripts/duck-hide local/wsl-scripts/duck-show ~/.local/bin/
chmod 755 ~/bin/duck-console ~/.local/bin/duck-{view,hide,show}
```

## WSL 环境要求

- WSLg + `GALLIUM_DRIVER=d3d12`（否则走 llvmpipe）
- `systemd-container`、`mmdebstrap`（容器模式 `boot`）
- 免密 sudo（`/etc/sudoers.d/90-wsl-nopasswd`）
- `gh`（GitHub CLI，仅用于同步 fork）