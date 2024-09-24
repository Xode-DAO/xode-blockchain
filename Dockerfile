# Use the latest Ubuntu image
FROM --platform=linux/arm64 ubuntu:22.04

# Install necessary packages
RUN apt-get update
RUN apt-get install -y curl wget supervisor
RUN apt-get clean

# Copy the xode-node script into the container
COPY setup-xode-node.sh .

# Make the script executable
RUN chmod +x setup-xode-node.sh

# Set an entrypoint to run the setup script and then hand control over to bash
ENTRYPOINT ["./setup-xode-node.sh"]

# Provide the default command to start bash after the script finishes
CMD ["/bin/bash"]
