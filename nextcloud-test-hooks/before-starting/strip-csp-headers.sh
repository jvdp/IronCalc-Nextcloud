CONFIG='Header unset Content-Security-Policy'
FILE=/var/www/html/.htaccess

if ! grep -Fqx "$CONFIG" "$FILE"; then
  sed -i "/^<IfModule mod_headers\.c>/a $CONFIG" "$FILE"
fi
