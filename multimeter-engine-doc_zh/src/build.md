#  运行程序

本项目于[Github仓库](https://github.com/Potato-Yao/multimeter-engine/releases)提供了便携版可执行程序，在终端中使用管理员权限运行`mutltimeter-engine.
exe`即可启动。本程序默认在`127.0.0.1:8080`端口开放，在程序完成启动后会打印`Server starts at 127.0.0.1:8080`字样，之后即可使用TCP协议与程序进行沟通。

> [!WARNING]
> 请不要改变`externals`文件夹与`mutlimeter-engine.exe`的相对位置。

## 启动参数

| 参数          | 作用        | 示例           |
|-------------|-----------|--------------|
| -p, -\-port | 改变程序开放的端口 | -\-port 8081 |

## 使用TCP协议沟通

> [!TIP]
> 推荐的用于与本程序进行沟通的软件：[Ncat](https://nmap.org/ncat/)

在程序正常运行后，使用任意支持TCP协议的软件向程序开放的端口输入Request即可获得相应的数据或执行相应的操作，结果以Response返回。Request格式见[API文档](API.md)。

## 运行示例

启动本程序：

```shell
./multimeter-engine.exe # 启动程序
# 也可以使用启动参数，如 ./multimeter-engine.exe --port 8081
```

在程序正常启动（出现`Server starts at 127.0.0.1:8080`字样）后，使用`Ncat`与其连接：

```shell
ncat.exe 127.0.0.1 8080 # 或换成你实际使用的端口
```

随后在`Ncat`中输入Request（目前只支持单行JSON作为Request）：

```json
{"version": 1,"id": "from_desktop_1","kind": "get_info","payload": {"value": "cpu_power","addition": null}}
```

即可得到Response：

```json
{"version":1,"id":"from_desktop_1","state":100,"payload":{"value":19.03214454650879,"addition":null}}
```

若要退出本程序，直接在终端中使用`Ctrl+C`即可。
