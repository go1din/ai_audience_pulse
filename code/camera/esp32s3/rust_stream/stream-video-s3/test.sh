#!/bin/bash
set -euo pipefail

# Allow overriding target IP via S3_IP environment variable; default to 192.168.129.201
# Optional first argument overrides path portion only (defaults to /stream)
S3_IP="${S3_IP:-192.168.129.201}"
PATH_PART="${1:-/stream}"
[[ "$PATH_PART" =~ ^http?:// ]] && URL="$PATH_PART" || URL="http://$S3_IP$PATH_PART"
TMP_FILE="$(mktemp -t frameXXXXXX.jpg)"
trap 'rm -f "$TMP_FILE"' EXIT

curl -fsS "$URL" -o "$TMP_FILE"

SAMPLE_LEN=64
SAMPLE_HEX="$(hexdump -v -n "$SAMPLE_LEN" -e '16/1 "%02X " "\n"' "$TMP_FILE")"
SAMPLE_NONZERO="$(python3 - "$TMP_FILE" "$SAMPLE_LEN" <<'PY'
import sys
path, length = sys.argv[1], int(sys.argv[2])
with open(path, 'rb') as fh:
    data = fh.read(length)
print(sum(1 for b in data if b))
PY
)"
APPROX_GREEN=$(awk -v nz="$SAMPLE_NONZERO" -v len="$SAMPLE_LEN" 'BEGIN { printf "%.1f", (nz/len)*100 }')
printf 'sample[0..%d]:\n%s(non-zero=%s/%d, approx_green=%s%%)\n' "$((SAMPLE_LEN-1))" "$SAMPLE_HEX" "$SAMPLE_NONZERO" "$SAMPLE_LEN" "$APPROX_GREEN"

magick "$TMP_FILE" -colorspace sRGB -resize 1x1\! \
    -format '%[fx:mean.r] %[fx:mean.g] %[fx:mean.b] %[fx:0.2126*mean.r+0.7152*mean.g+0.0722*mean.b]\n' info: \
| awk '{
    r=$1; g=$2; b=$3; y=$4;               # all in 0..1
    gi = (r+g+b>0)? g/(r+g+b) : 0;
    printf("R=%d G=%d B=%d  greenness=%.1f%%  brightness=%.1f%%\n",
           int(255*r), int(255*g), int(255*b), 100*gi, 100*y);
    if (100*y < 5) { print "⚠︎ frame looks BLACK (<5% brightness)"; }
}'
