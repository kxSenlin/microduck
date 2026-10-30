# 本机 fork 说明（kxSenlin/microduck，分支 `local/wsl`）

这是 [pollen-robotics/microduck](https://github.com/pollen-robotics/microduck) 的 fork，
上游默认分支是 `main`。**本仓库所有本地改动都在 `local/wsl` 分支上**，上游提交保持原样，
方便随时 merge：`git merge upstream/main`。

换一台 WSL 机器复现时，先照 **[duck-client 的 docs/local-setup.md](https://github.com/kxSenlin/duck-client/blob/master/docs/local-setup.md)**
准备系统环境（WSLg、`systemd-container`、sudo、GPU、`~/.bashrc` 托管块、`~/Pollen/` 布局），
再回来看这份文件。

## 本 fork 改了什么

相对上游 `main` 只有 4 个文件、253 行增量（`git diff --stat upstream/main...HEAD`）：

| 文件 | 改动 |
|---|---|
| `scripts/duck-sim` | 1) Debian rootfs 换阿里云镜像；2) `DUCK_SIM_VIEWER=viser` 时给 `body_server` 传 `--viewer viser --viewer-port`；3) 每只鸭子默认一个相机（`DUCK_SIM_CAMERAS=all`、720p30、帧端口 7901+），`DUCK_SIM_CAMERA_FPS` 可调；4) 启动/结尾打印浏览器地址；5) `boot` 先等 realtime 回 1.0、站立失败自动重试；6) 去掉本地原生窗口辅助脚本 |
| `local/wsl-scripts/duck-console` | 四窗口终端控制台：容器 shell / 健康状态 / 合并日志 / 每只鸭子 monitor，另带一个浏览器查看器窗口 |
| `local/wsl-scripts/duck-health` | 容器列表 + 每只鸭子健康状态 |
| `local/wsl-scripts/README.md` | 上述脚本和补丁的说明 |

`local/wsl-scripts/` 是 fork 新增目录，上游没有，所以 merge 上游不会冲突。

## fork 引入/使用的环境变量

客户端和仿真的**权威环境变量表**在
[duck-client/docs/env.md](https://github.com/kxSenlin/duck-client/blob/master/docs/env.md)，
这里只列和本 fork 直接相关的（默认值都是本机实测值）：

| 变量 | 默认值 | 作用 |
|---|---|---|
| `DUCK_SIM_VIEWER` | `viser` | 三维场景：`viser` 浏览器 / `1` 原生窗口 / `0` 无窗口 |
| `DUCK_SIM_VIEWER_PORT` | `8091` | viser 端口 |
| `DUCK_SIM_CAMERAS` | `all` | 开相机的鸭子：`all` / `a` / `a,c` |
| `DUCK_SIM_FRAME_PORT` | `7901` | 第一个相机帧端口；duck-a=7901、duck-b=7902… |
| `DUCK_SIM_CAMERA_FPS` | `30` | 相机帧率（要和 `mediad` 的 30fps 一致，否则日志一直报 capture 欠帧） |
| `DUCK_SIM_CAMERA_STATS` | `0` | 1 = body_server 每 5 秒打印实际帧率/耗时 |
| `DUCK_SIM_DUCKS` | `1` | 鸭子数量 |
| `DUCK_SIM_KEYFRAME` | `SIT` | 起始关键帧 |
| `DUCK_SIM_SCENE` | 空 | `apartment`（默认）/ 场景文件名 |
| `DUCK_SIM_RL` | `$HOME/Pollen/microduck_rl` | 仿真仓库位置（必须指到 fork 的 `local/wsl`） |
| `DUCK_SIM_STATE` | `$HOME/.cache/duck-sim` | 仿真状态目录 |
| `DUCK_SIM_DUCK` / `DUCK_IDENTITY` / `DUCK_RUNTIME_DIR` | — | 进阶覆盖，`scripts/duck-sim` 内使用 |

`~/.bashrc` 里的 `GALLIUM_DRIVER=d3d12`、`PATH=.../microduck/scripts:$PATH`、
`DUCK_SIM_VIEWER=viser` 三条由 duck-client 的 `scripts/local-setup.sh` 写入托管块，**不要手改**。

## 安装与运行

```bash
cd ~/Pollen
git clone git@github.com:kxSenlin/microduck.git
cd microduck
git checkout local/wsl

# 依赖：官方 README 的 Rust 工具链 + 本机约定（WSLg / systemd-container / sudo）
# 首次 boot 会自动 cargo build daemons（较慢）；之后走缓存。

# 起仿真（浏览器查看器 + 每只鸭子 720p 相机）
scripts/duck-sim boot 2

# 浏览器查看器
#   http://localhost:8091
# 监控（另一个终端）
duck-console
```

客户端（duck-client）只负责连端点，**不会启动/停止鸭子**：

```bash
cd ~/Pollen/duck-client
duck-client            # http://localhost:8617
```

## 与上游同步

```bash
cd ~/Pollen/microduck
git fetch upstream
git checkout local/wsl
git merge upstream/main       # 2026-10-30 已同步到 0.18.1（49 个上游提交，无冲突）
scripts/duck-sim down         # 重建 daemons 前先停
scripts/duck-sim boot 2       # 重新构建并验证
```

冲突几乎只会出现在 `scripts/duck-sim`（上游也在改它）。解决原则：**保留本 fork 的
`DUCK_SIM_CAMERAS`/`DUCK_SIM_CAMERA_FPS`/viser 参数，再吸收上游逻辑**；改完必须跑一次
`scripts/duck-sim boot` 验证。

## 快速验证（换机后）

```bash
cd ~/Pollen/microduck
git branch --show-current                       # local/wsl
bash -n scripts/duck-sim                        # 语法
scripts/duck-sim boot 1                         # 起一只
curl -s http://localhost:8091 >/dev/null && echo "viser ok"
cat ~/.cache/duck-sim/truth.json | head -c 120  # 应为 JSON（duck-client 的真值通道）
scripts/duck-sim down
```
