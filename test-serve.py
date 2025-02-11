import socket

if __name__ == "__main__":
    address = "127.0.0.1"
    port = 3000
    data = '{"command": "compile", "code": "var x = 1;\\nx+4"}'

    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    try:

        print(f"Connecting to {address}:{port}")
        sock.connect((address, port))

        print(f"Seding data:\n{data}")
        sock.sendall(data.encode('utf-8'))

        print("Closing sending connection")
        sock.shutdown(socket.SHUT_WR)

        response = b""
        while True:
            data = sock.recv(1024)
            if not data:
                break
            response += data

        print(f"Server response:")
        print(response.decode('utf-8'))
    except Exception as e:
        print(f"An error occurred: {e}")
    finally:
        sock.close()
        print("Connection closed.")
