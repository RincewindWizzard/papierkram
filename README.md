# papierkram

A CLI tool to help you to do less tedious tasks at work and get more actual work done (eg. sitting in meetings ;) )

## PPA Installation

This repository hosts its own PPA via GitHub pages.
If you are running a debian/ubuntu machine you can easily add it to your sources and install it with apt.

Download GPG public key:

    curl https://rincewindwizzard.github.io/papierkram/deb/KEY.gpg | gpg --dearmor > /etc/apt/trusted.gpg.d/rincewindwizzard.gpg

Create a new file in /etc/apt/sources.list.d/papierkram.list

    deb [arch=amd64,signed-by=/etc/apt/trusted.gpg.d/rincewindwizzard.gpg]  https://rincewindwizzard.github.io/papierkram/deb/ ./

Run apt

    apt update
    apt install papierkram

Upgrading to the newest version

    apt update
    apt upgrade

