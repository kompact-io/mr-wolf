# Mr wolf 

> What's the time...? [Subbit](https://subbit.xyz) time

This repo is a toy implementation to demo a part of subbit.
In particular, what _featherweight_ might mean.  

## Setup

This project uses [flakes](https://nixos.wiki/wiki/Flakes), 
although you can surely roll-your-own with no trouble.

## Run

There is a bash run script which runs a server and a number of clients.
```bash
./run.sh 10 # 10 clients
```

## TODO

This is just a toy so there are lots of things that could be done. 

- separate client into admin and subscriber
- add auth for admin
- use persistent storage 
