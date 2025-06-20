#!/usr/bin/env python3
import argparse
import http.server
import os
import socketserver
from pathlib import Path

HTML_FORM = """\
<!DOCTYPE html>
<html>
<head><title>Upload a File</title></head>
<body>
    {message}
    <form method="POST" enctype="multipart/form-data">
        <input type="file" name="file" required><br><br>
        <input type="submit" value="Upload">
    </form>
</body>
</html>
"""


class UploadHandler(http.server.BaseHTTPRequestHandler):
    def send_message(self, *, message: str) -> None:
        self.send_response(200)
        self.send_header("Content-Type", "text/html")
        self.end_headers()
        self.wfile.write(HTML_FORM.format(message=message).encode())

    def do_GET(self) -> None:
        self.send_message(message="Upload a file")

    def do_POST(self) -> None:
        content_length = int(self.headers.get("Content-Length", 0))
        content_type = self.headers.get("Content-Type", "")

        if "multipart/form-data" not in content_type:
            return self.send_error(400, "Expected multipart/form-data")

        boundary = content_type.split("boundary=")[-1].encode()
        boundary_line = b"--" + boundary

        # Read entire POST body
        post_data = self.rfile.read(content_length)

        # Find file header and content
        parts = post_data.split(boundary_line)
        for part in parts:
            if b"Content-Disposition" in part and b'name="file"' in part:
                headers, file_data = part.split(b"\r\n\r\n", 1)
                header_lines = headers.decode().split("\r\n")
                filename = None
                for header in header_lines:
                    if "filename=" in header:
                        filename = header.split("filename=")[1].strip().strip('"')
                        break

                if filename:
                    # Remove trailing boundary markers
                    file_data = file_data.removesuffix(b"\r\n")
                    file_data = file_data.removesuffix(b"--")

                    (Path("./") / os.path.basename(filename)).write_bytes(file_data)

                    return self.send_message(message=f"Upload successful: {filename}")

        return self.send_error(400, "File upload failed or malformed request")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Simple file upload HTTP server.")
    parser.add_argument(
        "port",
        nargs="?",
        type=int,
        default=8000,
        help="Port to run the server on (default: 8000)",
    )
    return parser.parse_args()


def main(*, port: int) -> None:
    with socketserver.TCPServer(("", port), UploadHandler) as httpd:
        print(f"Serving on port {port}")
        httpd.serve_forever()


if __name__ == "__main__":
    args = parse_args()
    main(port=args.port)
