#!/bin/bash
set -euo pipefail

URL="${1:-http://192.168.129.111/frame.jpg}"

curl -fsS "$URL" \
| magick - -colorspace sRGB -resize 1x1\! \
    -format '%[fx:mean.r] %[fx:mean.g] %[fx:mean.b] %[fx:0.2126*mean.r+0.7152*mean.g+0.0722*mean.b]\n' info: \
| awk '{
    r=$1; g=$2; b=$3; y=$4;               # all in 0..1
    gi = (r+g+b>0)? g/(r+g+b) : 0;
    printf("R=%d G=%d B=%d  greenness=%.1f%%  brightness=%.1f%%\n",
           int(255*r), int(255*g), int(255*b), 100*gi, 100*y);
    if (100*y < 5) { print "⚠︎ frame looks BLACK (<5% brightness)"; }
}'