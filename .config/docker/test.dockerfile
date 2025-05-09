FROM my_base:local

COPY --chown=user:user "./.git" "./.git"
CMD ./.local/bin/sanitize_synced_files.sh
