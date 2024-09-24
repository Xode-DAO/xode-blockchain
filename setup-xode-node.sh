#!/bin/bash

NODENAME=$2
NODENAME=${NODENAME:-xode-node-name}  # Default to 'xode-node-name' if no input is provided

# STEP 1: Download the xode-node binary
echo "$NODENAME : downloading xode-node binary..."
curl -L "https://drive.usercontent.google.com/download?id=10zStcLL08V3hiCy507CBXMCKCb2VFQsM&confirm=xxx" -o /home/ubuntu/xode-node

# STEP 2: Download the xode chainspec
echo "$NODENAME : downloading chainspec..."
wget --no-check-certificate "https://docs.google.com/uc?export=download&id=19C8s1MdVubYjMFiLBmvwhWxTK6bPeyve" -O /home/ubuntu/raw-xode-node-chainspec.json

# STEP 3: Make the xode-node binary executable
echo "$NODENAME : making xode-node binary executable..."
chmod +x /home/ubuntu/xode-node

# STEP 4: create xode base path directory
echo "$NODENAME : create xode base path directory..."
mkdir -p /home/ubuntu/xode

# STEP 5: Create the xode-node.sh script
echo "$NODENAME : create xode-node.sh script..."
cat <<EOL > /home/ubuntu/xode-node.sh
#!/bin/bash
/home/ubuntu/xode-node \\
  --chain /home/ubuntu/raw-xode-node-chainspec.json \\
  --base-path /home/ubuntu/xode \\
  --rpc-port 9944 \\
  --pruning archive \\
  --telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \\
  --name "$NODENAME"
EOL

# STEP 6: Make the xode-node.sh executable
echo "$NODENAME : making xode-node.sh executable..."
chmod +x /home/ubuntu/xode-node.sh

# STEP 7: Test if the script runs
#echo "Testing the script..."
#/xode-node.sh &

# STEP 8: Create a systemd service for the xode collator node
echo "$NODENAME : creating the xode-collator service..."
cat <<EOL > /etc/supervisor/conf.d/supervisord.conf
[supervisord]
nodaemon=true

[program:xode-node]
command=/home/ubuntu/xode-node.sh
autostart=true
autorestart=true
stderr_logfile=/var/log/myapp.err.log
stdout_logfile=/var/log/myapp.out.log
EOL

# STEP 9: Wait for 30 seconds and check if the service is running
echo "Waiting for 30 seconds..."
echo "$NODENAME : running..."
supervisord -c /etc/supervisor/conf.d/supervisord.conf
