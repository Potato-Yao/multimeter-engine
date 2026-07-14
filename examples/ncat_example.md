to send request via `ncat`:

```bash
jq -c . <<'EOF' | ncat 127.0.0.1 5000
{
  "version": 1,
  "id": "from_desktop_2",
  "command": "get_info",
  "payload": {
    "cpu_voltage": {
      "unit": "mV",
      "cores": [1],
      "precision": 2
    },
    "cpu_temperature": {
      "unit": "C",
      "cores": ["first", "last"]
    }
  }
}
EOF
```

the response:

```text
{"version":1,"id":"from_desktop_2","state":207,"payload":{"cpu_temperature":{"result":58.0},"cpu_voltage":{"error":{"code":"query_failed","message":"No data available for target: cpu_voltage"}}}}
```
