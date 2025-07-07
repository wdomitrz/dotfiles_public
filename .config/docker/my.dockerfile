FROM my_base:local

# Configure ssh
RUN sudo mkdir /var/run/sshd && \
    (echo "UsePAM yes" && cat /etc/ssh/sshd_config) | \
        sudo sponge /etc/ssh/sshd_config

# Add ssh keys
RUN mkdir --parents ./.ssh
ADD --chown=user https://github.com/wdomitrz.keys ./.ssh/authorized_keys

# Copy recent configs
COPY --chown=user:user "./.git" "./.git"
RUN git checkout -- .

EXPOSE 22
CMD sudo /usr/sbin/sshd -D
