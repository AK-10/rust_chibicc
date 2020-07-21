FROM ubuntu:latest
RUN apt-get update && apt-get install -y curl
RUN DEBIAN_FRONTEND=noninteractive apt-get install -y gcc make git binutils libc6-dev gdb sudo
RUN adduser --disabled-password --gecos '' user
RUN echo 'user ALL=(root) NOPASSWD:ALL' > /etc/sudoers.d/user
USER user
WORKDIR /home/user

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH /home/user/.cargo/bin:$PATH