# papierkram

## PPA Installation

Download GPG public key:

    curl https://rincewindwizzard.github.io/papierkram/deb/KEY.gpg | gpg --dearmor > /etc/apt/trusted.gpg.d/rincewindwizzard.gpg
    #curl http://192.168.178.49:8000/deb/KEY.gpg  | gpg --dearmor > /etc/apt/trusted.gpg.d/rincewindwizzard.gpg

/etc/apt/sources.list.d/papierkram.list

    #deb [arch=amd64,signed-by=/etc/apt/trusted.gpg.d/rincewindwizzard.gpg]  http://192.168.178.49:8000/deb/ ./
    deb [arch=amd64,signed-by=/etc/apt/trusted.gpg.d/rincewindwizzard.gpg]  https://rincewindwizzard.github.io/papierkram/deb/ ./