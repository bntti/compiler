import socket
import base64
import subprocess
import os
import json

ADDRESS = "127.0.0.1"
PORT = 3000


def check(a: str, b: str) -> str:
    lines_a = a.split("\n")
    lines_b = b.split("\n")

    lines_a = list(filter(lambda l: l.strip() != "", lines_a))
    lines_b = list(filter(lambda l: l.strip() != "", lines_b))

    if len(lines_a) != len(lines_b):
        return "Different number of lines"

    for i in range(len(lines_a)):
        line_a = lines_a[i].strip()
        line_b = lines_b[i].strip()
        if line_a != line_b:
            return f"{line_a} != {line_b}"

    return ""


def run_test(test_input: str, test_output: str):
    data = {"command": "compile"}
    data["code"] = test_input
    json_data = json.dumps(data)

    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    try:
        sock.connect((ADDRESS, PORT))
        sock.sendall(json_data.encode("utf-8"))
        sock.shutdown(socket.SHUT_WR)

        response = b""
        while True:
            data = sock.recv(1024)
            if not data:
                break
            response += data

        data = response.decode("utf-8")
        binary = base64.b64decode(json.loads(response.decode("utf-8"))["program"])
        with open("a.out", "bw") as f:
            f.write(binary)

        os.system("chmod u+x a.out")

        output = subprocess.check_output(["./a.out"]).decode("utf-8")
        result = check(output, test_output)
        if result == "":
            print("ok!")
        else:
            print(result)
    except Exception as e:
        print(f"Fail {e}")
    finally:
        sock.close()


if __name__ == "__main__":
    tests = {}
    for file in os.listdir("tests"):
        test_name = file.split(".")[0]
        ext = file.split(".")[1]

        with open(f"tests/{file}", "r") as f:
            data = f.readlines()

        if test_name not in tests:
            tests[test_name] = ["", ""]

        if ext == "in":
            tests[test_name][0] = "\n".join(data)
        else:
            tests[test_name][1] = "\n".join(data)

    for test_name in tests:
        print(f"Running test {test_name}", end=" ")
        run_test(tests[test_name][0], tests[test_name][1])
