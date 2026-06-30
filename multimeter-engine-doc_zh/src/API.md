# API指南

本程序的Request和Response均为JSON格式。

## Request

Request有`version`、`id`、`kind`和`payload`四个参数，任一的缺失都会导致Request无法执行。

### version

本Request使用的Request版本，类型是整型。目前仅有`1`一个版本。

### id

本Request的id，类型是字符串。id使用字符串是以便于设计出适合人类阅读的id为目的考虑的。id没有格式限制，但不可重复。

> [!NOTE]
> 当前版本并不强制限制id不可重复，但将在后期版本进行强制限制

### kind与payload

`kind`表明本Request的目的，类型是字符串。`payload`是相应的具体信息，它是一个JSON对象，包含字符串类型的`value`和字符串列表类型的`addition`。若没有额外信息，则`addition
`应赋值为`null`。

#### 获取软硬件信息

kind`get_info`表明本Request用于获取电脑的软硬件数据。相应有以下可获取的信息：

| 名称                     | 作用        | 类型           | 单位  |
|------------------------|-----------|--------------|-----|
| cpu_name               | CPU名称     | string       |     |
| cpu_temperature        | CPU温度     | double       | ℃   |
| cpu_temperature_first  | CPU首核温度   | double       | ℃   |
| cpu_temperature_last   | CPU末核温度   | double       | ℃   |
| cpu_tjmax_first        | CPU首核最高结温 | double       | ℃   |
| cpu_tjmax_last         | CPU末核最高结温 | double       | ℃   |
| cpu_power              | CPU功耗     | double       | W   |
| cpu_voltage_first      | CPU首核电压   | double       | V   |
| cpu_voltage_last       | CPU末核电压   | double       | V   |
| cpu_voltage            | CPU电压     | double       | V   |
| cpu_clock_first        | CPU首核频率   | double       | MHz |
| cpu_clock_last         | CPU末核频率   | double       | MHz |
| cpu_clock_avg          | CPU平均频率   | double       | MHz |
| cpu_clock_rms          | CPU等价频率   | double       | MHz |
| cpu_clock_max          | CPU最大频率   | double       | MHz |
| cpu_usage              | CPU使用率    | double       | %   |
| cpu_usage_first        | CPU首核使用率  | double       | %   |
| cpu_usage_last         | CPU末核使用率  | double       | %   |
| gpu_name               | GPU名称     | string       |     |
| gpu_temperature        | GPU温度     | double       | ℃   |
| gpu_power              | GPU功耗     | double       | W   |
| gpu_clock_rms          | GPU等价频率   | double       | MHz |
| gpu_mem_clock_rms      | GPU显存等价频率 | double       | MHz |
| gpu_usage              | GPU使用率    | double       | %   |
| mem_percentage         | 内存使用百分比   | double       | %   |
| mem_available          | 可用内存      | double       | GB  |
| mem_used               | 已用内存      | double       | GB  |
| bat_capacity_max       | 电池最大容量    | double       | Wh  |
| bat_capacity_remain    | 电池剩余容量    | double       | Wh  |
| bat_capacity_designed  | 电池设计容量    | double       | Wh  |
| bat_voltage            | 电池电压      | double       | V   |
| bat_rate               | 电池充放电速率   | double       | W   |
| bat_state              | 电池充电状态    | boolean      |     |
| os_activated           | 操作系统激活状态  | boolean      |     |
| disk_temperature_first | 首块硬盘温度    | double       | ℃   |
| disk_temperature_last  | 末块硬盘温度    | double       | ℃   |
| disk_disk_size         | 硬盘容量      | string array |     |

## Response

Response有`version`、`id`、`state`和`payload`四个参数。

### version

本Response使用的Response版本，类型是整型。目前仅有`1`一个版本。

### id

本Response的id，类型是字符串。Response的id就是对应Request的id。

### state

本Response的状态。

| state | 描述      |
|-------|---------|
| 100   | 程序正常执行  |
| 404   | 请求内容不存在 |
| 500   | 程序内部错误  |

### payload

此payload与Request中一致。

对于`get_info`，`value`为获取信息的数值，`addition`为`null`。

对于404或500情况，`value`为报错信息，`addition`为`null`。

## 示例

### 获取信息

Request:

```json
{"version": 1,"id": "from_desktop_1","kind": "get_info","payload": {"value": "cpu_power","addition": null}}
```

Response:

```json
{"version":1,"id":"from_desktop_1","state":100,"payload":{"value":19.03214454650879,"addition":null}}
```

### 404错误

Request:

```json
{"version": 1,"id": "from_desktop_1","kind": "get_info","payload": {"value": "cpu_pooower","addition": null}}
```

Response:

```json
{"version":1,"id":"from_desktop_1","state":404,"payload":{"value":"Failed to process request: Unknown query target: cpu_powerr","addition":null}}
```

### 请求格式错误
    
Request:
```json
{"version": 1,"id": "from_desktop_1","kind": "get_info","payload": {"value": "cpu_power","addition": null}
```

Response:

```json
{"version":1,"id":"__default_id__","state":404,"payload":{"value":"Failed to parse request: \"{\\\"version\\\": 1,\\\"id\\\": \\\"from_desktop_1\\\",\\\"kind\\\": \\\"get_info\\\",\\\"payload\\\": {\\\"value\\\": \\\"cpu_power\\\",\\\"addition\\\": null}\"","addition":null}}
```
