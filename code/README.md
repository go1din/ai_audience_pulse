# code setup and configuration instructions

![thumbsup](../assets/thumbup.jpg)

## install
> Supported distributions: Ubuntu, ... (adapt and test on osx too)

`./setup`

## configure

`./configure-network`

## run api for audio score & image feed endpoints

`./run`

## run tls-terminator / a.k.a backend server & static frontend file hosting

### todo:
- [ ] use frontend/html/config.json to configure the frontend part too, currently its embedded in frontend/html/index.html

> hint: need to run it where Caddyfile is, run `./configure-network` if you have no Caddyfile

`sudo caddy run`
