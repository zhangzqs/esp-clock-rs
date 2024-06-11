# 基于 Rust + SlintUI 实现的跨全平台的小电视项目

## 简介

这是一个使用 Rust 开发的跨平台，分辨率在 240x240 尺寸大小的彩屏小电视项目。

## 功能列表

- [x] 启动引导页
- [x] 天气时钟首页
- [x] 功能菜单页
- [x] MIDI 音乐播放器
- [x] 天气预报页
- [ ] PC 性能监视器
- [ ] PC 投屏
- [ ] 闹钟
- [ ] 后台管理 Vue 前端
- [ ] 后台管理 Shell

## 项目展示

TODO

## 编译运行

### ESP32C3

```bash
cd app/esp32c3-impl
cargo run -r
```

### 桌面端

```bash
cd app/desktop-impl
cargo run
```

### 浏览器端

```bash
cd app/wasm-impl
make release
make serve
```

## 方案说明

### 支持平台

- ESP32C3
  - **第一优先级**支持
- 浏览器端(WASM)
  - **第二优先级**支持
- 桌面端(Windows/Linux/MacOS)
  - **第三优先级**支持
- 桌面端(软件渲染器)
  - 主要用来做桌面端仿真 ESP32C3 中的运行效果
  - 预期应当做到和桌面端复用相同的非 ui 相关代码，后续可考虑使用条件编译开关与桌面端合并为一个 crate 实现（TODO）。
- 移动端
  - 原先 v1-old 分支中的旧版本支持，新版本未来可能会支持适配(TODO)

### app 中的各个 crate 介绍

- app-core(平台通用实现 lib)

  - 实现了消息调度器框架
  - 时间，GUI 分别依赖了跨平台的 time 库和 SlintUI 框架，所以平台无关
  - 部分平台相关的组件的默认实现
    - storage: 使用内存中的 HashMap 模拟实现
    - onebutton: 使用 SlintUI 框架的鼠标与键盘事件，结合 button-driver 第三方 crate 实现
    - system: 该模块用于 ESP32 上的内存调试使用，默认 mock 固定值实现

- esp32c3-impl(ESP32C3 端实现 bin)

  - 平台相关组件实现
    - httpclient: 基于 esp-idf-svc 的 httpclient 实现
    - httpserver: 基于 esp-idf-svc 的 httpserver 实现
    - buzzer: 基于 RMT 驱动蜂鸣器，midiplayer 的默认实现将依赖 buzzer
    - storage: 基于 ESP NVS 分区实现 KV 存储后端

- desktop-impl(桌面端实现 bin)

  - 平台相关组件实现
    - httpclient: 基于线程池+阻塞的 reqwest 实现(TODO: 改为 async 实现)
    - httpserver: 基于 tiny_http 实现
    - midiplayer: TODO

- wasm-impl(浏览器端实现 bin)

  - 平台相关组件实现
    - httpclient: 基于浏览器上的异步的 reqwest 支持
    - storage: 基于 localStorage 支持
    - midiplayer: MIDI.js 库提供支持

- admin-cli(app 管理后台工具)

  - 向 app 通过 http 发送 json 消息实现基于 RPC 的消息调用可轻易实现很多后台管理功能

- proto(消息包)
  - 所有消息实体的定义
  - IPC/RPC client 的封装
  - 相关抽象 trait 的定义

### 天气 API

天气 API 使用`和风天气`，对接了用于查询地理位置的 GeoApi，免费天气的 DevApi，付费的 API(TODO)。
由于和风天气强制使用 https 和 gzip 压缩，故在 ESP32C3 上引入了常用证书库，引入了 libflate crate 对响应进行解压，故相对于非 gzip 压缩和未加密的 http 请求而言，更消耗 ESP32C3 上的内存资源。

### 内存使用

Boot 页面下，双击按键可打开屏幕上悬浮的性能监视窗口。可用于实时检测剩余内存，剩余可分配的最大内存空闲块，FPS 计数器等信息。

由于 ESP32C3 内存资源有限，目前进入首页获取完成天气数据之后，剩余可用内存为 57KB 左右，最大连续空闲内存为 36KB 左右，在播放 MIDI 音频时候更占据大量的内存资源，仍需持续优化内存资源占用。

#### 消息的内存占用

以下是截至目前各个平台运行时单个消息体的栈空间内存占用（不包含具有堆内存分配的数据结构在堆上占据的内存大小）

| 运行平台          | ESP32C3 | Linux x86_64 |
| ----------------- | ------- | ------------ |
| Message           | 40B     | 64B          |
| MessageWithHeader | 72B     | 120B         |
| MessageQueueItem  | 88B     | 144B         |

#### 内存优化

- Scheduler 中，若 handle_message 后返回为 Pending 状态，则后续 poll 携带的 body 均为 Message::Empty，不再包含完整 Body，这对消息中包含堆内存的数据结构可减少重复的 clone 开销
- UI 中各个页面在跳转前进入 Hide 状态时，需要重新设置一些 UI 的 ViewModel 数据为 default，以释放 ui 资源内存占用
- MIDI 音乐每一轮切换需要关闭先前的音乐，先释放先前的内存占用后再切换下一首
- 当前的消息体积在 64 位桌面端上大小为 32 字节

## 消息通信机制的设计

整套程序采用消息传递机制完成整个 app 框架的设计，各个组件仅通过消息进行相互耦合实现通信，各个通信节点称为`Node`，通过枚举`NodeName`可以唯一标识一个组件。平台无关的组件放置在 app-core 中，平台相关的组件放置在各个平台的实现中。所有消息均实现了`serde::Serialize`和`serde::Deserialize`，故可天然通过 http 或 mqtt 传输 app 内的任意消息，使得 RPC 调用程序内的任意功能成为一个天然的可能，无需专门编写复杂的接口适配，同时这也为分布式 app 的可能性奠定了基础，app 内的各个组件节点可以分布工作在其他远程机器之上。

实际上本项目的实现实际上可以和 ROS 或微服务中的一些机制进行对比：

|                | 本项目                                                    | ROS                | 微服务            |
| -------------- | --------------------------------------------------------- | ------------------ | ----------------- |
| 一对多通信     | broadcast_global/broadcast_topic 广播机制                 | 话题通信机制       | 消息队列          |
| 一对一同步通信 | sync_call/async_call 消息调用机制                         | 服务通信机制       | RPC 通信          |
| 参数配置       | 基于 sync_call 的 KV Storage 模块                         | 参数服务器机制     | 配置中心          |
| 消息传输格式   | 本地消息通信直接内存访问，远程通过 serde 以 JSON 形式传输 | ROS 消息序列化格式 | json/protobuf/... |

### App 中消息通信机制的一些基本概念

**节点 Node**
一个节点实际上就作为 app 内的一个组件，它可以向其他组件发送消息也可以接收来自其他组件的消息。

**发送源**
发送源通常使用`from: NodeName`来标识消息从哪个`Node`发出，`Schedular`是一个特殊的`Node`标识调度器消息。

**发送目标 MessageTo**

- Boardcast: 某个组件可以向所有组件发起一个广播消息，其他组件均可接收到广播消息
- Topic(TopicName): 某个组件可以向一个指定的话题发消息，其他订阅该话题的组件均可接收到该广播消息
- Point(NodeName): 一个组件可以向另一个组件发送消息

**消息处理结果 HandleResult**
当一个消息被处理完成后，需要反馈一个消息处理结果，目前定义了三种处理结果：

- Finish(Message): 消息成功处理，并反馈结果消息给消息发送者。
- Discard: 消息被丢弃不处理，无任何反馈结果。
- Pending: 消息需要进入 Pending 状态，后续调度器将不断周期性执行 poll 函数根据消息的唯一 seq 标识轮询结果消息，直到 Context 中的 async_ready 调用后标识消息处理完成的结果后消息才将回调给最初的发送者。

**同步消息调用**

Context 中的 `sync_call(ctx, message) -> HandleResult` 为同步消息调用，组件间的通信实际上就是直接的函数调用。

同步调用的优点：

1. 简化组件的使用，可使得编码风格形成更加自然的业务流程的顺序调用。

同步调用的缺点：

注意这种调用可能会造成一些问题，如：

1. 两个组件互相循环通信时可能造成对同一个变量的两次可变借用从而使程序崩溃或获取两次锁造成死锁。
2. 组件相互通信可能导致程序陷入死循环，其他所有消息均无法正常调度，程序将死机。
3. 消息处理必须是短时间内可直接执行完毕的消息，调用完成后必须立刻返回 Finish 或 Discard 状态，禁止返回 Pending 状态。

**异步消息调用**

Context 中的 `async_call(ctx, message, FnOnce(HandleResult))` 为异步消息调用，实际上是发送一个异步消息到调度器的调度队列中进行调度，某一时刻若消息完成则通过回调函数异步通知发送者消息执行完毕。

异步调用的优点：

1. 两个组件循环通信本质上都是调度器在两个节点上依次消息调度，即使组件陷入长时间的消息循环，甚至无限循环，其他消息依旧可以正常调度。
2. 可以天然表达异步耗时任务的调用，可以使用 Pending 状态表明该消息需要等待一段时间后才能返回结果。

异步调用的缺点：

1. 组件调用将必须以回调的形式接收结果，影响了代码风格。
   当然若将 Rust 的 async 机制对接到调度器，可大大简化异步消息的代码风格(TODO)。
