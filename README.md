## 键鼠宏脚本

### 使用 
```shell
# 运行脚本
./kmm.exe run ./config.toml

# 获取按键代码
./kmm.exe event

# 获取坐标: AltGr(右) 获取当前鼠标坐标 Esc 清屏
./kmm.exe point
```

###  在某些软件/游戏上可能没反应
- 这些软件可能是 root 权限打开的
- kmm 也需要 root 权限打开才能生效
- 可以生成 kmm 的快捷方式配置好启动命令右键管理员启动

### 配置说明
```toml
# 全局延迟
delay = 20
# 缩放比例
scaling = 1.5
# 偏移位置(一般双屏才用)
offset = [0, 0]
# 窗口位置
point = [800.0, 80.0]
# 字体大小
font_size = 20.0
# 字体颜色
font_color = [97, 218, 217]
# 是否显示边框
border = false

# 脚本 A
[[scripts]]
title = "退出程序"
# 单独配置延迟
delay = 10
repeat = 1
trigger = [{ key = "End" }]
methods = [{ event = "Exit" }]

# 脚本 B
[[scripts]]
title = "鼠标侧键回到桌面"
repeat = 1
trigger = [{ mouse = { Unknown = 1 } }]
methods = [{ event = "Keys", args = ["MetaLeft", "KeyD"] }]

# 脚本 C
[[scripts]]
title = "测试显示"
repeat = 4
trigger = [{ key = "Home" }]
methods = [{ event = "Sleep", args = 500 }]

# 脚本 D
[[scripts]]
title = "XXX"
# 执行次数; PS: 0 无限循环直到下次触发停止
repeat = 0
# 触发的按键; PS: 没有数量和按键限制
trigger = [{ key = "PageUp" }]
# 脚本方法
methods = [
    # 点击当前鼠标位置
    { event = "Click", args = "Left" },
    # 按下鼠标
    { event = "ClickDown", args = "Left" },
    # 松开鼠标
    { event = "ClickUp", args = "Left" },
    # 移动鼠标到指定位置并点击; 参数: [x, y]
    { event = "ClickOn", args = ["Left", 2140.0, 1075.0] },
    # 拖拽到指定位置; 参数: [x, y, x2, y2]
    { event = "ClickTo", args = ["Left", 100.0, 100.0, 2140.0, 1075.0, 100] },
    # 点击键盘按键; 参数: 下表的 Key
    { event = "Key", args = "KeyA" },
    # 按下键盘按键
    { event = "KeyDown", args = "KeyA" },
    # 松开键盘按键
    { event = "KeyUp", args = "KeyA" },
    # 点击多个键盘按键; 参数: [下表的 Key] PS: 同时 down 和 up 可以触发组合键
    { event = "Keys", args = ["KeyA", "KeyB", { Unknown = 999 }] },
    # 鼠标移动到指定位置; 参数: [x, y]
    { event = "Move", args = [2140.0, 1075.0] },
    # 滚轮移动; 参数: [x, y] PS: 正数向上/右滚 负数向下/左滚
    { event = "Scroll", args = [0, -100] },
    # 睡眠(执行间隔); 参数: ms
    { event = "Sleep", args = 100 },
    # 退出整个程序
    { event = "Exit" },
]
```