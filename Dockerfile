# Use the latest Ubuntu image
FROM ubuntu:latest

# Set environment variables
ENV DEBIAN_FRONTEND=noninteractive

# Install necessary packages
RUN apt-get update && \
    apt-get install -y curl wget && \
    apt-get clean

# Copy the xode-node script into the container
COPY xode-node-script.sh .

# Make the script executable
RUN chmod +x xode-node-script.sh

# Set the default command to run the script
CMD ["./xode-node-script.sh"]
