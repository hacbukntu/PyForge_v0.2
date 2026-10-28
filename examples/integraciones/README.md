# Ejemplos de integraciones opcionales

Cada carpeta es un proyecto independiente con permisos explícitos.
Ejecuta desde esa carpeta, no desde la raíz del repositorio.

- [diagnostico](diagnostico/README.md)
- [archivos](archivos/README.md)
- [sqlite](sqlite/README.md)
- [cifrado_texto](cifrado_texto/README.md)
- [contrasenas](contrasenas/README.md)
- [documentos](documentos/README.md)
- [word_pdf](word_pdf/README.md)
- [convertir_doc](convertir_doc/README.md)
- [pdf_paginas](pdf_paginas/README.md)
- [comandos](comandos/README.md)
- [vpn_diagnostico](vpn_diagnostico/README.md)
- [gui](gui/README.md)

Las pruebas TCP/UDP/TLS de bucle local están en `tests/test_integraciones.py`;
no requieren servidores ni credenciales reales. SSH/FTPS y Qt necesitan dependencias
y un entorno de prueba configurado antes de validarse.
