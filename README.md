# Braun

Braun is a tiny utility to weld unix pipes to websockets. 

## Installation

    git clone https://github.com/holtchesley/braun
    cd braun
    cargo install


## Usage

Braun may be used as a server

    braun server 'server.address:port' 

or a client

    braun client 'ws://server.address:port'

In client mode, braun will send everything from stdin across the websocket, and echo out anything it receives to stdout.

In server mode, braun will send everything from stdin to _every_ client connected, and echo everything it receives to stdout.
