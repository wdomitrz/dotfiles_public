FROM my_base:local

# Copy recent configs
COPY --chown=user:user "./.git" "./.git"
RUN git checkout -- .

CMD ./.local/bin/sanitize_synced_files.sh
