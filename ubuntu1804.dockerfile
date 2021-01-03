FROM ubuntu:bionic

# Arguments
ARG user=ubuntu
ARG uid=1000
ARG home=/home/ubuntu
ARG shell=/bin/bash

# Basic Utilities
RUN apt-get -qy update && apt-get install -qy sudo lsb-base passwd adduser libsystemd0 libpam-systemd libselinux1 debconf procps
RUN apt-get -qy update && apt-get install -qy --no-install-recommends lsb-release nano net-tools inetutils-ping dnsutils kmod iproute2 isc-dhcp-client less

# Clean apt
RUN apt-get clean && apt-get autoclean && rm -rf /var/lib/apt/lists/*

# Create user
RUN useradd -ms ${shell} --uid ${uid} ${user}\
    && echo "${user} ALL=(ALL) ALL" > "/etc/sudoers.d/${user}"\
    && chmod 0440 "/etc/sudoers.d/${user}"\
    && echo "ubuntu:ubuntu" | chpasswd

# Switch to user
USER "${user}"

# Switch to the workspace
WORKDIR ${home}
