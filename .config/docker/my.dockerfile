FROM my_base:local

# Add recent ssh keys and configs
RUN mkdir --parents ./.ssh
ADD --chown=user https://github.com/wdomitrz.keys ./.ssh/authorized_keys
COPY --chown=user:user "./.git" "./.git"
RUN git checkout -- .

EXPOSE 22
CMD sudo /usr/sbin/sshd -D
