FROM my:local

RUN sudo apt-get install --yes npm nodejs node-typescript
RUN npm config set prefix '~/.local/'
RUN npm install --global @vscode/vsce
