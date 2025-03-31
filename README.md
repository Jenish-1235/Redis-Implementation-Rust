Certainly! Here’s a more detailed and explanatory version of your README file. It includes more insight into why each decision was made and the overall rationale behind the optimizations and implementation choices.

---

# KV-Cache: High-Performance In-Memory Key-Value Store

## Overview
The **KV-Cache** project is designed as a **high-performance in-memory key-value store**. It uses **Rust** and **Tokio** for asynchronous processing and **LRU (Least Recently Used) caching** to automatically manage memory by evicting the least-used entries. This project focuses on achieving **extreme throughput** and **low-latency operations** by utilizing raw TCP sockets, efficient memory management, and network stack optimizations.

- **Tech Stack:** Rust + Tokio + LRU Cache
- **Performance Achieved:** **33K RPS** with **2 Locust workers**
- **Docker Image:** Available at [Docker Hub: jenu1235/kv-server](https://hub.docker.com/r/jenu1235/kv-server)
- **Benchmarking Tools:** Load tested using `wrk`, `k6`, and `Locust`
- **Target Users:** Developers looking for a high-performance, low-latency key-value store that can handle a large volume of read/write requests.

---

## Features

### ✅ **High-Throughput TCP-based Cache**
- We opted for raw **TCP sockets** instead of HTTP to reduce overhead and improve performance. TCP is a connection-oriented protocol that allows for more control over network operations and better throughput than HTTP.

### ✅ **Efficient Memory Management with LRU Eviction**
- The cache uses the **LRU algorithm** to automatically remove the least recently used entries when the memory limit is reached. This ensures that the memory footprint stays within the configured limit (e.g., **1.4GB**).

### ✅ **Optimized for Multi-Core Performance**
- The server is designed to take advantage of multi-core systems by processing requests concurrently with **async tasks**. This allows for high throughput by handling many requests in parallel.

### ✅ **Asynchronous & Lock-Free Reads**
- **Tokio** allows us to handle asynchronous I/O operations, ensuring that read and write operations don't block each other. This increases the responsiveness of the server even under high load.

### ✅ **Minimal Network Overhead**
- We enabled **TCP_NODELAY** to disable **Nagle's algorithm**, which reduces the latency for small requests and improves response time, especially under high concurrency.

### ✅ **Fully Containerized and Deployable on AWS EC2**
- The server is packaged as a Docker container, which can be easily deployed to any environment, including **AWS EC2**. The configuration ensures that it runs efficiently on minimal resources.

### ✅ **Optimized Linux Network Stack for Maximum RPS**
- Network stack optimizations are included in the `install.sh` script to maximize TCP throughput and reduce latency. These adjustments are critical for running the server at scale.

---

## Installation & Running the Server
⚠️ Important EC2 Setup: Run install.sh Before Starting the Server
If you plan to run the server on an AWS EC2 instance, ensure you first execute the install.sh script to optimize the network stack and configure the necessary system parameters for maximum performance. The script will:

 - Disable TCP slow start after idle

 - Increase the maximum number of open file descriptors

 - Optimize read/write buffer sizes

```sh
# Run install.sh to configure the EC2 instance for optimal performance
chmod +x install.sh
sudo ./install.sh
```
#### Why is this necessary?
The optimizations in install.sh are critical to ensure the server can handle high throughput and low latency, especially in cloud environments like EC2, where default configurations may not be optimized for high-load networking.
(For more detailed reading scroll to last section of README.md)

### 1️⃣ **Pull and Run from Docker Hub (Recommended)**
If you prefer a quick start, the pre-built Docker image is available for easy deployment.

```sh
# Pull the pre-built image from Docker Hub
docker pull jenu1235/kv-server

# Run the container (using host network mode for direct communication with the host network)
docker run -p 7171:7171 --network host --ulimit memlock=-1:-1 jenu1235/kv-server
```
- This will launch the server on port `7171` and will allow connections from the host network.

### 2️⃣ **Build and Run Manually**
If you'd like to build the server from source, follow these steps.

#### Install Dependencies
```sh
# Install Rust if you don't have it already
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Navigate to the project directory and install dependencies
cargo install --path .
```

#### Run the Server
```sh
# Build and run the server in release mode
cargo run --release
```

---

## Optimizations & Design Decisions

### 1️⃣ **Switched from HTTP to Raw TCP Sockets**
- **Why this choice?**  
  HTTP introduces significant overhead due to header parsing, content encoding, and additional error handling. TCP sockets, on the other hand, provide more direct and efficient communication, minimizing protocol overhead.

- **Impact:**  
  This transition led to a **~3x increase in Requests Per Second (RPS)** when compared to an HTTP-based implementation.

### 2️⃣ **Used `Bytes` Struct for Efficient Memory Handling**
- **Why `Bytes`?**  
  The `Bytes` struct from the `bytes` crate is specifically designed for efficient memory management when working with binary data. It avoids unnecessary memory copies and allows for efficient handling of large data buffers.

- **Impact:**  
  This reduces unnecessary memory allocations and improves performance, particularly when dealing with frequent read/write operations.

### 3️⃣ **Enabled `TCP_NODELAY` for Low-Latency Responses**
- **Why `TCP_NODELAY`?**  
  By default, TCP uses **Nagle's algorithm**, which batches small data packets to send them together to improve efficiency. While this is effective for large packets, it can introduce latency in scenarios where small packets (e.g., key-value pairs) are sent frequently.

- **Impact:**  
  Disabling Nagle's algorithm with `TCP_NODELAY` reduces latency for small packets and results in a **~15% reduction in response time**.

### 4️⃣ **Optimized with LRU Cache for Faster Lookups**
- **Why LRU Cache?**  
  The LRU (Least Recently Used) caching mechanism ensures that when the cache reaches its memory limit (e.g., **1.4GB**), the least-recently accessed items are evicted. This prevents the cache from growing uncontrollably, ensuring that the system remains performant and stable under load.

- **Impact:**  
  It ensures efficient memory usage by evicting old keys, improving lookup times for frequently used keys.

### 5️⃣ **Multi-Worker Parallel Processing**
- **Why Multi-Worker?**  
  Handling requests in a single thread can become a bottleneck, especially under high concurrency. By using **multi-worker** architecture, the server can handle more requests in parallel, reducing the time each request spends waiting for resources.

- **Impact:**  
  This led to a significant performance boost, increasing RPS from **16K** to **33K** when running with **2 workers**.

### 6️⃣ **Linux Network Stack Tuning** *(Configured via `install.sh`)*
- **What optimizations are made?**
    - **Disable TCP slow start after idle**: This reduces the initial latency when a connection is re-established.
    - **Increase maximum open file descriptors**: Allowing the server to handle more simultaneous connections.
    - **Increase read/write buffer sizes**: Optimizing for high-throughput data transmission.

- **Impact:**  
  These settings improve network performance by minimizing connection setup times and allowing the server to handle a larger number of concurrent connections.

---

## Benchmarking & Load Testing

### 1️⃣ **Locust Multi-Worker Setup**
To benchmark the server, we used **Locust** to simulate a high volume of concurrent connections and test server throughput.

```sh
# Start Locust Master
locust -f sdk/locustfile.py --master

# Start Worker 1
locust -f sdk/locustfile.py --worker --master-host=127.0.0.1

# Start Worker 2
locust -f sdk/locustfile.py --worker --master-host=127.0.0.1
```

**Performance:**
- **1 Worker:** Achieved **16K RPS** (Requests Per Second).
- **2 Workers:** Scaled to **33K RPS**, demonstrating the effectiveness of multi-worker parallel processing.

## Future Enhancements

✅ **Protobuf Serialization for Faster Encoding**
- By moving to **Protobuf**, we aim to improve the serialization/deserialization speed and reduce the size of the data sent over the network, potentially boosting performance by **+20% RPS**.

✅ **Sharded HashMap for Lock-Free Multi-Threaded Reads**
- We plan to implement a **sharded HashMap** to allow multiple threads to concurrently read from the cache without locking, further improving scalability.

✅ **Integration with RESP (Redis Serialization Protocol)**
- To enhance compatibility and further boost performance, we will explore integrating **RESP**, a lightweight binary protocol used by Redis.

---

It looks like the README now has a good structure, covering all the necessary aspects of your project. You can later add the detailed explanation of optimizations done in `install.sh` at the end of the **Optimizations & Design Decisions** section.

Here’s the section that you can add later on as requested:

---

## **Detailed Explanation of `install.sh` Optimizations**

The `install.sh` script plays a crucial role in configuring the system environment for optimal performance when running the KV-Cache server on EC2 or similar cloud environments. Below is a detailed explanation of the optimizations performed:

#### 1️⃣ **Disable TCP Slow Start After Idle**
- **What it Does:**
    - By default, TCP connections go through a slow start phase after being idle, where the transmission rate increases slowly until a certain threshold is reached. Disabling this helps in reducing the latency when the connection is re-established after idle periods.
- **Why it’s Important:**
    - In high-performance scenarios, this adjustment ensures that the server can immediately start transmitting data at the desired rate when a client reconnects, reducing overall latency.

```bash
# Disable TCP slow start after idle
echo 1 > /proc/sys/net/ipv4/tcp_no_metrics_save
```

#### 2️⃣ **Increase Maximum Open File Descriptors**
- **What it Does:**
    - Linux has a limit on the number of file descriptors (handles for open files, sockets, etc.) a process can have. By default, this might be too low for a server handling many concurrent connections. Increasing this limit allows the server to handle more simultaneous connections.
- **Why it’s Important:**
    - This is particularly important for high-throughput systems that handle thousands of requests per second. Raising the file descriptor limit ensures the server can manage a large number of open connections without hitting the system limit.

```bash
# Increase the maximum number of open file descriptors
ulimit -n 100000
```

#### 3️⃣ **Increase Read/Write Buffer Sizes**
- **What it Does:**
    - In this step, we increase the buffer size for reading and writing data over TCP sockets. Larger buffer sizes allow the system to handle more data at once, reducing the need for frequent data copy operations.
- **Why it’s Important:**
    - In high-throughput systems, especially those handling large datasets or high-concurrency loads, having large buffers ensures efficient network operations and reduces the likelihood of dropping data during transmission.

```bash
# Increase read and write buffer sizes
echo "net.core.rmem_max=16777216" >> /etc/sysctl.conf
echo "net.core.wmem_max=16777216" >> /etc/sysctl.conf
```

#### 4️⃣ **Enable TCP BBR (Bottleneck Bandwidth and RTT) Congestion Control**
- **What it Does:**
    - TCP BBR is a congestion control algorithm that improves TCP performance by optimizing for bandwidth and round-trip time. It replaces the default congestion control algorithm (Cubic or Reno) with BBR, which can significantly improve throughput, especially in high-latency networks.
- **Why it’s Important:**
    - For cloud-based applications running on EC2 or other virtual machines with potentially high latencies, BBR can optimize network throughput by adjusting transmission rates based on real-time network performance.

```bash
# Enable BBR congestion control algorithm
sysctl -w net.ipv4.tcp_congestion_control=bbr
```

#### 5️⃣ **Tune Network Parameters**
- **What it Does:**
    - This tweak involves setting parameters that govern the network’s performance in handling large-scale data transfer. These parameters optimize the system for high-concurrency applications by improving TCP flow control and buffer management.
- **Why it’s Important:**
    - These tweaks allow the network stack to efficiently handle the increased traffic load, minimizing packet loss and ensuring data is transferred at optimal rates.

```bash
# Tune network parameters for better performance
sysctl -w net.ipv4.tcp_fin_timeout=30
sysctl -w net.ipv4.tcp_keepalive_time=120
sysctl -w net.ipv4.tcp_rmem="4096 87380 16777216"
sysctl -w net.ipv4.tcp_wmem="4096 87380 16777216"
```

---

## Author
This project is maintained by **jenu1235** 🚀

---