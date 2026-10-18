#!/bin/sh
set -e

# Ensure data directory exists and is writable by podpdf user
if [ "$(id -u)" = "0" ]; then
    mkdir -p /home/podpdf/data
    chown -R podpdf:podpdf /home/podpdf/data 2>/dev/null || true
    chmod -R 777 /home/podpdf/data 2>/dev/null || true
    exec su -s /bin/sh podpdf -c "/usr/local/bin/podpdf"
else
    exec /usr/local/bin/podpdf
fi
