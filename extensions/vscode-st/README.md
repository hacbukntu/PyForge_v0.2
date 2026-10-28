# Extensión ST para VS Code

Colores TextMate, comentarios, autocierre, sugerencias de palabras, símbolos locales y
funciones de módulos importados. F5 ejecuta el archivo activo con `pyforgec run` como
proceso, sin construir comandos de shell. No proporciona breakpoints ni inspección
paso a paso: aún no hay un adaptador de depuración ST.

Instalación: `code --install-extension dist/extensions/pyforge-st-0.1.0.vsix --force`.
En Windows usa VS Code conectado a WSL, donde está disponible el compilador nativo.
No se instala automáticamente ni cambia el tema global.

`pyforge.compilerPath` permite elegir un compilador. `pyforge.sourceRoot` permite leer
las palabras y módulos desde una copia del código fuente. Sin fuentes usa el
vocabulario empaquetado. Para actualizarlo: `python3 scripts/build-vscode-extension.py`.
La fuente versionable está aquí; `.vscode/` sigue reservado a configuración local.

No puede garantizarse compatibilidad con todas las versiones futuras de VS Code.
Se usan APIs públicas desde 1.89, sin dependencias de Node externas. Hay pruebas con
un anfitrión simulado; la apariencia y teclas deben comprobarse en el editor real.
