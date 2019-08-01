
# Romulus

[![Build Status](https://travis-ci.org/eulegang/romulus.svg?branch=master)](https://travis-ci.org/eulegang/romulus)

A mondern alternative to sed

## Versions

Currently while under development I will only be bumping patch versions rather then the normal minor versions 
until I feel like as if romulus is a somewhat mature release and ready for v1

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

