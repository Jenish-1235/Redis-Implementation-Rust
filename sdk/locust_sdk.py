from locust import User, task, events
import socket
import json
import random
import time

HOST = "127.0.0.1"
PORT = 7171

class TCPClient:
    def __init__(self):
        self.socket = None
        self.connect()

    def connect(self):
        self.socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.socket.connect((HOST, PORT))
        self.socket.setsockopt(socket.IPPROTO_TCP, socket.TCP_NODELAY, 1)

    def send_request(self, request_data):
        try:
            if not self.socket:
                self.connect()

            request_json = json.dumps(request_data).encode() + b"\n"
            self.socket.sendall(request_json)

            response_data = b""
            while True:
                chunk = self.socket.recv(1024)
                if not chunk:
                    break
                response_data += chunk
                if response_data.endswith(b"}"):
                    break

            return json.loads(response_data.decode())

        except Exception as e:
            return {"status": "ERROR", "message": str(e)}

    def close(self):
        if self.socket:
            self.socket.close()
            self.socket = None


class TCPUser(User):
    abstract = True

    def __init__(self, environment):
        super().__init__(environment)
        self.client = TCPClient()

    def on_stop(self):
        self.client.close()


class KeyValueStoreUser(TCPUser):
    generated_keys = []

    @task(2)
    def set_key(self):
        key = f"key_{int(time.time() * 1000)}"
        value = f"value_{random.randint(1, 1000)}"
        self.generated_keys.append(key)

        start_time = time.time()
        response = self.client.send_request({"key": key, "value": value})
        total_time = int((time.time() - start_time) * 1000)

        events.request.fire(
            request_type="TCP",
            name="set_key",
            response_time=total_time,
            response_length=len(json.dumps(response)),
            exception=None if response.get("status") == "OK" else Exception(response.get("message")),
        )

    @task(1)
    def get_key(self):
        if not self.generated_keys:
            return

        key = random.choice(self.generated_keys)
        start_time = time.time()
        response = self.client.send_request({"key": key})
        total_time = int((time.time() - start_time) * 1000)

        events.request.fire(
            request_type="TCP",
            name="get_key",
            response_time=total_time,
            response_length=len(json.dumps(response)),
            exception=None if response.get("status") == "OK" else Exception(response.get("message")),
        )


# Run Locust with:
# locust -f locustfile.py --headless -u 100 -r 10 -t 30s
