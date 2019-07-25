
# Romulus

A mondern alternative to sed

## Sample

given a executable file parse\_ifconfig such as

```
#! /usr/bin/env romulus -f

/^(?P<inter>[a-zA-Z0-9]+): /,/^[a-zA-Z0-9]+:/ {
	/inet (?P<ip>[0-9]{1,3}(\.[0-9]{1,3}){3})/ {
		print("${inter}: ${ip}")
	}
}
```

running `ifconfig | parse\_ifconfig` should yield your current network interfaces which have ips

