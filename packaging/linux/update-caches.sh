#!/bin/sh
# Los archivos pertenecen al paquete. Refresca índices al instalar/retirar;
# no impone un editor predeterminado en la configuración personal del usuario.
set -e
if command -v update-mime-database >/dev/null 2>&1; then
    update-mime-database /usr/share/mime
fi
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database -q /usr/share/applications
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1 && [ -f /usr/share/icons/hicolor/index.theme ]; then
    gtk-update-icon-cache -q -t /usr/share/icons/hicolor
fi
