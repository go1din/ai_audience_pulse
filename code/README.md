# code setup and configuration instructions

## todo:

- [ ] use frontend/html/config.json to configure the frontend part too, currently its embedded in frontend/html/index.html


![thumbsup](../assets/thumbup.jpg)

## install

> Supported distributions: Ubuntu, ...
> (adapt and test on osx too)

`./setup`

## configure

`./configure-network`

## run api

- audio score [audio](audio/)
- image feed endpoints from esp32 [cameras](camera/)

`./run`

## run server

- tls-terminator & reverse-proxy
- static frontend file hosting for the [frontend](frontend/)

> hint: need to run it where Caddyfile is, run `./configure-network` if you have no Caddyfile

`sudo caddy run`

## non-goals at this point:

- ~~[ ] integrate ml backend~~ (unfortunately too much, but it - face detection & emotion classification -  works locally on linux), but we have some machine-learning sauce in the frontend anyway click 🍝 [extras](ml/readme.md) 🍝 if you have too much time for reviewing our efforts
- ~~[ ] integrate second frontend~~ (works with mocked data and is 🎉 beauuuuitiful 🎉, but non-functional- our UX vibe coding study experiment)
