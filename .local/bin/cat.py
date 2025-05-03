#!/usr/bin/env python3
import sys
from pathlib import Path
from typing import Any


def cat(fp: Path) -> Any:
    match fp.suffix:
        case ".parquet" | ".pqt" | ".pq":
            import pandas as pd

            return pd.read_parquet(fp)
        case ".pickle" | ".pkl":
            import pickle

            return pickle.loads(fp.read_bytes())
        case ".csv":
            import pandas as pd

            return pd.read_csv(fp)
        case suffix:
            raise NotImplementedError(f"Unsupported {suffix=} for {fp}")


def main() -> None:
    for fp in map(Path, sys.argv[1:]):
        try:
            data = cat(fp)
            print(data)
        except NotImplementedError as e:
            print(e, file=sys.stderr)
        except BrokenPipeError:
            pass


if __name__ == "__main__":
    main()
