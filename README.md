# Xode Blockchain


## Setting up a Xode Node in AWS
### Downloading the xode-node binary
```
$ curl -L "https://drive.usercontent.google.com/download?id=10zStcLL08V3hiCy507CBXMCKCb2VFQsM=xxx" -o raw-xode-node-chainspec.json
```
### Download the xode chainspec
$ wget --no-check-certificate 'https://docs.google.com/uc?export=download&id=19C8s1MdVubYjMFiLBmvwhWxTK6bPeyve' -O raw-xode-node-chainspec.json
### Make the xode-node binary executable
$ chmod +x xode-node
### Make a xode base path directory
$ mkdir xode
### Create a shell script: $ nano xode-node.sh
#!/bin/bash

/home/ubuntu/xode-node \
--chain /home/ubuntu/raw-xode-node-chainspec.json \
--base-path /home/ubuntu/xode \
--rpc-port 9944 \
--pruning archive \
--telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
--name "xode-node-name”
### Make the xode-node.sh executable
$ chmod + xode-node.sh
### Test if it runs
$ ./xode-node.sh
### Create a service: $ sudo nano /etc/systemd/system/xode-collator.service
[Unit]
Description=Xode Node

[Service]
ExecStart=/home/ubuntu/xode-node.sh
Restart=always
RestartSec=120

[Install]
WantedBy=multi-user.target
### Enable the service
$ sudo systemctl enable xode-collator.service
### Start the service
$ sudo systemctl start xode-collator.service
### Wait for 30 seconds and check if the service is running
$ journalctl -f -u xode-collator
### Check the telemetry site to view your node
https://telemetry.polkadot.io/#/0x28cc1df52619f4edd9f0389a7e910a636276075ecc429600f1dd434e281a04e9
