# code setup and configuration instructions

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

### todo:

- [ ] use frontend/html/config.json to configure the frontend part too, currently its embedded in frontend/html/index.html

### non-goals at this point:

- __[ ] integrate ml backend__ (unfortunately too much, but it - face detection & emotion classification -  works locally on linux), [extras](ml/readme.md)
- __[ ] integrate second frontend__ (works with mocked data and is 🎉 beauuuuitiful 🎉, but non-functional- our UX vibe coding study experiment)
