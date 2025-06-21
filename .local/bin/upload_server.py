#!/usr/bin/env python3
import argparse
from pathlib import Path

from flask import Flask, render_template_string, request

app = Flask(__name__)

HTML_FORM = """\
<!DOCTYPE html>
<html>
<head>
    <title>Upload Files</title>
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
</head>
<body>
    <h2>Upload Files</h2>
    <form method="POST" enctype="multipart/form-data">
        <div>
            <label for="files">Files:</label><br>
            <input type="file" name="files" id="files" multiple><br>
        </div><br>
        <div>
            <label for="directories">Directories:</label><br>
            <input type="file" name="directories" id="directories" multiple webkitdirectory><br>
        </div><br>
        <div>
            <input type="submit" value="Upload">
        </div><br>
    </form>
    {% if uploaded %}
        <h3>Uploaded:</h3>
        <ul>
        {% for uploaded_file in uploaded %}
            <li> {{ uploaded_file }} </li>
        {% endfor %}
        </ul>
    {% endif %}
    {% if existed %}
        <h3>Already exists:</h3>
        <ul>
        {% for existed_file in existed %}
            <li> {{ existed_file }} </li>
        {% endfor %}
        </ul>
    {% endif %}
</body>
</html>
"""


def handle_upload(*, upload_dir: Path) -> tuple[list[str], list[str]]:
    uploaded, existed = [], []
    for file in request.files.getlist("files") + request.files.getlist("directories"):
        file_relative_path = getattr(file, "webkitRelativePath", file.filename)
        if file_relative_path is None or file_relative_path == "":
            continue
        elif (
            not (upload_dir / file_relative_path)
            .resolve()
            .is_relative_to(upload_dir.resolve())
        ):
            continue
        elif (upload_dir / file_relative_path).exists():
            existed.append(file_relative_path)
            continue
        else:
            (upload_dir / file_relative_path).parent.mkdir(parents=True, exist_ok=True)
            file.save(upload_dir / file_relative_path)
            uploaded.append(file_relative_path)
    return uploaded, existed


@app.route("/", methods=["GET", "POST"])
def upload_files() -> str:
    uploaded, existed = [], []
    match request.method:
        case "POST":
            uploaded, existed = handle_upload(upload_dir=app.config["UPLOAD_DIR"])
        case "GET":
            pass
        case _:
            pass
    return render_template_string(HTML_FORM, uploaded=uploaded, existed=existed)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Flask File Upload Server")
    parser.add_argument("--port", type=int, default=8000)
    parser.add_argument("--host", type=str, default="0.0.0.0")
    parser.add_argument("--debug", type=bool, default=None)
    parser.add_argument(
        "--upload-dir",
        type=Path,
        default=Path("./"),
        help="(default: '.')",
    )
    return parser.parse_args()


def main():
    args = parse_args()

    if not args.upload_dir.exists():
        raise ValueError("{upload_dir} doesn't exist")

    app.config["UPLOAD_DIR"] = args.upload_dir
    app.run(
        host=args.host,
        port=args.port,
        debug=args.debug,
    )


if __name__ == "__main__":
    main()
