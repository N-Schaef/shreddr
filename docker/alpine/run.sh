#!/bin/sh

echo "Ensuring up-to-date tesseract data packages for languages: \"$TESS_LANGUAGES\""
languages=""
packages=""
for lang in $(echo "$TESS_LANGUAGES" | sed "s/,/ /g")
do
    languages="$languages -t $lang"
    if [ "$lang" != "eng" ]
    then
      packages="$packages tesseract-ocr-data-${lang}"
    fi
done

echo "Installing packages $packages"
echo "apk add --no-cache $packages"
apk add --no-cache $packages

echo "Starting Shreddr"
echo "/usr/local/bin/shreddr -s -d /data -c /consume $languages --config /shreddr.yml"
exec /usr/local/bin/shreddr -s -d /data -c /consume $languages --config /shreddr.yml 