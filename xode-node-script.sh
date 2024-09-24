#!/bin/bash
# Prompt the user for the dynamic node name (e.g., xode-node-name)
read -p "Enter the node name (default: xode-node-name): " NODENAME
NODENAME=${NODENAME:-xode-node-name}  # Default to 'xode-node-name' if no input is provided

# STEP 1: Download the xode-node binary as the xode user
echo "Downloading the xode-node binary..."
sudo -u root bash -c 'curl -L "https://drive.usercontent.google.com/download?id=10zStcLL08V3hiCy507CBXMCKCb2VFQsM&confirm=xxx" -o /home/xode/xode-node'

# STEP 2: Download the xode chainspec as the xode user
echo "Downloading the xode chainspec..."
sudo -u root bash -c 'wget --no-check-certificate "https://docs.google.com/uc?export=download&id=19C8s1MdVubYjMFiLBmvwhWxTK6bPeyve" -O /home/xode/raw-xode-node-chainspec.json'

# STEP 3: Make the xode-node binary executable
echo "Making the xode-node binary executable..."
sudo chmod +x /home/ubuntu/xode-node

# STEP 4: Make a xode base path directory
echo "Creating the xode base path directory..."
sudo -u xode bash -c 'mkdir -p /home/xode/xode'

# STEP 5: Create the xode-node.sh script
echo "Creating the xode-node.sh script..."
sudo bash -c 'cat <<EOL > /home/xode/xode-node.sh
#!/bin/bash
/home/xode/xode-node \\
  --chain /home/xode/raw-xode-node-chainspec.json \\
  --base-path /home/xode/xode \\
  --rpc-port 9944 \\
  --pruning archive \\
  --telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \\
  --name $NODENAME
EOL'

# STEP 6: Make the xode-node.sh executable
echo "Making the xode-node.sh executable..."
sudo chmod +x /home/ubuntu/xode-node.sh

# STEP 7: Test if the script runs as the xode user
echo "Testing the script..."
sudo -u xode bash -c '/home/ubuntu/xode-node.sh &'

# STEP 8: Create a systemd service for the xode collator node
echo "Creating the xode-collator service..."
sudo bash -c 'cat <<EOL > /etc/systemd/system/xode-collator.service
[Unit]
Description=Xode Node

[Service]
ExecStart=/home/ubuntu/xode-node.sh
Restart=always
RestartSec=120

[Install]
WantedBy=multi-user.target
EOL'

# STEP 9: Enable the service
echo "Enabling the xode-collator service..."
sudo systemctl enable xode-collator.service

# STEP 10: Start the service
echo "Starting the xode-collator service..."
sudo systemctl start xode-collator.service

# STEP 11: Wait for 30 seconds and check if the service is running
echo "Waiting for 30 seconds..."
sleep 30
echo "Checking the xode-collator service status..."
sudo journalctl -f -u xode-collator.service
