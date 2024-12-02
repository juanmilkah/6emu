from http.server import HTTPServer, BaseHTTPRequestHandler

import argparse
import pathlib
import subprocess
import json

server = None
path_to_emu = None

def compile_asm(asm):
    temp_file = open("temp-asm.s", mode="w")
    temp_file.write(asm)
    temp_file.flush()
    command = ['nasm', '-f', 'bin', '-o', '/dev/stdout', 'temp-asm.s']
    command2 = [path_to_emu, '--stdin']
    result = subprocess.run(command, capture_output=True)
    if result.returncode != 0 or len(result.stderr)>0:
        temp_file.close()
        return False, result.stderr
    temp_file.close()
    nasm_proc = subprocess.Popen(command, stdout=subprocess.PIPE)
    emu_proc = subprocess.Popen(command2, stdin=nasm_proc.stdout, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    stdout, stderr = emu_proc.communicate()
    pathlib.Path.unlink("temp-asm.s")
    if len(stderr) > 0:
        return False, stderr
    return True, stdout

class Foo(BaseHTTPRequestHandler):
    def do_GET(self):
        self.send_response(200)
        self.send_header("Content-Type", "text/html")
        self.send_header("Access-Control-Allow-Origin", "*")
        self.end_headers()
        file = open("index.html")
        self.wfile.write(file.read().encode())


    def do_UPDATE(self):
        self.send_response(200)
        self.end_headers()
        server.server_close()
        print("Closing server")
        exit(0)
    
    def do_POST(self):
        body = self.rfile.read(int(self.headers["Content-Length"])).decode()
        success, res = compile_asm(body)
        if not success:
            self.send_response(404)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps({"error":res.decode()}).encode())
        else:
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(res)

def main():
    global path_to_emu
    global server
    parser = argparse.ArgumentParser(usage="python3 server.py -f path/to/emu")
    parser.add_argument("-f", "--file", required=True, help="path to emu required")
    parser.add_argument("-p", "--port", help="port", default="6666")
    args = parser.parse_args()
    path_to_emu = args.file
    print(f"serving on port localhost:{args.port}")
    server = HTTPServer(("localhost", int(args.port)), Foo)
    server.serve_forever()

main()

