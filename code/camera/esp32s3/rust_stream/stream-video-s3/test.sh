#!/bin/bash

rm -f frame.jpg

curl -s http://192.168.129.111/frame.jpg \
| magick - -colorspace sRGB -resize 1x1\! -format "%[pixel:p{0,0}]\n" info: \
| tr -d '[:alpha:()] ' \
| awk -F, '{r=$1+0; g=$2+0; b=$3+0; printf("R=%d G=%d B=%d  greenness=%.1f%%\n", r,g,b, 100*g/(r+g+b))}'