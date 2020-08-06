# Taichi Agent
作为交易推送数据搜集程序部署在矿池节点侧。

## 编译部署

### Docker 部署

直接使用本项目中的 Dockerfile 文件制作镜像即可。

### Rust 本地编译部署

安装 Rust 环境: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

编译： `cargo build --release`

## 运行

最简单的参数只需要提供矿池节点名字和 enode 即可，运行命令：`./taichi-agent --name <PooL Name> --pool-enode <enode://...>`。

其它配置参数都是默认的，包括：Geth 节点 Json-Rpc 接口地址、间隔几分钟获取一次数据、Taichi 推送节点的 enode、Taichi 数据接收服务的 Grpc 地址。

可查询 help 信息: `./target/release/taichi-agent --help`

```
Taichi Agent 0.1.0
An Agent to get tx pushed data.

USAGE:
    taichi-agent [FLAGS] [OPTIONS] --name <name> --pool-enode <pool-enode>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
    -v, --verbose    log verbose, defualt WARN, -v Info, -VV DEBUG

OPTIONS:
    -g, --geth-http <geth-http>                            json-rpc url of pool node [default: http://127.0.0.1:8545]
    -n, --name <name>                                      pool name
    -p, --poll-interval-minutes <poll-interval-minutes>    how long to fetch tx pushed data [default: 1]
        --pool-enode <pool-enode>                          enode of pool node
        --taichi-enode <taichi-enode>
            enode of Taichi push node [default:
            enode://24a2bdca9fae77873ecedfbb3d418a524601790de393f8fa62620fd2092429b5c40fbc30d4e89049964dcf9167913a1b2198592044ee72c8eeec4c7c3fd29336@47.114.137.69:32303]
        --taichi-grpc <taichi-grpc>                        Taichi GRpc url [default: http://grpc.taichi.network:11003]
```